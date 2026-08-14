use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::api::AppState;
use crate::engine::NuGetEngine;
use crate::services::RepositoryService;

pub async fn handle_nuget_service_index(
    State(_state): State<AppState>,
    Path(repo_name): Path<String>,
) -> Response {
    let base_url = format!("http://localhost:8081/repository/{}", repo_name);
    let index = NuGetEngine::get_service_index(&base_url).await;
    Json(index).into_response()
}

pub async fn handle_nuget_package_versions(
    State(state): State<AppState>,
    Path((repo_name, id)): Path<(String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match NuGetEngine::get_package_versions(&state.runtime, &repo, &id).await {
        Ok(Some(versions)) => Json(versions).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Package not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_nuget_get_package(
    State(state): State<AppState>,
    Path((repo_name, id, version, _file)): Path<(String, String, String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match NuGetEngine::get_package_file(&state.runtime, &repo, &state.blobstore, &id, &version).await {
        Ok(Some(data)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/zip"),
            );
            (StatusCode::OK, headers, data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Package not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_nuget_push(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let id = params.get("id").map(|s| s.as_str()).unwrap_or("sample-package");
    let version = params.get("version").map(|s| s.as_str()).unwrap_or("1.0.0");

    match NuGetEngine::upload_package(&state.runtime, &repo, &state.blobstore, id, version, &body).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
