use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::api::AppState;
use crate::engine::ConanEngine;
use crate::security::TokenService;
use crate::services::RepositoryService;

type ConanPackageRevisionPath = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);
type ConanPackageFilePath = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

#[derive(Debug, Deserialize)]
pub struct ConanSearchQuery {
    pub q: Option<String>,
}

async fn repository(
    request: &crate::security::RequestContext,
    name: &str,
) -> Result<teaql_registry_core::RepositoryConfiguration, Response> {
    match RepositoryService::find_by_name(&request.runtime, name).await {
        Ok(Some(repo)) if repo.recipe_name().starts_with("conan-") => Ok(repo),
        Ok(Some(_)) => Err((
            StatusCode::BAD_REQUEST,
            format!("Repository {name} is not a Conan repository"),
        )
            .into_response()),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Repository not found").into_response()),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

pub async fn handle_conan_ping() -> Response {
    let mut response = StatusCode::OK.into_response();
    response.headers_mut().insert(
        HeaderNameExt::conan_capabilities(),
        HeaderValue::from_static("revisions"),
    );
    response
}

struct HeaderNameExt;

impl HeaderNameExt {
    fn conan_capabilities() -> header::HeaderName {
        header::HeaderName::from_static("x-conan-server-capabilities")
    }
}

pub async fn handle_conan_authenticate(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
) -> Response {
    if request.user_id == 0 {
        return (StatusCode::OK, "teaql-test-token").into_response();
    }
    let scopes = ["repository:read", "repository:write"]
        .into_iter()
        .filter(|scope| request.has_privilege(scope))
        .map(str::to_string)
        .collect();
    match TokenService::create_token(
        &request.runtime,
        request.tenant_id,
        request.user_id,
        &request.username,
        "Conan native client login",
        scopes,
        Some(30),
    )
    .await
    {
        Ok((token, _)) => (StatusCode::OK, token).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

pub async fn handle_conan_check_credentials() -> Response {
    (StatusCode::OK, "ok").into_response()
}

pub async fn handle_conan_search(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
    Query(query): Query<ConanSearchQuery>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::search_recipes(&request.runtime, &repo, query.q.as_deref()).await {
        Ok(results) => Json(serde_json::json!({ "results": results })).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_recipe_revisions(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel)): Path<(String, String, String, String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::recipe_revisions(&request.runtime, &repo, &name, &version, &user, &channel)
        .await
    {
        Ok(revisions) if revisions.revisions.is_empty() => StatusCode::NOT_FOUND.into_response(),
        Ok(revisions) => Json(revisions).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_recipe_latest(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel)): Path<(String, String, String, String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::recipe_revisions(&request.runtime, &repo, &name, &version, &user, &channel)
        .await
        .and_then(|revisions| ConanEngine::latest(&revisions))
    {
        Ok(revision) => Json(revision).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn handle_conan_recipe_snapshot(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel, revision)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::recipe_snapshot(
        &request.runtime,
        &repo,
        &name,
        &version,
        &user,
        &channel,
        &revision,
    )
    .await
    {
        Ok(Some(snapshot)) => Json(snapshot).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_recipe_file_get(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel, revision, file_path)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::recipe_file(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &user,
        &channel,
        &revision,
        &file_path,
    )
    .await
    {
        Ok(Some(bytes)) => bytes.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_recipe_file_put(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel, revision, file_path)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
    body: Bytes,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::put_recipe_file(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &user,
        &channel,
        &revision,
        &file_path,
        &body,
    )
    .await
    {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(err) if err.to_string().contains("already exists") => {
            (StatusCode::CONFLICT, err.to_string()).into_response()
        }
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_package_search(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel, recipe_revision)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::package_search(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &user,
        &channel,
        &recipe_revision,
    )
    .await
    {
        Ok(packages) => Json(packages).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_package_revisions(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel, recipe_revision, package_id)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::package_revisions(
        &request.runtime,
        &repo,
        &name,
        &version,
        &user,
        &channel,
        &recipe_revision,
        &package_id,
    )
    .await
    {
        Ok(revisions) if revisions.revisions.is_empty() => StatusCode::NOT_FOUND.into_response(),
        Ok(revisions) => Json(revisions).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_package_latest(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel, recipe_revision, package_id)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::package_revisions(
        &request.runtime,
        &repo,
        &name,
        &version,
        &user,
        &channel,
        &recipe_revision,
        &package_id,
    )
    .await
    .and_then(|revisions| ConanEngine::latest(&revisions))
    {
        Ok(revision) => Json(revision).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn handle_conan_package_snapshot(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, name, version, user, channel, recipe_revision, package_id, package_revision)): Path<ConanPackageRevisionPath>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::package_snapshot(
        &request.runtime,
        &repo,
        &name,
        &version,
        &user,
        &channel,
        &recipe_revision,
        &package_id,
        &package_revision,
    )
    .await
    {
        Ok(Some(snapshot)) => Json(snapshot).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_package_file_get(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((
        repo_name,
        name,
        version,
        user,
        channel,
        recipe_revision,
        package_id,
        package_revision,
        file_path,
    )): Path<ConanPackageFilePath>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::package_file(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &user,
        &channel,
        &recipe_revision,
        &package_id,
        &package_revision,
        &file_path,
    )
    .await
    {
        Ok(Some(bytes)) => bytes.into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn handle_conan_package_file_put(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((
        repo_name,
        name,
        version,
        user,
        channel,
        recipe_revision,
        package_id,
        package_revision,
        file_path,
    )): Path<ConanPackageFilePath>,
    body: Bytes,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match ConanEngine::put_package_file(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &user,
        &channel,
        &recipe_revision,
        &package_id,
        &package_revision,
        &file_path,
        &body,
    )
    .await
    {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(err) if err.to_string().contains("already exists") => {
            (StatusCode::CONFLICT, err.to_string()).into_response()
        }
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}
