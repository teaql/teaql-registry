use axum::http::{Method, Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

use teaql_registry::api::{build_app, AppState};
use teaql_registry::blobstore::{BlobStore, MemoryBlobStore};
use teaql_registry_core::{service_runtime, ServiceRuntimeConfig};

async fn setup_search_test_app() -> axum::Router {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime error"));
    runtime.ensure_schema().await.expect("Schema error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("search-store"));
    blobstore.init().await.expect("Blobstore init error");

    build_app(AppState { runtime, blobstore })
}

#[tokio::test(flavor = "multi_thread")]
async fn test_search_components_and_assets() {
    let app = setup_search_test_app().await;

    // 1. Search components with keyword
    let req = Request::builder()
        .method(Method::GET)
        .uri("/service/rest/v1/search?keyword=commons")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("items").is_some());
    assert!(json.get("total").is_some());

    // 2. Search assets
    let req_assets = Request::builder()
        .method(Method::GET)
        .uri("/service/rest/v1/search/assets")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp_assets = app.oneshot(req_assets).await.unwrap();
    assert_eq!(resp_assets.status(), StatusCode::OK);
}
