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
use crate::services::ServiceLogService;

pub async fn handle_npm_get_package(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, package_name)): Path<(String, String)>,
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

    let base_url = format!("http://localhost:8081/repository/{}", repo_name);
    match NpmEngine::get_package_document(&request.runtime, &repo, &package_name, &base_url).await {
        Ok(Some(doc)) => Json(doc).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Package not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_npm_publish(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, package_name)): Path<(String, String)>,
    _headers: HeaderMap,
    Json(doc): Json<NpmPackageDocument>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let path = format!("/{}", package_name);
    let size: i64 = doc
        .attachments
        .values()
        .map(|a| a.length.unwrap_or(a.data.len()) as i64)
        .sum();

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
                &path,
                "npm",
                size,
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
                &path,
                "npm",
                size,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match NpmEngine::publish_package(&request.runtime, &repo, &state.blobstore, &doc).await {
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
                &path,
                "npm",
                size,
                "success",
                "",
            )
            .await;
            (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response()
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &path,
                "npm",
                size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

pub async fn handle_npm_get_tarball(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, package_name, tarball)): Path<(String, String, String)>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let path = format!("/{}/-/{}", package_name, tarball);

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
                &path,
                "npm",
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
                &path,
                "npm",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match NpmEngine::get_tarball(&request.runtime, &repo, &state.blobstore, &path).await {
        Ok(Some(data)) => {
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
                &path,
                "npm",
                size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
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
                &path,
                "npm",
                0,
                "error",
                "Tarball not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Tarball not found").into_response()
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
                &path,
                "npm",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
