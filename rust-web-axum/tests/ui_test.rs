use axum::http::{Method, Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

use teaql_registry::api::{build_app, AppState};
use teaql_registry::blobstore::{BlobStore, MemoryBlobStore};
use teaql_registry_core::{service_runtime, ServiceRuntimeConfig};

#[tokio::test(flavor = "multi_thread")]
async fn test_embedded_ui_console() {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime error"));
    runtime.ensure_schema().await.expect("Schema error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("ui-store"));
    blobstore.init().await.expect("Blobstore init error");

    let app = build_app(AppState { runtime, blobstore });

    // 1. GET /
    let req_root = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp_root = app.clone().oneshot(req_root).await.unwrap();
    assert_eq!(resp_root.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp_root.into_body(), 1024 * 1024).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body_str.contains("TeaQL Registry Console"));

    // 2. GET /console
    let req_console = Request::builder()
        .method(Method::GET)
        .uri("/console")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp_console = app.oneshot(req_console).await.unwrap();
    assert_eq!(resp_console.status(), StatusCode::OK);
}
