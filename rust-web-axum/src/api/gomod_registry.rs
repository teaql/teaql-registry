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
use crate::services::ServiceLogService;

pub async fn handle_gomod_get(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, path)): Path<(String, String)>,
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
                &path,
                "gomod",
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
                "gomod",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let parsed = match parse_gomod_path(&path) {
        Some(p) => p,
        None => {
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
                "gomod",
                0,
                "error",
                "Invalid Go module path",
            )
            .await;
            return (StatusCode::BAD_REQUEST, "Invalid Go module path").into_response();
        }
    };

    let (module, version, ext) = parsed;

    if ext == "list" {
        match GoModEngine::list_versions(&request.runtime, &repo, &module).await {
            Ok(list) => {
                let size = list.len() as i64;
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
                    "gomod",
                    size,
                    "success",
                    "",
                )
                .await;
                let mut res_headers = HeaderMap::new();
                res_headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                );
                (StatusCode::OK, res_headers, list).into_response()
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
                    "gomod",
                    0,
                    "error",
                    &e.to_string(),
                )
                .await;
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    } else if ext == "info" {
        match GoModEngine::get_version_info(&request.runtime, &repo, &module, &version).await {
            Ok(Some(info)) => {
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
                    "gomod",
                    0,
                    "success",
                    "",
                )
                .await;
                Json(info).into_response()
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
                    "gomod",
                    0,
                    "error",
                    "Version info not found",
                )
                .await;
                (StatusCode::NOT_FOUND, "Version info not found").into_response()
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
                    "gomod",
                    0,
                    "error",
                    &e.to_string(),
                )
                .await;
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    } else {
        match GoModEngine::get_file(&request.runtime, &repo, &state.blobstore, &path).await {
            Ok(Some((data, ct))) => {
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
                    "gomod",
                    size,
                    "success",
                    "",
                )
                .await;
                let mut res_headers = HeaderMap::new();
                if let Ok(val) = HeaderValue::from_str(&ct) {
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
                    &path,
                    "gomod",
                    0,
                    "error",
                    "Artifact not found",
                )
                .await;
                (StatusCode::NOT_FOUND, "Artifact not found").into_response()
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
                    "gomod",
                    0,
                    "error",
                    &e.to_string(),
                )
                .await;
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

pub async fn handle_gomod_put(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, path)): Path<(String, String)>,
    body: Bytes,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let size = body.len() as i64;

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
                "gomod",
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
                "gomod",
                size,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let (module, version, ext) = match parse_gomod_path(&path) {
        Some(p) => p,
        None => {
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
                "gomod",
                size,
                "error",
                "Invalid Go module path",
            )
            .await;
            return (StatusCode::BAD_REQUEST, "Invalid Go module path").into_response();
        }
    };

    match GoModEngine::upload_artifact(
        &request.runtime,
        &repo,
        &state.blobstore,
        &module,
        &version,
        &ext,
        &body,
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
                &path,
                "gomod",
                size,
                "success",
                "",
            )
            .await;
            StatusCode::CREATED.into_response()
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
                "gomod",
                size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}
