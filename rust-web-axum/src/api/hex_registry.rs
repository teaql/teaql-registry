use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sha2::{Digest, Sha256};

use crate::api::AppState;
use crate::engine::HexEngine;
use crate::services::RepositoryService;

async fn repository(
    request: &crate::security::RequestContext,
    name: &str,
) -> Result<teaql_registry_core::RepositoryConfiguration, Response> {
    match RepositoryService::find_by_name(&request.runtime, name).await {
        Ok(Some(repo)) if repo.recipe_name().starts_with("hex-") => Ok(repo),
        Ok(Some(_)) => Err((
            StatusCode::BAD_REQUEST,
            format!("Repository {name} is not a Hex repository"),
        )
            .into_response()),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Repository not found").into_response()),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

fn registry_response(headers: &HeaderMap, contents: Vec<u8>) -> Response {
    let etag = format!("\"{}\"", hex::encode(Sha256::digest(&contents)));
    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|candidate| candidate.trim() == etag))
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        if let Ok(value) = HeaderValue::from_str(&etag) {
            response.headers_mut().insert(header::ETAG, value);
        }
        return response;
    }
    let mut response = contents.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("max-age=60, public"),
    );
    if let Ok(value) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(header::ETAG, value);
    }
    response
}

pub async fn handle_hex_publish(
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
    match HexEngine::publish(&request.runtime, &repo, &state.blobstore, &body).await {
        Ok(package) => (
            StatusCode::CREATED,
            Json(HexEngine::publish_response(&repo, &package)),
        )
            .into_response(),
        Err(err) if err.to_string().contains("already exists") => {
            (StatusCode::CONFLICT, err.to_string()).into_response()
        }
        Err(err) => (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response(),
    }
}

pub async fn handle_hex_tarball(
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
    match HexEngine::tarball(&request.runtime, &repo, &state.blobstore, &filename).await {
        Ok(Some(bytes)) => bytes.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_hex_public_key(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
) -> Response {
    if let Err(response) = repository(&request, &repo_name).await {
        return response;
    }
    match HexEngine::public_key_pem().await {
        Ok(pem) => {
            let mut response = pem.into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/x-pem-file"),
            );
            response
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn handle_hex_names(
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
    match HexEngine::names(&request.runtime, &repo, &state.blobstore).await {
        Ok(contents) => registry_response(&headers, contents),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn handle_hex_versions(
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
    match HexEngine::versions(&request.runtime, &repo, &state.blobstore).await {
        Ok(contents) => registry_response(&headers, contents),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn handle_hex_package(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, package)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match HexEngine::package_registry(&request.runtime, &repo, &state.blobstore, &package).await {
        Ok(Some(contents)) => registry_response(&headers, contents),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}
