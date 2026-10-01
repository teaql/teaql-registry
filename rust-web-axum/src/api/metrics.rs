use axum::{
    extract::State,
    http::{header, Response, StatusCode},
};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::api::AppState;
use crate::services::{
    service_log::{DROPPED_SERVICE_LOG_COUNT, FAILED_SERVICE_LOG_WRITE_COUNT},
    AssetService, RepositoryService,
};

pub static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

pub async fn handle_metrics(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
) -> Response<axum::body::Body> {
    let requests = REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);

    let repos = RepositoryService::list(&request.runtime)
        .await
        .unwrap_or_default();
    let referenced_asset_blob_ids = AssetService::list_all_referenced_blob_ids(&request.runtime)
        .await
        .unwrap_or_default();
    let blobs = AssetService::list_all_blobs(&request.runtime)
        .await
        .unwrap_or_default();

    let total_storage_bytes: i64 = blobs.iter().map(|b| b.blob_size()).sum();

    let mut body = String::new();

    body.push_str(
        "# HELP teaql_registry_http_requests_total Total number of HTTP requests processed\n",
    );
    body.push_str("# TYPE teaql_registry_http_requests_total counter\n");
    body.push_str(&format!(
        "teaql_registry_http_requests_total {}\n\n",
        requests
    ));

    body.push_str(
        "# HELP teaql_registry_repositories_count Total number of configured repositories\n",
    );
    body.push_str("# TYPE teaql_registry_repositories_count gauge\n");
    body.push_str(&format!(
        "teaql_registry_repositories_count {}\n\n",
        repos.len()
    ));

    body.push_str("# HELP teaql_registry_assets_count Total number of artifact assets\n");
    body.push_str("# TYPE teaql_registry_assets_count gauge\n");
    body.push_str(&format!(
        "teaql_registry_assets_count {}\n\n",
        referenced_asset_blob_ids.len()
    ));

    body.push_str(
        "# HELP teaql_registry_storage_bytes_total Total size of stored artifact blobs in bytes\n",
    );
    body.push_str("# TYPE teaql_registry_storage_bytes_total gauge\n");
    body.push_str(&format!(
        "teaql_registry_storage_bytes_total {}\n",
        total_storage_bytes
    ));

    body.push_str(
        "\n# HELP teaql_registry_service_logs_dropped_total Operational service logs dropped because the bounded writer was saturated\n",
    );
    body.push_str("# TYPE teaql_registry_service_logs_dropped_total counter\n");
    body.push_str(&format!(
        "teaql_registry_service_logs_dropped_total {}\n",
        DROPPED_SERVICE_LOG_COUNT.load(Ordering::Relaxed)
    ));
    body.push_str(
        "\n# HELP teaql_registry_service_log_write_failures_total Operational service log database write failures\n",
    );
    body.push_str("# TYPE teaql_registry_service_log_write_failures_total counter\n");
    body.push_str(&format!(
        "teaql_registry_service_log_write_failures_total {}\n",
        FAILED_SERVICE_LOG_WRITE_COUNT.load(Ordering::Relaxed)
    ));

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )
        .body(axum::body::Body::from(body))
        .expect("failed to build metrics response")
}
