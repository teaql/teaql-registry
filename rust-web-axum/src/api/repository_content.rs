use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{error, info};

use crate::blobstore::BlobStore;
use crate::engine::RepositoryDispatcher;
use crate::services::RepositoryService;
use teaql_registry_core::ServiceRuntime;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<ServiceRuntime>,
    pub blobstore: Arc<dyn BlobStore>,
}

#[axum::debug_handler]
pub async fn handle_get_content(
    State(state): State<AppState>,
    Path((repo_name, path)): Path<(String, String)>,
) -> Response {
    let repo_path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };

    info!("GET /repository/{}{}", repo_name, repo_path);

    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => {
            error!("Error finding repository: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match RepositoryDispatcher::get(&state.runtime, &repo, &state.blobstore, &repo_path).await {
        Ok(Some((bytes, content_type))) => {
            let mut headers = HeaderMap::new();
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                headers.insert(http::header::CONTENT_TYPE, val);
            }
            (StatusCode::OK, headers, bytes).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Artifact not found").into_response(),
        Err(e) => {
            error!("Error getting repository content: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_head_content(
    State(state): State<AppState>,
    Path((repo_name, path)): Path<(String, String)>,
) -> Response {
    let repo_path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };

    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match RepositoryDispatcher::get(&state.runtime, &repo, &state.blobstore, &repo_path).await {
        Ok(Some((_, content_type))) => {
            let mut headers = HeaderMap::new();
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                headers.insert(http::header::CONTENT_TYPE, val);
            }
            (StatusCode::OK, headers).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn handle_put_content(
    State(state): State<AppState>,
    Path((repo_name, path)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo_path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };

    info!("PUT /repository/{}{} ({} bytes)", repo_name, repo_path, body.len());

    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let content_type = headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream");

    match RepositoryDispatcher::put(
        &state.runtime,
        &repo,
        &state.blobstore,
        &repo_path,
        &body,
        content_type,
    )
    .await
    {
        Ok(_) => (StatusCode::CREATED, "Artifact uploaded successfully").into_response(),
        Err(e) => {
            error!("Error uploading artifact: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}
