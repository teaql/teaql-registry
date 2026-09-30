use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use base64::Engine as _;
use sha2::{Digest, Sha256};

use crate::api::AppState;
use crate::engine::RubyGemsEngine;
use crate::services::RepositoryService;

async fn repository(
    request: &crate::security::RequestContext,
    name: &str,
) -> Result<teaql_registry_core::RepositoryConfiguration, Response> {
    match RepositoryService::find_by_name(&request.runtime, name).await {
        Ok(Some(repo)) if repo.recipe_name().starts_with("rubygems-") => Ok(repo),
        Ok(Some(_)) => Err((
            StatusCode::BAD_REQUEST,
            format!("Repository {name} is not a RubyGems repository"),
        )
            .into_response()),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Repository not found").into_response()),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

fn compact_response(request_headers: &HeaderMap, contents: String) -> Response {
    let sha256 = Sha256::digest(contents.as_bytes());
    let etag = format!("\"{}\"", hex::encode(sha256));
    let repr_digest = format!(
        "sha-256=\"{}\"",
        base64::engine::general_purpose::STANDARD.encode(sha256)
    );
    let not_modified = request_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|candidate| candidate.trim() == etag));
    let range_start = request_headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("bytes="))
        .and_then(|value| value.strip_suffix('-'))
        .and_then(|value| value.parse::<usize>().ok());
    let full_len = contents.len();
    let (status, body, content_range) = if not_modified {
        (StatusCode::NOT_MODIFIED, String::new(), None)
    } else if let Some(start) = range_start.filter(|start| *start < full_len) {
        (
            StatusCode::PARTIAL_CONTENT,
            contents[start..].to_string(),
            Some(format!("bytes {start}-{}/{full_len}", full_len - 1)),
        )
    } else {
        (StatusCode::OK, contents, None)
    };
    let mut response = (status, body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("max-age=60, public"),
    );
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    if let Ok(value) = HeaderValue::from_str(&etag) {
        headers.insert(header::ETAG, value);
    }
    if let Ok(value) = HeaderValue::from_str(&repr_digest) {
        headers.insert("repr-digest", value);
    }
    if let Some(content_range) = content_range {
        if let Ok(value) = HeaderValue::from_str(&content_range) {
            headers.insert(header::CONTENT_RANGE, value);
        }
    }
    response
}

pub async fn handle_rubygems_push(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
    body: Bytes,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match RubyGemsEngine::publish(&request.runtime, &repo, &state.blobstore, &body).await {
        Ok(metadata) => (
            StatusCode::OK,
            format!(
                "Successfully registered gem: {} ({})",
                metadata.name, metadata.version
            ),
        )
            .into_response(),
        Err(err) if err.to_string().contains("already exists") => {
            (StatusCode::CONFLICT, err.to_string()).into_response()
        }
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_rubygems_download(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, filename)): Path<(String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match RubyGemsEngine::gem(&request.runtime, &repo, &state.blobstore, &filename).await {
        Ok(Some(bytes)) => {
            let mut response = bytes.into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            response
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_rubygems_versions(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match RubyGemsEngine::compact_versions(&request.runtime, &repo, &state.blobstore).await {
        Ok(contents) => compact_response(&headers, contents),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn handle_rubygems_info(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, gem_name)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match RubyGemsEngine::compact_info(&request.runtime, &repo, &state.blobstore, &gem_name).await {
        Ok(Some(contents)) => compact_response(&headers, contents),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn handle_rubygems_names(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match RubyGemsEngine::names(&request.runtime, &repo, &state.blobstore).await {
        Ok(contents) => compact_response(&headers, contents),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}
