use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::api::AppState;
use crate::engine::SwiftEngine;
use crate::format::swift::manifest_filename_for_swift_version;
use crate::services::{RepositoryService, ServiceLogService};

const CONTENT_VERSION: HeaderName = HeaderName::from_static("content-version");

fn versioned_json<T: Serialize>(status: StatusCode, value: T) -> Response {
    let mut response = (status, Json(value)).into_response();
    response
        .headers_mut()
        .insert(CONTENT_VERSION, HeaderValue::from_static("1"));
    response
}

fn problem(status: StatusCode, detail: impl Into<String>) -> Response {
    let body = serde_json::json!({
        "status": status.as_u16(),
        "title": status.canonical_reason().unwrap_or("Swift package registry error"),
        "detail": detail.into(),
    });
    let mut response = versioned_json(status, body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}

async fn repository(
    request: &crate::security::RequestContext,
    repo_name: &str,
) -> Result<teaql_registry_core::RepositoryConfiguration, Response> {
    match RepositoryService::find_by_name(&request.runtime, repo_name).await {
        Ok(Some(repository)) if repository.recipe_name().starts_with("swift-") => Ok(repository),
        Ok(Some(_)) => Err(problem(
            StatusCode::BAD_REQUEST,
            format!("Repository {repo_name} is not a Swift package repository"),
        )),
        Ok(None) => Err(problem(
            StatusCode::NOT_FOUND,
            format!("Repository not found: {repo_name}"),
        )),
        Err(error) => Err(problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )),
    }
}

fn package_name_without_json(value: &str) -> &str {
    value.strip_suffix(".json").unwrap_or(value)
}

pub async fn handle_swift_list_releases(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    headers: HeaderMap,
    Path((repo_name, scope, package)): Path<(String, String, String)>,
) -> Response {
    let package = package_name_without_json(&package);
    let repo = match repository(&request, &repo_name).await {
        Ok(repository) => repository,
        Err(response) => return response,
    };
    let base_url = format!(
        "{}/repository/{repo_name}/swift",
        crate::api::public_base_url(&headers)
    );
    match SwiftEngine::list_releases(&request.runtime, &repo, &base_url, &scope, package).await {
        Ok(Some(releases)) => {
            let latest = releases
                .releases
                .keys()
                .filter_map(|version| semver::Version::parse(version).ok())
                .max();
            let mut response = versioned_json(StatusCode::OK, releases);
            if let Some(latest) = latest {
                let link =
                    format!("<{base_url}/{scope}/{package}/{latest}>; rel=\"latest-version\"");
                if let Ok(value) = HeaderValue::from_str(&link) {
                    response.headers_mut().insert(header::LINK, value);
                }
            }
            response
        }
        Ok(None) => problem(StatusCode::NOT_FOUND, "package not found"),
        Err(error) => problem(StatusCode::BAD_REQUEST, error.to_string()),
    }
}

pub async fn handle_swift_release_resource(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, scope, package, resource)): Path<(String, String, String, String)>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repository) => repository,
        Err(response) => return response,
    };
    if let Some(version) = resource.strip_suffix(".zip") {
        return match SwiftEngine::get_source_archive(
            &request.runtime,
            &repo,
            &state.blobstore,
            &scope,
            &package,
            version,
        )
        .await
        {
            Ok(Some(archive)) => {
                let size = archive.len() as i64;
                let checksum = sha2::Sha256::digest(&archive);
                let digest = format!(
                    "sha-256={}",
                    base64::engine::general_purpose::STANDARD.encode(checksum)
                );
                let mut response = archive.into_response();
                *response.status_mut() = StatusCode::OK;
                let headers = response.headers_mut();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/zip"),
                );
                headers.insert(CONTENT_VERSION, HeaderValue::from_static("1"));
                headers.insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, immutable"),
                );
                headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
                if let Ok(value) = HeaderValue::from_str(&format!(
                    "attachment; filename=\"{package}-{version}.zip\""
                )) {
                    headers.insert(header::CONTENT_DISPOSITION, value);
                }
                if let Ok(value) = HeaderValue::from_str(&digest) {
                    headers.insert("digest", value);
                }
                if let Ok(Some(signature)) = SwiftEngine::get_archive_signature(
                    &request.runtime,
                    &repo,
                    &state.blobstore,
                    &scope,
                    &package,
                    version,
                )
                .await
                {
                    if let Ok(value) = HeaderValue::from_str(&signature.signature_format) {
                        headers.insert("x-swift-package-signature-format", value);
                    }
                    if let Ok(value) = HeaderValue::from_str(&signature.signature_base64_encoded) {
                        headers.insert("x-swift-package-signature", value);
                    }
                }
                ServiceLogService::log_event(
                    &request.runtime,
                    request.tenant_id,
                    "service",
                    request.user_id as i64,
                    &request.username,
                    &request.client_ip,
                    "download",
                    &repo_name,
                    &format!("{scope}/{package}/{version}.zip"),
                    "swift",
                    size,
                    "success",
                    "",
                )
                .await;
                response
            }
            Ok(None) => problem(StatusCode::NOT_FOUND, "package release not found"),
            Err(error) => problem(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        };
    }

    let version = resource.strip_suffix(".json").unwrap_or(&resource);
    match SwiftEngine::get_release_metadata(
        &request.runtime,
        &repo,
        &state.blobstore,
        &scope,
        &package,
        version,
    )
    .await
    {
        Ok(Some(metadata)) => versioned_json(StatusCode::OK, metadata),
        Ok(None) => problem(StatusCode::NOT_FOUND, "package release not found"),
        Err(error) => problem(StatusCode::BAD_REQUEST, error.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct ManifestQuery {
    #[serde(rename = "swift-version")]
    pub swift_version: Option<String>,
}

pub async fn handle_swift_manifest(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    headers: HeaderMap,
    Path((repo_name, scope, package, version)): Path<(String, String, String, String)>,
    Query(query): Query<ManifestQuery>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repository) => repository,
        Err(response) => return response,
    };
    let filename = query
        .swift_version
        .as_deref()
        .map(manifest_filename_for_swift_version)
        .unwrap_or_else(|| "Package.swift".to_string());
    match SwiftEngine::get_manifest(
        &request.runtime,
        &repo,
        &state.blobstore,
        &scope,
        &package,
        &version,
        &filename,
    )
    .await
    {
        Ok(Some(manifest)) => {
            let mut response = manifest.into_response();
            let response_headers = response.headers_mut();
            response_headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/x-swift"),
            );
            response_headers.insert(CONTENT_VERSION, HeaderValue::from_static("1"));
            response_headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, immutable"),
            );
            if let Ok(value) =
                HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
            {
                response_headers.insert(header::CONTENT_DISPOSITION, value);
            }
            if query.swift_version.is_none() {
                let base_url = format!(
                    "{}/repository/{repo_name}/swift/{scope}/{package}/{version}/Package.swift",
                    crate::api::public_base_url(&headers)
                );
                if let Ok(manifests) = SwiftEngine::list_manifests(
                    &request.runtime,
                    &repo,
                    &state.blobstore,
                    &scope,
                    &package,
                    &version,
                )
                .await
                {
                    let links = manifests
                        .into_iter()
                        .filter(|manifest| manifest.filename != "Package.swift")
                        .filter_map(|manifest| {
                            let tools_version = manifest.tools_version?;
                            let requested_version = manifest
                                .filename
                                .strip_prefix("Package@swift-")?
                                .strip_suffix(".swift")?;
                            Some(format!(
                                "<{base_url}?swift-version={requested_version}>; rel=\"alternate\"; filename=\"{}\"; swift-tools-version=\"{tools_version}\"",
                                manifest.filename
                            ))
                        })
                        .collect::<Vec<_>>();
                    if !links.is_empty() {
                        if let Ok(value) = HeaderValue::from_str(&links.join(", ")) {
                            response_headers.insert(header::LINK, value);
                        }
                    }
                }
            }
            response
        }
        Ok(None) if query.swift_version.is_some() => {
            let location = format!(
                "{}/repository/{repo_name}/swift/{scope}/{package}/{version}/Package.swift",
                crate::api::public_base_url(&headers)
            );
            let mut response = StatusCode::SEE_OTHER.into_response();
            response
                .headers_mut()
                .insert(CONTENT_VERSION, HeaderValue::from_static("1"));
            if let Ok(value) = HeaderValue::from_str(&location) {
                response.headers_mut().insert(header::LOCATION, value);
            }
            response
        }
        Ok(None) => problem(StatusCode::NOT_FOUND, "package manifest not found"),
        Err(error) => problem(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

pub async fn handle_swift_publish(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    headers: HeaderMap,
    Path((repo_name, scope, package, version)): Path<(String, String, String, String)>,
    mut multipart: Multipart,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repository) => repository,
        Err(response) => return response,
    };
    match SwiftEngine::release_exists(&request.runtime, &repo, &scope, &package, &version).await {
        Ok(true) => {
            return problem(
                StatusCode::CONFLICT,
                format!("a release with version {version} already exists"),
            )
        }
        Ok(false) => {}
        Err(error) => return problem(StatusCode::BAD_REQUEST, error.to_string()),
    }

    let mut source_archive = None;
    let mut archive_signature = None;
    let mut metadata = serde_json::json!({});
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                return problem(
                    StatusCode::BAD_REQUEST,
                    format!("invalid multipart body: {error}"),
                )
            }
        };
        match field.name() {
            Some("source-archive") => match field.bytes().await {
                Ok(bytes) => source_archive = Some(bytes),
                Err(error) => {
                    return problem(
                        StatusCode::BAD_REQUEST,
                        format!("invalid source archive: {error}"),
                    )
                }
            },
            Some("source-archive-signature") => match field.bytes().await {
                Ok(bytes) => archive_signature = Some(bytes),
                Err(error) => {
                    return problem(
                        StatusCode::BAD_REQUEST,
                        format!("invalid archive signature: {error}"),
                    )
                }
            },
            Some("metadata") => match field.bytes().await {
                Ok(bytes) => match serde_json::from_slice(&bytes) {
                    Ok(value @ serde_json::Value::Object(_)) => metadata = value,
                    Ok(_) => {
                        return problem(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            "release metadata must be a JSON object",
                        )
                    }
                    Err(error) => {
                        return problem(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            format!("invalid release metadata: {error}"),
                        )
                    }
                },
                Err(error) => {
                    return problem(
                        StatusCode::BAD_REQUEST,
                        format!("invalid release metadata: {error}"),
                    )
                }
            },
            _ => {}
        }
    }
    let Some(source_archive) = source_archive else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "missing source-archive multipart field",
        );
    };
    let signature_format = headers
        .get("x-swift-package-signature-format")
        .and_then(|value| value.to_str().ok());
    if archive_signature.is_some() && signature_format.is_none() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "signed archive is missing X-Swift-Package-Signature-Format",
        );
    }
    let signature = signature_format.zip(archive_signature.as_deref());
    let size = source_archive.len() as i64;
    match SwiftEngine::publish_release(
        &request.runtime,
        &repo,
        &state.blobstore,
        &scope,
        &package,
        &version,
        &source_archive,
        metadata,
        signature,
    )
    .await
    {
        Ok(()) => {
            let location = format!(
                "{}/repository/{repo_name}/swift/{scope}/{package}/{version}",
                crate::api::public_base_url(&headers)
            );
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &request.username,
                &request.client_ip,
                "upload",
                &repo_name,
                &format!("{scope}/{package}/{version}"),
                "swift",
                size,
                "success",
                "",
            )
            .await;
            let mut response = StatusCode::CREATED.into_response();
            response
                .headers_mut()
                .insert(CONTENT_VERSION, HeaderValue::from_static("1"));
            if let Ok(value) = HeaderValue::from_str(&location) {
                response.headers_mut().insert(header::LOCATION, value);
            }
            response
        }
        Err(error) if error.to_string().contains("already exists") => {
            problem(StatusCode::CONFLICT, error.to_string())
        }
        Err(error) if error.to_string().contains("Package.swift") => {
            problem(StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
        }
        Err(error) => problem(StatusCode::BAD_REQUEST, error.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct IdentifierQuery {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct IdentifierResponse {
    pub identifiers: Vec<String>,
}

pub async fn handle_swift_identifiers(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
    Query(query): Query<IdentifierQuery>,
) -> Response {
    let repo = match repository(&request, &repo_name).await {
        Ok(repository) => repository,
        Err(response) => return response,
    };
    match SwiftEngine::lookup_identifiers(&request.runtime, &repo, &state.blobstore, &query.url)
        .await
    {
        Ok(identifiers) => versioned_json(StatusCode::OK, IdentifierResponse { identifiers }),
        Err(error) => problem(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

pub async fn handle_swift_options() -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::ALLOW,
        HeaderValue::from_static("GET, HEAD, PUT, OPTIONS"),
    );
    response
}
