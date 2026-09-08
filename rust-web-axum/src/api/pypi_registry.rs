use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};

use crate::api::AppState;
use crate::engine::PyPiEngine;
use crate::services::RepositoryService;
use crate::services::ServiceLogService;

pub async fn handle_pypi_simple_root(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&request.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                format!("Repository not found: {}", repo_name),
            )
                .into_response()
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match PyPiEngine::get_simple_root(&request.runtime, &repo).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_pypi_simple_package(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, project_name)): Path<(String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&request.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                format!("Repository not found: {}", repo_name),
            )
                .into_response()
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match PyPiEngine::get_simple_package(&request.runtime, &repo, &project_name).await {
        Ok(Some(html)) => Html(html).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Project not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_pypi_get_package_file(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, filename)): Path<(String, String)>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();

    let repo = match RepositoryService::find_by_name(&request.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &filename,
                "pypi",
                0,
                "error",
                "Repository not found",
            )
            .await;
            return (
                StatusCode::NOT_FOUND,
                format!("Repository not found: {}", repo_name),
            )
                .into_response();
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &filename,
                "pypi",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match PyPiEngine::get_package_file(&request.runtime, &repo, &state.blobstore, &filename).await {
        Ok(Some((data, content_type))) => {
            let size = data.len() as i64;
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &filename,
                "pypi",
                size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                res_headers.insert(header::CONTENT_TYPE, val);
            }
            (StatusCode::OK, res_headers, data).into_response()
        }
        Ok(None) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &filename,
                "pypi",
                0,
                "error",
                "Package file not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Package file not found").into_response()
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &filename,
                "pypi",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_pypi_upload(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path(repo_name): Path<String>,
    mut multipart: Multipart,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();

    let repo = match RepositoryService::find_by_name(&request.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                "",
                "pypi",
                0,
                "error",
                "Repository not found",
            )
            .await;
            return (
                StatusCode::NOT_FOUND,
                format!("Repository not found: {}", repo_name),
            )
                .into_response();
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                "",
                "pypi",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
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
        ServiceLogService::log_event(
            &request.runtime,
            request.tenant_id,
            "service",
            request.user_id as i64,
            &username,
            &client_ip,
            "upload",
            &repo_name,
            &filename,
            "pypi",
            0,
            "error",
            "Missing required upload fields",
        )
        .await;
        return (StatusCode::BAD_REQUEST, "Missing required upload fields").into_response();
    }

    if filename.is_empty() {
        filename = format!("{}-{}.whl", name, version);
    }

    let size = content.len() as i64;
    match PyPiEngine::upload_file(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &filename,
        &content,
    )
    .await
    {
        Ok(_) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &filename,
                "pypi",
                size,
                "success",
                "",
            )
            .await;
            (StatusCode::OK, "Upload successful").into_response()
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &filename,
                "pypi",
                size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}
