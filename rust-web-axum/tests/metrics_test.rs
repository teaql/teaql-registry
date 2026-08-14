use axum::http::{Method, Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

use teaql_registry::api::{build_app, AppState};
use teaql_registry::blobstore::{BlobStore, MemoryBlobStore};
use teaql_registry_core::{service_runtime, ServiceRuntimeConfig};

#[tokio::test(flavor = "multi_thread")]
async fn test_prometheus_metrics_endpoint() {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime error"));
    runtime.ensure_schema().await.expect("Schema error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("metrics-store"));
    blobstore.init().await.expect("Blobstore init error");

    let app = build_app(AppState { runtime, blobstore });

    let req = Request::builder()
        .method(Method::GET)
        .uri("/metrics")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_str.contains("teaql_registry_http_requests_total"));
    assert!(body_str.contains("teaql_registry_repositories_count"));
    assert!(body_str.contains("teaql_registry_storage_bytes_total"));
}
