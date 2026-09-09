#![recursion_limit = "256"]

mod common;

use axum::http::{header, Method, Request, StatusCode};
use std::sync::Arc;
use teaql_registry::{
    api::{build_app, AppState},
    blobstore::{BlobStore, S3BlobStore},
    context::NexusContextExt,
    security::hash_password,
    services::{BlobStoreService, RepositoryService, SecurityService, TenantService},
};
use teaql_registry_core::service_runtime;
use tower::ServiceExt;

async fn setup_tenant_test_app() -> axum::Router {
    let config = common::runtime_config();
    let postgres_config = config
        .database_url
        .parse::<tokio_postgres::Config>()
        .expect("PostgreSQL config error");
    let manager = deadpool_postgres::Manager::new(postgres_config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .build()
        .expect("Pool build error");
    let mut runtime = teaql_registry_core::service_runtime_from_pool(pool.clone())
        .await
        .expect("Runtime connect error");
    runtime.set_tenant(1, "Default Tenant");
    let runtime = Arc::new(runtime);
    runtime.ensure_schema().await.expect("Schema init error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(S3BlobStore::from_env("tenant-blobs"));
    blobstore.init().await.expect("Blobstore init error");

    build_app(AppState::with_runtime_pool_unsecured_for_tests(
        runtime, pool, blobstore,
    ))
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tenant_creation_and_provisioning_lifecycle() {
    let app = setup_tenant_test_app().await;

    let unique_suffix = uuid::Uuid::new_v4().simple().to_string();
    let tenant_name = format!("Tenant-{unique_suffix}");
    let tenant_code = format!("test-{unique_suffix}");
    let payload = serde_json::json!({
        "name": tenant_name,
        "code": tenant_code,
        "description": "Test Tenant Description",
        "blobRoot": "/tmp/test_blobs",
        "adminPassword": "Tenant-Test-Password-42!"
    });

    // 1. Create Tenant via REST API: POST /service/rest/v1/tenants
    let create_req = Request::builder()
        .method(Method::POST)
        .uri("/service/rest/v1/tenants")
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&payload).unwrap(),
        ))
        .unwrap();
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = axum::body::to_bytes(create_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
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
    let get_body = axum::body::to_bytes(get_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
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
    let list_body = axum::body::to_bytes(list_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
    assert!(list_json
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["id"] == tenant_id_str));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_context_tenant_isolation() {
    let config = common::runtime_config();
    let base_runtime = service_runtime(config.clone())
        .await
        .expect("Runtime connect error");
    base_runtime
        .ensure_schema()
        .await
        .expect("Schema init error");

    // 1. Create two isolated tenants under Platform
    let tenant_a = TenantService::create_tenant(&base_runtime, "Acme Corporation", "acme")
        .await
        .unwrap();
    let tenant_b = TenantService::create_tenant(&base_runtime, "Beta Enterprises", "beta")
        .await
        .unwrap();

    // 2. Create distinct UserContext for Tenant A and Tenant B
    let mut ctx_a = service_runtime(config.clone()).await.unwrap();
    ctx_a.set_tenant(tenant_a.id(), "Acme Corporation");

    let mut ctx_b = service_runtime(config).await.unwrap();
    ctx_b.set_tenant(tenant_b.id(), "Beta Enterprises");

    // 3. Provision through the corresponding tenant context.
    let password_hash = hash_password("Tenant-Test-Password-42!");
    let users = [(
        "admin",
        "Tenant",
        "Administrator",
        "admin@tenant.test",
        password_hash.as_str(),
    )];
    TenantService::provision_tenant(&ctx_a, tenant_a.id(), "/tmp/tenant_a_blobs", &users)
        .await
        .unwrap();
    TenantService::provision_tenant(&ctx_b, tenant_b.id(), "/tmp/tenant_b_blobs", &users)
        .await
        .unwrap();

    assert_eq!(ctx_a.tenant_id(), tenant_a.id());
    assert_eq!(ctx_b.tenant_id(), tenant_b.id());

    // 4. Verify Repository isolation transparently via UserContext (no extra tenant_id param passed!)
    let repos_a = RepositoryService::list(&ctx_a).await.unwrap();
    let repos_b = RepositoryService::list(&ctx_b).await.unwrap();

    assert!(repos_a.iter().any(|r| r.name() == "maven-releases"));
    assert!(repos_b.iter().any(|r| r.name() == "maven-releases"));
    assert!(repos_a.iter().any(|r| r.name() == "swift-hosted"));
    assert!(repos_b.iter().any(|r| r.name() == "swift-hosted"));
    assert_ne!(repos_a[0].id(), repos_b[0].id());

    // 5. Verify BlobStore isolation transparently via UserContext
    let bs_a = BlobStoreService::list(&ctx_a).await.unwrap();
    let bs_b = BlobStoreService::list(&ctx_b).await.unwrap();

    assert_eq!(bs_a.len(), 1);
    assert_eq!(bs_b.len(), 1);
    assert!(bs_a[0]
        .path()
        .contains(&format!("tenant_{}", tenant_a.id())));
    assert!(bs_b[0]
        .path()
        .contains(&format!("tenant_{}", tenant_b.id())));
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
