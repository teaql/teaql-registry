#![recursion_limit = "256"]

mod common;

use axum::http::{Method, Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

use teaql_registry::api::{build_app, AppState};
use teaql_registry::blobstore::{BlobStore, MemoryBlobStore};
use teaql_registry::services::{
    BlobStoreService, ComponentService, RepositoryService, TenantService,
};
use teaql_registry_core::service_runtime;

async fn setup_search_test_app() -> axum::Router {
    let config = common::runtime_config();
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime error"));
    runtime.ensure_schema().await.expect("Schema error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("search-store"));
    blobstore.init().await.expect("Blobstore init error");

    build_app(AppState::new(runtime, blobstore))
}

#[test]
fn test_search_components_and_assets() {
    common::run_with_large_stack(test_search_components_and_assets_body);
}

async fn test_search_components_and_assets_body() {
    let app = setup_search_test_app().await;

    // 1. Search components with keyword
    let req = Request::builder()
        .method(Method::GET)
        .uri("/service/rest/v1/search?keyword=commons")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
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

#[test]
fn test_repository_facets_follow_component_filters() {
    common::run_with_large_stack(test_repository_facets_body);
}

async fn test_repository_facets_body() {
    let config = common::runtime_config();
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime error"));
    runtime.ensure_schema().await.expect("Schema error");
    if TenantService::find_tenant_by_code(&runtime, "default")
        .await
        .expect("find default tenant")
        .is_none()
    {
        TenantService::create_tenant_with_platform(&runtime, 1, "Default", "default", "Test")
            .await
            .expect("create default tenant");
    }
    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("facet-store"));
    blobstore.init().await.expect("Blobstore init error");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let package_name = format!("facet-package-{suffix}");
    let store = BlobStoreService::create(
        &runtime,
        &format!("facet-store-{suffix}"),
        "/tmp/facet-blobs",
        true,
    )
    .await
    .expect("create blob store");

    let mut expected = Vec::new();
    for (format, recipe, group) in [
        ("MAVEN2", "maven2-hosted", "facet.group"),
        ("NPM", "npm-hosted", "other.group"),
    ] {
        let repo_name = format!("facet-{}-{suffix}", format.to_lowercase());
        let repo = RepositoryService::create(
            &runtime,
            &repo_name,
            recipe,
            "HOSTED",
            format,
            "ALLOW_WRITE",
            store.id(),
            true,
            "",
        )
        .await
        .expect("create repository");
        let content_repo = RepositoryService::get_content_repository(&runtime, repo.id())
            .await
            .expect("load content repository")
            .expect("content repository exists");
        ComponentService::create(
            &runtime,
            content_repo.id(),
            group,
            &package_name,
            "1.0.0",
            "1.0.0",
            "package",
        )
        .await
        .expect("create component");
        expected.push((repo_name, format.to_string()));
    }

    let app = build_app(AppState::new(runtime, blobstore));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/service/rest/v1/search/facets?name={package_name}"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let facets = json["repositories"].as_array().unwrap();
    assert_eq!(facets.len(), 2);
    for (name, format) in &expected {
        assert!(facets
            .iter()
            .any(|facet| facet["name"] == *name && facet["format"] == *format));
    }

    let uppercase = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/service/rest/v1/search/facets?name={}",
                    package_name.to_uppercase()
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(uppercase.status(), StatusCode::OK);
    let body = axum::body::to_bytes(uppercase.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["repositories"].as_array().unwrap().len(), 2);

    let filtered = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/service/rest/v1/search/facets?name={package_name}&group=facet.group"
                ))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(filtered.status(), StatusCode::OK);
    let body = axum::body::to_bytes(filtered.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let facets = json["repositories"].as_array().unwrap();
    assert_eq!(facets.len(), 1);
    assert_eq!(facets[0]["name"], expected[0].0);
}
