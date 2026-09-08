use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::api::AppState;
use crate::engine::CargoEngine;
use crate::services::RepositoryService;
use crate::services::ServiceLogService;

pub async fn handle_cargo_config(
    State(_state): State<AppState>,
    Path(repo_name): Path<String>,
) -> Response {
    let repo_url = format!("http://localhost:8081/repository/{}", repo_name);
    let config = CargoEngine::get_config(&repo_url).await;
    Json(config).into_response()
}

pub async fn handle_cargo_download(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, crate_name, version)): Path<(String, String, String)>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let path = format!("{}/{}", crate_name, version);

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
                "cargo",
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
                "cargo",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match CargoEngine::get_crate_tarball(
        &request.runtime,
        &repo,
        &state.blobstore,
        &crate_name,
        &version,
    )
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
                "cargo",
                size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/gzip"),
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
                "cargo",
                0,
                "error",
                "Crate not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Crate not found").into_response()
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
                "cargo",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_cargo_sparse_index(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, index_path)): Path<(String, String)>,
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

    let crate_name = index_path.rsplit('/').next().unwrap_or(&index_path);
    match CargoEngine::get_sparse_index(&request.runtime, &repo, crate_name).await {
        Ok(Some(lines)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            (StatusCode::OK, headers, lines).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Crate index not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_cargo_publish(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path(repo_name): Path<String>,
    body: Bytes,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let body_size = body.len() as i64;

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
                "cargo",
                body_size,
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
                "cargo",
                body_size,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    // Cargo publish payload format:
    // 4 bytes: JSON length (little endian)
    // N bytes: JSON metadata (name, vers, deps, features, authors, description, etc.)
    // 4 bytes: Crate tarball length (little endian)
    // M bytes: .crate tarball
    if body.len() < 8 {
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
            "cargo",
            body_size,
            "error",
            "Invalid cargo publish payload",
        )
        .await;
        return (StatusCode::BAD_REQUEST, "Invalid cargo publish payload").into_response();
    }

    let json_len = u32::from_le_bytes([body[0], body[1], body[2], body[3]]) as usize;
    if body.len() < 4 + json_len + 4 {
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
            "cargo",
            body_size,
            "error",
            "Payload too short for json metadata",
        )
        .await;
        return (
            StatusCode::BAD_REQUEST,
            "Payload too short for json metadata",
        )
            .into_response();
    }

    let json_bytes = &body[4..4 + json_len];
    let meta: serde_json::Value = match serde_json::from_slice(json_bytes) {
        Ok(v) => v,
        Err(e) => {
            let err_msg = format!("Invalid json metadata: {}", e);
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
                "cargo",
                body_size,
                "error",
                &err_msg,
            )
            .await;
            return (StatusCode::BAD_REQUEST, err_msg).into_response();
        }
    };

    let name = meta
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let vers = meta.get("vers").and_then(|v| v.as_str()).unwrap_or("1.0.0");
    let path = format!("{}/{}", name, vers);

    let crate_offset = 4 + json_len + 4;
    let crate_bytes = &body[crate_offset..];
    let crate_size = crate_bytes.len() as i64;

    match CargoEngine::upload_crate(
        &request.runtime,
        &repo,
        &state.blobstore,
        name,
        vers,
        crate_bytes,
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
                "cargo",
                crate_size,
                "success",
                "",
            )
            .await;
            Json(serde_json::json!({"warnings": {"invalid_categories": [], "invalid_badges": [], "other": []}})).into_response()
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
                "cargo",
                crate_size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}
