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
    Path((repo_name, crate_name, version)): Path<(String, String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match CargoEngine::get_crate_tarball(&state.runtime, &repo, &state.blobstore, &crate_name, &version).await {
        Ok(Some(data)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/gzip"),
            );
            (StatusCode::OK, headers, data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Crate not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_cargo_sparse_index(
    State(state): State<AppState>,
    Path((repo_name, index_path)): Path<(String, String)>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let crate_name = index_path.rsplit('/').next().unwrap_or(&index_path);
    match CargoEngine::get_sparse_index(&state.runtime, &repo, crate_name).await {
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
    Path(repo_name): Path<String>,
    body: Bytes,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Repository not found: {}", repo_name)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Cargo publish payload format:
    // 4 bytes: JSON length (little endian)
    // N bytes: JSON metadata (name, vers, deps, features, authors, description, etc.)
    // 4 bytes: Crate tarball length (little endian)
    // M bytes: .crate tarball
    if body.len() < 8 {
        return (StatusCode::BAD_REQUEST, "Invalid cargo publish payload").into_response();
    }

    let json_len = u32::from_le_bytes([body[0], body[1], body[2], body[3]]) as usize;
    if body.len() < 4 + json_len + 4 {
        return (StatusCode::BAD_REQUEST, "Payload too short for json metadata").into_response();
    }

    let json_bytes = &body[4..4 + json_len];
    let meta: serde_json::Value = match serde_json::from_slice(json_bytes) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid json metadata: {}", e)).into_response(),
    };

    let name = meta.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
    let vers = meta.get("vers").and_then(|v| v.as_str()).unwrap_or("1.0.0");

    let crate_offset = 4 + json_len + 4;
    let crate_bytes = &body[crate_offset..];

    match CargoEngine::upload_crate(&state.runtime, &repo, &state.blobstore, name, vers, crate_bytes).await {
        Ok(_) => Json(serde_json::json!({"warnings": {"invalid_categories": [], "invalid_badges": [], "other": []}})).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
