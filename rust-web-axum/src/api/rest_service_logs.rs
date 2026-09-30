use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::services::ServiceLogService;

#[derive(Deserialize)]
pub struct ServiceLogQuery {
    pub tenant_id: Option<u64>,
    pub log_type: Option<String>,
    pub operator_name: Option<String>,
    pub action: Option<String>,
    pub page: Option<usize>,
    pub size: Option<usize>,
}

#[derive(Serialize)]
pub struct ServiceLogItem {
    pub id: u64,
    pub tenant_id: u64,
    pub event_time: String,
    pub log_type: String,
    pub operator_id: i64,
    pub operator_name: String,
    pub client_ip: String,
    pub action: String,
    pub repository_name: String,
    pub artifact_path: String,
    pub format_name: String,
    pub content_size: i64,
    pub status: String,
    pub error_message: String,
}

pub async fn list_service_logs(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Query(params): Query<ServiceLogQuery>,
) -> Response {
    let limit = params.size.unwrap_or(50).min(200);
    let offset = params.page.unwrap_or(0).saturating_mul(limit);

    match ServiceLogService::query_logs(
        &request.runtime,
        params.tenant_id,
        params.log_type.as_deref(),
        params.operator_name.as_deref(),
        params.action.as_deref(),
        offset,
        limit,
    )
    .await
    {
        Ok(logs) => {
            let items: Vec<ServiceLogItem> = logs
                .into_iter()
                .map(|log| ServiceLogItem {
                    id: log.id(),
                    tenant_id: log.tenant_id(),
                    event_time: log.event_time().to_datetime().to_rfc3339(),
                    log_type: log.log_type().to_string(),
                    operator_id: log.operator_id(),
                    operator_name: log.operator_name().to_string(),
                    client_ip: log.client_ip().to_string(),
                    action: log.action().to_string(),
                    repository_name: log.repository_name().to_string(),
                    artifact_path: log.artifact_path().to_string(),
                    format_name: log.format_name().to_string(),
                    content_size: log.content_size(),
                    status: log.status().to_string(),
                    error_message: log.error_message().to_string(),
                })
                .collect();
            Json(items).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query service logs: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
