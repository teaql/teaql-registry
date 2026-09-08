use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::api::AppState;
use crate::engine::DockerEngine;
use crate::format::docker::{DockerTagList, DOCKER_MANIFEST_V2_MEDIA_TYPE};
use crate::services::{RepositoryService, ServiceLogService};

fn body_stream(body: Body) -> crate::blobstore::ByteStream {
    use futures_util::StreamExt;

    Box::pin(
        body.into_data_stream()
            .map(|chunk| chunk.map_err(std::io::Error::other)),
    )
}

#[derive(Debug, Deserialize)]
pub struct UploadQueryParams {
    pub digest: Option<String>,
}

pub async fn handle_v2_ping() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::HeaderName::from_static("docker-distribution-api-version"),
        HeaderValue::from_static("registry/2.0"),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    (StatusCode::OK, headers, "{}").into_response()
}

async fn resolve_docker_repo(
    runtime: &teaql_registry_core::ServiceRuntime,
    repo_name: Option<&str>,
) -> Result<teaql_registry_core::RepositoryConfiguration, (StatusCode, String)> {
    let name = repo_name.unwrap_or("docker-hosted");
    match RepositoryService::find_by_name(runtime, name).await {
        Ok(Some(r)) => Ok(r),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("Docker repository not found: {}", name),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 1. Tags list: GET /v2/<name>/tags/list
pub async fn handle_tags_list(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(name): Path<String>,
) -> Response {
    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::list_tags(&request.runtime, &repo, &name).await {
        Ok(tags) => {
            let list = DockerTagList { name, tags };
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            (StatusCode::OK, headers, Json(list)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// 2. Blobs Upload Init: POST /v2/<name>/blobs/uploads/
pub async fn handle_blob_upload_init(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(name): Path<String>,
    Query(query): Query<UploadQueryParams>,
    body: Body,
) -> Response {
    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    // If monolithic upload with digest
    if let Some(digest) = query.digest {
        match DockerEngine::store_blob_stream(
            &request.runtime,
            &repo,
            &state.blobstore,
            &name,
            &digest,
            body_stream(body),
        )
        .await
        {
            Ok((digest_res, _)) => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::HeaderName::from_static("docker-distribution-api-version"),
                    HeaderValue::from_static("registry/2.0"),
                );
                headers.insert(
                    header::LOCATION,
                    HeaderValue::from_str(&format!("/v2/{}/blobs/{}", name, digest_res))
                        .expect("valid ASCII header value"),
                );
                headers.insert(
                    header::HeaderName::from_static("docker-content-digest"),
                    HeaderValue::from_str(&digest_res).expect("valid ASCII header value"),
                );
                return (StatusCode::CREATED, headers).into_response();
            }
            Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    }

    let uuid = match DockerEngine::start_upload(&name).await {
        Ok(uuid) => uuid,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    };
    let size = match DockerEngine::append_chunk(&uuid, body_stream(body)).await {
        Ok(size) => size,
        Err(error) => {
            DockerEngine::cancel_upload(&uuid).await;
            return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
        }
    };
    let mut headers = HeaderMap::new();
    headers.insert(
        header::HeaderName::from_static("docker-distribution-api-version"),
        HeaderValue::from_static("registry/2.0"),
    );
    headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&format!("/v2/{}/blobs/uploads/{}", name, uuid))
            .expect("valid ASCII header value"),
    );
    headers.insert(
        header::HeaderName::from_static("docker-upload-uuid"),
        HeaderValue::from_str(&uuid).expect("valid ASCII header value"),
    );
    headers.insert(
        header::HeaderName::from_static("range"),
        HeaderValue::from_str(&format!("0-{}", size.saturating_sub(1)))
            .expect("valid ASCII header value"),
    );

    (StatusCode::ACCEPTED, headers).into_response()
}

// 3. Blobs Upload Chunk: PATCH /v2/<name>/blobs/uploads/<uuid>
pub async fn handle_blob_upload_chunk(
    State(_state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
    body: Body,
) -> Response {
    match DockerEngine::append_chunk(&uuid, body_stream(body)).await {
        Ok(len) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::LOCATION,
                HeaderValue::from_str(&format!("/v2/{}/blobs/uploads/{}", name, uuid))
                    .expect("valid ASCII header value"),
            );
            headers.insert(
                header::HeaderName::from_static("docker-upload-uuid"),
                HeaderValue::from_str(&uuid).expect("valid ASCII header value"),
            );
            headers.insert(
                header::HeaderName::from_static("range"),
                HeaderValue::from_str(&format!("0-{}", len.saturating_sub(1)))
                    .expect("valid ASCII header value"),
            );
            (StatusCode::ACCEPTED, headers).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

// 4. Blobs Upload Finish: PUT /v2/<name>/blobs/uploads/<uuid>?digest=sha256:...
pub async fn handle_blob_upload_finish(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((name, uuid)): Path<(String, String)>,
    Query(query): Query<UploadQueryParams>,
    _headers: HeaderMap,
    body: Body,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let upload_path = format!("/v2/{}/blobs/uploads/{}", name, uuid);

    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => {
            let log_type = if code == StatusCode::INTERNAL_SERVER_ERROR {
                "system"
            } else {
                "service"
            };
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                log_type,
                0,
                &username,
                &client_ip,
                "upload",
                &name,
                &upload_path,
                "docker",
                0,
                "error",
                &msg,
            )
            .await;
            return (code, msg).into_response();
        }
    };
    let repo_name = repo.name();

    let digest = query.digest.unwrap_or_default();
    match DockerEngine::finish_upload(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &uuid,
        &digest,
        Some(body_stream(body)),
    )
    .await
    {
        Ok((digest_res, body_size)) => {
            let artifact_path = format!("/v2/{}/blobs/{}", name, digest_res);
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &artifact_path,
                "docker",
                body_size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            res_headers.insert(
                header::LOCATION,
                HeaderValue::from_str(&format!("/v2/{}/blobs/{}", name, digest_res))
                    .expect("valid ASCII header value"),
            );
            res_headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest_res).expect("valid ASCII header value"),
            );
            (StatusCode::CREATED, res_headers).into_response()
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &upload_path,
                "docker",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

// 5. Blob HEAD / GET: /v2/<name>/blobs/<digest>
pub async fn handle_blob_get(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((name, digest)): Path<(String, String)>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let path = format!("/v2/{}/blobs/{}", name, digest);

    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => {
            let log_type = if code == StatusCode::INTERNAL_SERVER_ERROR {
                "system"
            } else {
                "service"
            };
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                log_type,
                0,
                &username,
                &client_ip,
                "download",
                &name,
                &path,
                "docker",
                0,
                "error",
                &msg,
            )
            .await;
            return (code, msg).into_response();
        }
    };
    let repo_name = repo.name();

    match DockerEngine::get_blob(&request.runtime, &repo, &state.blobstore, &name, &digest).await {
        Ok(Some((data, content_type))) => {
            let size = data.len() as i64;
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &path,
                "docker",
                size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            res_headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).expect("valid ASCII header value"),
            );
            res_headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            res_headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&data.len().to_string()).expect("valid ASCII header value"),
            );
            (StatusCode::OK, res_headers, data).into_response()
        }
        Ok(None) => {
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &path,
                "docker",
                0,
                "error",
                "Blob not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Blob not found").into_response()
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &path,
                "docker",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_blob_head(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((name, digest)): Path<(String, String)>,
) -> Response {
    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::has_blob(&request.runtime, &repo, &name, &digest).await {
        Ok(Some((size, content_type))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).expect("valid ASCII header value"),
            );
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&size.to_string()).expect("valid ASCII header value"),
            );
            (StatusCode::OK, headers).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// 6. Manifest PUT: PUT /v2/<name>/manifests/<reference>
pub async fn handle_manifest_put(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((name, reference)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let body_size = body.len() as i64;
    let path = format!("/v2/{}/manifests/{}", name, reference);

    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => {
            let log_type = if code == StatusCode::INTERNAL_SERVER_ERROR {
                "system"
            } else {
                "service"
            };
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                log_type,
                0,
                &username,
                &client_ip,
                "upload",
                &name,
                &path,
                "docker",
                body_size,
                "error",
                &msg,
            )
            .await;
            return (code, msg).into_response();
        }
    };
    let repo_name = repo.name();

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or(DOCKER_MANIFEST_V2_MEDIA_TYPE);

    match DockerEngine::put_manifest(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &reference,
        &body,
        content_type,
    )
    .await
    {
        Ok(digest) => {
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &path,
                "docker",
                body_size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            res_headers.insert(
                header::LOCATION,
                HeaderValue::from_str(&format!("/v2/{}/manifests/{}", name, reference))
                    .expect("valid ASCII header value"),
            );
            res_headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).expect("valid ASCII header value"),
            );
            (StatusCode::CREATED, res_headers).into_response()
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &path,
                "docker",
                body_size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

// 7. Manifest GET / HEAD: /v2/<name>/manifests/<reference>
pub async fn handle_manifest_get(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((name, reference)): Path<(String, String)>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let path = format!("/v2/{}/manifests/{}", name, reference);

    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => {
            let log_type = if code == StatusCode::INTERNAL_SERVER_ERROR {
                "system"
            } else {
                "service"
            };
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                log_type,
                0,
                &username,
                &client_ip,
                "download",
                &name,
                &path,
                "docker",
                0,
                "error",
                &msg,
            )
            .await;
            return (code, msg).into_response();
        }
    };
    let repo_name = repo.name();

    match DockerEngine::get_manifest(&request.runtime, &repo, &state.blobstore, &name, &reference)
        .await
    {
        Ok(Some((data, content_type, digest))) => {
            let size = data.len() as i64;
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &path,
                "docker",
                size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            res_headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).expect("valid ASCII header value"),
            );
            res_headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type)
                    .unwrap_or(HeaderValue::from_static(DOCKER_MANIFEST_V2_MEDIA_TYPE)),
            );
            res_headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&data.len().to_string()).expect("valid ASCII header value"),
            );
            (StatusCode::OK, res_headers, data).into_response()
        }
        Ok(None) => {
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &path,
                "docker",
                0,
                "error",
                "Manifest not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Manifest not found").into_response()
        }
        Err(e) => {
            ServiceLogService::log_event(
                &request.runtime,
                1_u64,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &path,
                "docker",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_manifest_head(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((name, reference)): Path<(String, String)>,
) -> Response {
    let repo = match resolve_docker_repo(&request.runtime, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::get_manifest(&request.runtime, &repo, &state.blobstore, &name, &reference)
        .await
    {
        Ok(Some((data, content_type, digest))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).expect("valid ASCII header value"),
            );
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type)
                    .unwrap_or(HeaderValue::from_static(DOCKER_MANIFEST_V2_MEDIA_TYPE)),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&data.len().to_string()).expect("valid ASCII header value"),
            );
            (StatusCode::OK, headers).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
