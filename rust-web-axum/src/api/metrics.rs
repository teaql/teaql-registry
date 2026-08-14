use axum::{
    extract::State,
    http::{header, Response, StatusCode},
};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::api::AppState;
use crate::services::{AssetService, RepositoryService};

pub static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

pub async fn handle_metrics(State(state): State<AppState>) -> Response<axum::body::Body> {
    let requests = REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);

    let repos = RepositoryService::list(&state.runtime).await.unwrap_or_default();
    let assets = AssetService::list_all_assets(&state.runtime).await.unwrap_or_default();
    let blobs = AssetService::list_all_blobs(&state.runtime).await.unwrap_or_default();

    let total_storage_bytes: i64 = blobs.iter().map(|b| b.blob_size()).sum();

    let mut body = String::new();

    body.push_str("# HELP teaql_registry_http_requests_total Total number of HTTP requests processed\n");
    body.push_str("# TYPE teaql_registry_http_requests_total counter\n");
    body.push_str(&format!("teaql_registry_http_requests_total {}\n\n", requests));

    body.push_str("# HELP teaql_registry_repositories_count Total number of configured repositories\n");
    body.push_str("# TYPE teaql_registry_repositories_count gauge\n");
    body.push_str(&format!("teaql_registry_repositories_count {}\n\n", repos.len()));

    body.push_str("# HELP teaql_registry_assets_count Total number of artifact assets\n");
    body.push_str("# TYPE teaql_registry_assets_count gauge\n");
    body.push_str(&format!("teaql_registry_assets_count {}\n\n", assets.len()));

    body.push_str("# HELP teaql_registry_storage_bytes_total Total size of stored artifact blobs in bytes\n");
    body.push_str("# TYPE teaql_registry_storage_bytes_total gauge\n");
    body.push_str(&format!("teaql_registry_storage_bytes_total {}\n", total_storage_bytes));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")
        .body(axum::body::Body::from(body))
        .unwrap()
}
