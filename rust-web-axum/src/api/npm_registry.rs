use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::api::AppState;
use crate::engine::NpmEngine;
use crate::format::npm::NpmPackageDocument;
use crate::services::RepositoryService;

pub async fn handle_npm_get_package(
    State(state): State<AppState>,
    Path((repo_name, package_name)): Path<(String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let base_url = format!("http://localhost:8081/repository/{}", repo_name);
    match NpmEngine::get_package_document(&state.runtime, &repo, &package_name, &base_url).await {
        Ok(Some(doc)) => Json(doc).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Package not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_npm_publish(
    State(state): State<AppState>,
    Path((repo_name, _package_name)): Path<(String, String)>,
    Json(doc): Json<NpmPackageDocument>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match NpmEngine::publish_package(&state.runtime, &repo, &state.blobstore, &doc).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn handle_npm_get_tarball(
    State(state): State<AppState>,
    Path((repo_name, package_name, tarball)): Path<(String, String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let path = format!("/{}/-/{}", package_name, tarball);
    match NpmEngine::get_tarball(&state.runtime, &repo, &state.blobstore, &path).await {
        Ok(Some(data)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            (StatusCode::OK, headers, data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Tarball not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
