use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

use crate::api::AppState;
use crate::services::{BlobStoreGcService, CleanupPolicy, CleanupService};

#[derive(Debug, Deserialize)]
pub struct RunCleanupRequest {
    pub repository: String,
    pub max_versions_per_component: Option<usize>,
    pub snapshot_only: Option<bool>,
}

pub async fn handle_run_cleanup(
    State(state): State<AppState>,
    Json(req): Json<RunCleanupRequest>,
) -> Response {
    let policy = CleanupPolicy {
        max_versions_per_component: req.max_versions_per_component,
        snapshot_only: req.snapshot_only.unwrap_or(false),
    };

    match CleanupService::run_cleanup(&state.runtime, state.blobstore.as_ref(), &req.repository, &policy).await {
        Ok(report) => Json(report).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_run_gc(State(state): State<AppState>) -> Response {
    match BlobStoreGcService::run_gc(&state.runtime, state.blobstore.as_ref()).await {
        Ok(report) => Json(report).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
