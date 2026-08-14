use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::api::AppState;
use crate::engine::GoModEngine;
use crate::format::gomod::parse_gomod_path;
use crate::services::RepositoryService;

pub async fn handle_gomod_get(
    State(state): State<AppState>,
    Path((repo_name, path)): Path<(String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let parsed = match parse_gomod_path(&path) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "Invalid Go module path").into_response(),
    };

    let (module, version, ext) = parsed;

    if ext == "list" {
        match GoModEngine::list_versions(&state.runtime, &repo, &module).await {
            Ok(list) => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                );
                (StatusCode::OK, headers, list).into_response()
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else if ext == "info" {
        match GoModEngine::get_version_info(&state.runtime, &repo, &module, &version).await {
            Ok(Some(info)) => Json(info).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, "Version info not found").into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        match GoModEngine::get_file(&state.runtime, &repo, &state.blobstore, &path).await {
            Ok(Some((data, ct))) => {
                let mut headers = HeaderMap::new();
                if let Ok(val) = HeaderValue::from_str(&ct) {
                    headers.insert(header::CONTENT_TYPE, val);
                }
                (StatusCode::OK, headers, data).into_response()
            }
            Ok(None) => (StatusCode::NOT_FOUND, "Artifact not found").into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

pub async fn handle_gomod_put(
    State(state): State<AppState>,
    Path((repo_name, path)): Path<(String, String)>,
    body: Bytes,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let (module, version, ext) = match parse_gomod_path(&path) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "Invalid Go module path").into_response(),
    };

    match GoModEngine::upload_artifact(
        &state.runtime,
        &repo,
        &state.blobstore,
        &module,
        &version,
        &ext,
        &body,
    )
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
