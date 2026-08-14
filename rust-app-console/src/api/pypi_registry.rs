use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};

use crate::api::AppState;
use crate::engine::PyPiEngine;
use crate::services::RepositoryService;

pub async fn handle_pypi_simple_root(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match PyPiEngine::get_simple_root(&state.runtime, &repo).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_pypi_simple_package(
    State(state): State<AppState>,
    Path((repo_name, project_name)): Path<(String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match PyPiEngine::get_simple_package(&state.runtime, &repo, &project_name).await {
        Ok(Some(html)) => Html(html).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Project not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_pypi_get_package_file(
    State(state): State<AppState>,
    Path((repo_name, filename)): Path<(String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match PyPiEngine::get_package_file(&state.runtime, &repo, &state.blobstore, &filename).await {
        Ok(Some((data, content_type))) => {
            let mut headers = HeaderMap::new();
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                headers.insert(header::CONTENT_TYPE, val);
            }
            (StatusCode::OK, headers, data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Package file not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_pypi_upload(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    mut multipart: Multipart,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut name = String::new();
    let mut version = String::new();
    let mut filename = String::new();
    let mut content = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "name" {
            if let Ok(text) = field.text().await {
                name = text;
            }
        } else if field_name == "version" {
            if let Ok(text) = field.text().await {
                version = text;
            }
        } else if field_name == "content" {
            if let Some(fname) = field.file_name() {
                filename = fname.to_string();
            }
            if let Ok(bytes) = field.bytes().await {
                content = bytes.to_vec();
            }
        }
    }

    if name.is_empty() || version.is_empty() || content.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing required upload fields").into_response();
    }

    if filename.is_empty() {
        filename = format!("{}-{}.whl", name, version);
    }

    match PyPiEngine::upload_file(
        &state.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &filename,
        &content,
    )
    .await
    {
        Ok(_) => (StatusCode::OK, "Upload successful").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
