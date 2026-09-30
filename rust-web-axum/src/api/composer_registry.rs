use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::api::{public_base_url, AppState};
use crate::engine::ComposerEngine;
use crate::services::RepositoryService;

async fn repository(
    request: &crate::security::RequestContext,
    name: &str,
) -> Result<teaql_registry_core::RepositoryConfiguration, Response> {
    match RepositoryService::find_by_name(&request.runtime, name).await {
        Ok(Some(repo)) if repo.recipe_name().starts_with("composer-") => Ok(repo),
        Ok(Some(_)) => Err((
            StatusCode::BAD_REQUEST,
            format!("Repository {name} is not a Composer repository"),
        )
            .into_response()),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Repository not found").into_response()),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

fn json_response(value: serde_json::Value) -> Response {
    let mut response = Json(value).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("max-age=60, public"),
    );
    response
}

pub async fn handle_composer_root(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ComposerEngine::root_metadata(&request.runtime, &repo).await {
        Ok(value) => json_response(value),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn handle_composer_p2(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, package_path)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    let Some(path) = package_path.strip_suffix(".json") else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let (package, development_only) = path
        .strip_suffix("~dev")
        .map_or((path, false), |package| (package, true));
    match ComposerEngine::p2_metadata(
        &request.runtime,
        &repo,
        &state.blobstore,
        package,
        &public_base_url(&headers),
        development_only,
    )
    .await
    {
        Ok(Some(value)) => json_response(value),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_composer_dist(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, vendor, package, filename)): Path<(String, String, String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    let Some(version) = filename.strip_suffix(".zip") else {
        return (StatusCode::BAD_REQUEST, "Composer dist must end in .zip").into_response();
    };
    let name = format!("{vendor}/{package}");
    match ComposerEngine::archive(&request.runtime, &repo, &state.blobstore, &name, version).await {
        Ok(Some(bytes)) => {
            let mut response = bytes.into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/zip"),
            );
            response
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_composer_publish(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, vendor, package, filename)): Path<(String, String, String, String)>,
    body: Bytes,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    let Some(version) = filename.strip_suffix(".zip") else {
        return (StatusCode::BAD_REQUEST, "Composer dist must end in .zip").into_response();
    };
    let name = format!("{vendor}/{package}");
    match ComposerEngine::publish(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        version,
        &body,
    )
    .await
    {
        Ok(stored) => (StatusCode::CREATED, Json(stored)).into_response(),
        Err(err) if err.to_string().contains("already exists") => {
            (StatusCode::CONFLICT, err.to_string()).into_response()
        }
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}
