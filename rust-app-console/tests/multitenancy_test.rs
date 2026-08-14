use axum::http::{header, Method, Request, StatusCode};
use nexus_repository_service_core::{service_runtime, ServiceRuntimeConfig};
use nexus_repository_service_core_workspace::{
    api::{build_app, AppState},
    blobstore::FileBlobStore,
    context::NexusContextExt,
    services::{BlobStoreService, RepositoryService, SecurityService, TenantService},
};
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_tenant_test_app() -> axum::Router {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime connect error"));
    runtime.ensure_schema().await.expect("Schema init error");

    let test_dir =
        std::env::temp_dir().join(format!("nexus_tenant_test_{}", uuid::Uuid::new_v4().simple()));
    let blobstore = Arc::new(FileBlobStore::new(&test_dir, "tenant-blobs"));
    blobstore.init().await.expect("Blobstore init error");

    build_app(AppState { runtime, blobstore })
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tenant_creation_and_provisioning_lifecycle() {
    let app = setup_tenant_test_app().await;

    let tenant_name = format!("Tenant-{}", uuid::Uuid::new_v4().simple());
    let payload = serde_json::json!({
        "name": tenant_name,
        "code": "test-code",
        "description": "Test Tenant Description",
        "blobRoot": "/tmp/test_blobs"
    });

    // 1. Create Tenant via REST API: POST /service/rest/v1/tenants
    let create_req = Request::builder()
        .method(Method::POST)
        .uri("/service/rest/v1/tenants")
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = axum::body::to_bytes(create_resp.into_body(), 1024 * 1024).await.unwrap();
    let tenant_json: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let tenant_id_str = tenant_json["id"].as_str().unwrap();
    let tenant_id: u64 = tenant_id_str.parse().unwrap();

    // 2. Query Tenant details: GET /service/rest/v1/tenants/:id
    let get_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/service/rest/v1/tenants/{}", tenant_id))
        .body(axum::body::Body::empty())
        .unwrap();
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = axum::body::to_bytes(get_resp.into_body(), 1024 * 1024).await.unwrap();
    let get_json: serde_json::Value = serde_json::from_slice(&get_body).unwrap();
    assert_eq!(get_json["name"], tenant_name);

    // 3. List all tenants: GET /service/rest/v1/tenants
    let list_req = Request::builder()
        .method(Method::GET)
        .uri("/service/rest/v1/tenants")
        .body(axum::body::Body::empty())
        .unwrap();
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = axum::body::to_bytes(list_resp.into_body(), 1024 * 1024).await.unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
    assert!(list_json.as_array().unwrap().iter().any(|t| t["id"] == tenant_id_str));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_context_tenant_isolation() {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let base_runtime = service_runtime(config.clone()).await.expect("Runtime connect error");

    // 1. Create two isolated tenants under Platform
    let tenant_a = TenantService::create_tenant(&base_runtime, "Acme Corporation", "acme").await.unwrap();
    let tenant_b = TenantService::create_tenant(&base_runtime, "Beta Enterprises", "beta").await.unwrap();

    // 2. Provision isolated repositories and blobstores for each tenant
    TenantService::provision_tenant(&base_runtime, tenant_a.id(), "/tmp/tenant_a_blobs").await.unwrap();
    TenantService::provision_tenant(&base_runtime, tenant_b.id(), "/tmp/tenant_b_blobs").await.unwrap();

    // 3. Create distinct UserContext for Tenant A and Tenant B
    let mut ctx_a = service_runtime(config.clone()).await.unwrap();
    ctx_a.set_tenant(tenant_a.id(), "Acme Corporation");

    let mut ctx_b = service_runtime(config).await.unwrap();
    ctx_b.set_tenant(tenant_b.id(), "Beta Enterprises");

    assert_eq!(ctx_a.tenant_id(), tenant_a.id());
    assert_eq!(ctx_b.tenant_id(), tenant_b.id());

    // 4. Verify Repository isolation transparently via UserContext (no extra tenant_id param passed!)
    let repos_a = RepositoryService::list(&ctx_a).await.unwrap();
    let repos_b = RepositoryService::list(&ctx_b).await.unwrap();

    assert!(repos_a.iter().any(|r| r.name() == "maven-releases"));
    assert!(repos_b.iter().any(|r| r.name() == "maven-releases"));
    assert_ne!(repos_a[0].id(), repos_b[0].id());

    // 5. Verify BlobStore isolation transparently via UserContext
    let bs_a = BlobStoreService::list(&ctx_a).await.unwrap();
    let bs_b = BlobStoreService::list(&ctx_b).await.unwrap();

    assert_eq!(bs_a.len(), 1);
    assert_eq!(bs_b.len(), 1);
    assert!(bs_a[0].path().contains(&format!("tenant_{}", tenant_a.id())));
    assert!(bs_b[0].path().contains(&format!("tenant_{}", tenant_b.id())));
    assert_ne!(bs_a[0].id(), bs_b[0].id());

    // 6. Verify User isolation transparently via UserContext
    let users_a = SecurityService::list_users(&ctx_a).await.unwrap();
    let users_b = SecurityService::list_users(&ctx_b).await.unwrap();

    assert_eq!(users_a.len(), 1);
    assert_eq!(users_b.len(), 1);
    assert_eq!(users_a[0].username(), "admin");
    assert_eq!(users_b[0].username(), "admin");
    assert_ne!(users_a[0].id(), users_b[0].id());
}
