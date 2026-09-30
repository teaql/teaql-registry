use crate::api::AppState;
use crate::engine::DartEngine;
use crate::services::RepositoryService;
use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

const PUB_V2: &str = "application/vnd.pub.v2+json";

fn json_response(status: StatusCode, value: serde_json::Value) -> Response {
    let mut response = (status, Json(value)).into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(PUB_V2));
    response
}

fn error(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    json_response(
        status,
        serde_json::json!({"error": {"code": code, "message": message.into()}}),
    )
}

async fn repository(
    request: &crate::security::RequestContext,
    name: &str,
) -> Result<teaql_registry_core::RepositoryConfiguration, Response> {
    match RepositoryService::find_by_name(&request.runtime, name).await {
        Ok(Some(repo)) if repo.recipe_name().starts_with("dart-") => Ok(repo),
        Ok(Some(_)) => Err(error(
            StatusCode::BAD_REQUEST,
            "invalid_repository",
            format!("Repository {name} is not a Dart Pub repository"),
        )),
        Ok(None) => Err(error(
            StatusCode::NOT_FOUND,
            "repository_not_found",
            format!("Repository not found: {name}"),
        )),
        Err(err) => Err(error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "repository_error",
            err.to_string(),
        )),
    }
}

pub async fn handle_dart_versions(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    headers: HeaderMap,
    Path((repo_name, package)): Path<(String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    let base_url = format!(
        "{}/repository/{repo_name}/dart",
        crate::api::public_base_url(&headers)
    );
    match DartEngine::package_versions(
        &request.runtime,
        &repo,
        &state.blobstore,
        &base_url,
        &package,
    )
    .await
    {
        Ok(Some(versions)) => json_response(StatusCode::OK, serde_json::json!(versions)),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "package_not_found",
            "package not found",
        ),
        Err(err) => error(StatusCode::BAD_REQUEST, "invalid_package", err.to_string()),
    }
}

pub async fn handle_dart_version(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    headers: HeaderMap,
    Path((repo_name, package, version)): Path<(String, String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    let base_url = format!(
        "{}/repository/{repo_name}/dart",
        crate::api::public_base_url(&headers)
    );
    match DartEngine::package_versions(
        &request.runtime,
        &repo,
        &state.blobstore,
        &base_url,
        &package,
    )
    .await
    {
        Ok(Some(package_versions)) => match package_versions
            .versions
            .into_iter()
            .find(|candidate| candidate.version == version)
        {
            Some(version) => json_response(StatusCode::OK, serde_json::json!(version)),
            None => error(
                StatusCode::NOT_FOUND,
                "version_not_found",
                "version not found",
            ),
        },
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "package_not_found",
            "package not found",
        ),
        Err(err) => error(StatusCode::BAD_REQUEST, "invalid_package", err.to_string()),
    }
}

pub async fn handle_dart_archive(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, package, archive)): Path<(String, String, String)>,
) -> Response {
    let Some(version) = archive.strip_suffix(".tar.gz") else {
        return error(
            StatusCode::NOT_FOUND,
            "archive_not_found",
            "archive not found",
        );
    };
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match DartEngine::archive(&request.runtime, &repo, &state.blobstore, &package, version).await {
        Ok(Some(bytes)) => {
            let mut response = bytes.into_response();
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, immutable"),
            );
            response
        }
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "archive_not_found",
            "archive not found",
        ),
        Err(err) => error(StatusCode::BAD_REQUEST, "invalid_archive", err.to_string()),
    }
}

pub async fn handle_dart_new_upload(
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    headers: HeaderMap,
    Path(repo_name): Path<String>,
) -> Response {
    if let Err(response) = repository(&request, &repo_name).await {
        return response;
    }
    let url = format!(
        "{}/repository/{repo_name}/dart/api/packages/versions/newUpload",
        crate::api::public_base_url(&headers)
    );
    json_response(
        StatusCode::OK,
        serde_json::json!({"url": url, "fields": {}}),
    )
}

pub async fn handle_dart_upload(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    headers: HeaderMap,
    Path(repo_name): Path<String>,
    mut multipart: Multipart,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    let mut archive = None;
    loop {
        match multipart.next_field().await {
            Ok(Some(field)) if field.name() == Some("file") => match field.bytes().await {
                Ok(bytes) => archive = Some(bytes),
                Err(err) => {
                    return error(StatusCode::BAD_REQUEST, "invalid_upload", err.to_string())
                }
            },
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(err) => return error(StatusCode::BAD_REQUEST, "invalid_upload", err.to_string()),
        }
    }
    let Some(archive) = archive else {
        return error(
            StatusCode::BAD_REQUEST,
            "missing_file",
            "multipart field 'file' is required",
        );
    };
    match DartEngine::publish(&request.runtime, &repo, &state.blobstore, &archive).await {
        Ok(package) => {
            let location = format!(
                "{}/repository/{repo_name}/dart/api/packages/versions/newUpload/complete/{}/{}",
                crate::api::public_base_url(&headers),
                package.name,
                package.version
            );
            let mut response = StatusCode::NO_CONTENT.into_response();
            if let Ok(value) = HeaderValue::from_str(&location) {
                response.headers_mut().insert(header::LOCATION, value);
            }
            response
        }
        Err(err) if err.to_string().contains("already exists") => {
            error(StatusCode::CONFLICT, "version_exists", err.to_string())
        }
        Err(err) => error(StatusCode::BAD_REQUEST, "invalid_package", err.to_string()),
    }
}

pub async fn handle_dart_finalize(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, package, version)): Path<(String, String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repo) => repo,
        Err(response) => return response,
    };
    match DartEngine::archive(
        &request.runtime,
        &repo,
        &state.blobstore,
        &package,
        &version,
    )
    .await
    {
        Ok(Some(_)) => json_response(
            StatusCode::OK,
            serde_json::json!({"success": {"message": format!("Successfully published {package} {version}")}}),
        ),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "upload_not_found",
            "uploaded package not found",
        ),
        Err(err) => error(StatusCode::BAD_REQUEST, "invalid_upload", err.to_string()),
    }
}
