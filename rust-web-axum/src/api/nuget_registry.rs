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
use crate::services::ServiceLogService;

pub async fn handle_nuget_service_index(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Path(repo_name): Path<String>,
) -> Response {
    let base_url = format!(
        "{}/repository/{}",
        crate::api::public_base_url(&headers),
        repo_name
    );
    let index = NuGetEngine::get_service_index(&base_url).await;
    Json(index).into_response()
}

pub async fn handle_nuget_package_versions(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, id)): Path<(String, String)>,
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

    match NuGetEngine::get_package_versions(&request.runtime, &repo, &id).await {
        Ok(Some(versions)) => Json(versions).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Package not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_nuget_get_package(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, id, version, _file)): Path<(String, String, String, String)>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let path = format!("{}/{}", id, version);

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
                "nuget",
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
                "nuget",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match NuGetEngine::get_package_file(&request.runtime, &repo, &state.blobstore, &id, &version)
        .await
    {
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
                "nuget",
                size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/zip"),
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
                "nuget",
                0,
                "error",
                "Package not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Package not found").into_response()
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
                "nuget",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_nuget_push(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path(repo_name): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
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
                "",
                "nuget",
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
                "",
                "nuget",
                size,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let id = params
        .get("id")
        .map(|s| s.as_str())
        .unwrap_or("sample-package");
    let version = params.get("version").map(|s| s.as_str()).unwrap_or("1.0.0");
    let path = format!("{}/{}", id, version);

    match NuGetEngine::upload_package(
        &request.runtime,
        &repo,
        &state.blobstore,
        id,
        version,
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
                "nuget",
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
                "nuget",
                size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}
