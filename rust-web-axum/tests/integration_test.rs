use axum::http::StatusCode;
use bytes::Bytes;
use nexus_repository_service_core::{service_runtime, ServiceRuntimeConfig};
use nexus_repository_service_core_workspace::{
    api::{build_app, AppState},
    blobstore::{BlobStore, S3BlobStore},
    security::password::hash_password,
    services::{
        BlobStoreService, ComponentService, RepositoryService, SecurityService,
    },
};
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_test_app() -> axum::Router {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };

    let runtime = service_runtime(config).await.expect("Runtime connect error");
    runtime.ensure_schema().await.expect("Schema init error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(S3BlobStore::from_env("test-blobs"));
    blobstore.init().await.expect("Blobstore init error");

    // Seed baseline test data
    let blob_stores = BlobStoreService::list(&runtime).await.unwrap();
    let bs = if let Some(s) = blob_stores.into_iter().find(|s| s.name() == "default") {
        s
    } else {
        BlobStoreService::create(&runtime, "default", "data/blobs/default", true)
            .await
            .unwrap()
    };

    let repos = RepositoryService::list(&runtime).await.unwrap();
    if !repos.iter().any(|r| r.name() == "maven-releases") {
        RepositoryService::create(
            &runtime,
            "maven-releases",
            "maven2-hosted",
            "HOSTED",
            "MAVEN2",
            "ALLOW_ONCE",
            bs.id(),
            true,
            "",
        )
        .await
        .unwrap();
    }
    if !repos.iter().any(|r| r.name() == "maven-public") {
        RepositoryService::create(
            &runtime,
            "maven-public",
            "maven2-group",
            "GROUP",
            "MAVEN2",
            "READ_ONLY",
            bs.id(),
            true,
            "",
        )
        .await
        .unwrap();
    }
    if !repos.iter().any(|r| r.name() == "raw-hosted") {
        RepositoryService::create(
            &runtime,
            "raw-hosted",
            "raw-hosted",
            "HOSTED",
            "RAW",
            "ALLOW_WRITE",
            bs.id(),
            true,
            "",
        )
        .await
        .unwrap();
    }

    let users = SecurityService::list_users(&runtime).await.unwrap();
    if !users.iter().any(|u| u.username() == "admin") {
        let password_hash = hash_password("admin123");
        SecurityService::create_user(
            &runtime,
            "admin",
            "Administrator",
            "User",
            "admin@example.com",
            &password_hash,
        )
        .await
        .unwrap();
    }

    let app_state = AppState {
        runtime: Arc::new(runtime),
        blobstore,
    };

    build_app(app_state)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_debug_component_query() {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = service_runtime(config).await.unwrap();
    runtime.ensure_schema().await.unwrap();
    let res = ComponentService::find_or_create(&runtime, 1, "com.example", "demo", "1.0.0", "jar").await;
    println!("find_or_create result: {:?}", res);
    assert!(res.is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rest_status_endpoints() {
    let app = setup_test_app().await;

    // Test /service/rest/v1/status
    let request = http::Request::builder()
        .uri("/service/rest/v1/status")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test /service/rest/v1/status/writable
    let request = http::Request::builder()
        .uri("/service/rest/v1/status/writable")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rest_repositories_and_security() {
    let app = setup_test_app().await;

    // List repositories
    let request = http::Request::builder()
        .uri("/service/rest/v1/repositories")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // List blobstores
    let request = http::Request::builder()
        .uri("/service/rest/v1/blobstores")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // List security users
    let request = http::Request::builder()
        .uri("/service/rest/v1/security/users")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Anonymous config
    let request = http::Request::builder()
        .uri("/service/rest/v1/security/anonymous")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_artifact_upload_download_and_group_routing() {
    let app = setup_test_app().await;

    let jar_data = b"PK\x03\x04fake-jar-binary-content-1.0.0";

    let version = format!("1.0.{}", uuid::Uuid::new_v4().simple());
    let path = format!("/repository/maven-releases/com/example/demo/{}/demo-{}.jar", version, version);
    let group_path = format!("/repository/maven-public/com/example/demo/{}/demo-{}.jar", version, version);

    // 1. Upload Maven artifact to hosted repo
    let put_req = http::Request::builder()
        .method(http::Method::PUT)
        .uri(&path)
        .header(http::header::CONTENT_TYPE, "application/java-archive")
        .body(axum::body::Body::from(Bytes::from_static(jar_data)))
        .unwrap();
    let put_resp = app.clone().oneshot(put_req).await.unwrap();
    let status = put_resp.status();
    let body_bytes = axum::body::to_bytes(put_resp.into_body(), 1024 * 1024).await.unwrap();
    let err_msg = String::from_utf8_lossy(&body_bytes);
    assert_eq!(status, StatusCode::CREATED, "PUT failed with: {}", err_msg);

    // 2. Download from hosted repo
    let get_req = http::Request::builder()
        .method(http::Method::GET)
        .uri(&path)
        .body(axum::body::Body::empty())
        .unwrap();
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(get_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(body_bytes.as_ref(), jar_data);

    // 3. Download via Group repository (maven-public)
    let group_req = http::Request::builder()
        .method(http::Method::GET)
        .uri(&group_path)
        .body(axum::body::Body::empty())
        .unwrap();
    let group_resp = app.clone().oneshot(group_req).await.unwrap();
    assert_eq!(group_resp.status(), StatusCode::OK);
    let group_bytes = axum::body::to_bytes(group_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(group_bytes.as_ref(), jar_data);

    // 4. Query Components REST API
    let comp_req = http::Request::builder()
        .method(http::Method::GET)
        .uri("/service/rest/v1/components?repository=maven-releases")
        .body(axum::body::Body::empty())
        .unwrap();
    let comp_resp = app.clone().oneshot(comp_req).await.unwrap();
    assert_eq!(comp_resp.status(), StatusCode::OK);

    // 5. Query Assets REST API
    let asset_req = http::Request::builder()
        .method(http::Method::GET)
        .uri("/service/rest/v1/assets?repository=maven-releases")
        .body(axum::body::Body::empty())
        .unwrap();
    let asset_resp = app.oneshot(asset_req).await.unwrap();
    assert_eq!(asset_resp.status(), StatusCode::OK);
}
