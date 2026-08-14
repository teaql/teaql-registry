use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::api::AppState;
use crate::engine::DockerEngine;
use crate::format::docker::{DockerTagList, DOCKER_MANIFEST_V2_MEDIA_TYPE};
use crate::services::RepositoryService;

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

async fn resolve_docker_repo(state: &AppState, repo_name: Option<&str>) -> Result<teaql_registry_core::RepositoryConfiguration, (StatusCode, String)> {
    let name = repo_name.unwrap_or("docker-hosted");
    match RepositoryService::find_by_name(&state.runtime, name).await {
        Ok(Some(r)) => Ok(r),
        Ok(None) => Err((StatusCode::NOT_FOUND, format!("Docker repository not found: {}", name))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 1. Tags list: GET /v2/<name>/tags/list
pub async fn handle_tags_list(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::list_tags(&state.runtime, &repo, &name).await {
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
    Path(name): Path<String>,
    Query(query): Query<UploadQueryParams>,
    body: Bytes,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    // If monolithic upload with digest
    if let Some(digest) = query.digest {
        let upload_uuid = DockerEngine::start_upload(&name);
        match DockerEngine::finish_upload(
            &state.runtime,
            &repo,
            &state.blobstore,
            &name,
            &upload_uuid,
            &digest,
            Some(&body),
        )
        .await
        {
            Ok(digest_res) => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::HeaderName::from_static("docker-distribution-api-version"),
                    HeaderValue::from_static("registry/2.0"),
                );
                headers.insert(
                    header::LOCATION,
                    HeaderValue::from_str(&format!("/v2/{}/blobs/{}", name, digest_res)).unwrap(),
                );
                headers.insert(
                    header::HeaderName::from_static("docker-content-digest"),
                    HeaderValue::from_str(&digest_res).unwrap(),
                );
                return (StatusCode::CREATED, headers).into_response();
            }
            Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    }

    let uuid = DockerEngine::start_upload(&name);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::HeaderName::from_static("docker-distribution-api-version"),
        HeaderValue::from_static("registry/2.0"),
    );
    headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&format!("/v2/{}/blobs/uploads/{}", name, uuid)).unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("docker-upload-uuid"),
        HeaderValue::from_str(&uuid).unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("range"),
        HeaderValue::from_static("0-0"),
    );

    (StatusCode::ACCEPTED, headers).into_response()
}

// 3. Blobs Upload Chunk: PATCH /v2/<name>/blobs/uploads/<uuid>
pub async fn handle_blob_upload_chunk(
    State(_state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
    body: Bytes,
) -> Response {
    match DockerEngine::append_chunk(&uuid, &body) {
        Ok(len) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::LOCATION,
                HeaderValue::from_str(&format!("/v2/{}/blobs/uploads/{}", name, uuid)).unwrap(),
            );
            headers.insert(
                header::HeaderName::from_static("docker-upload-uuid"),
                HeaderValue::from_str(&uuid).unwrap(),
            );
            headers.insert(
                header::HeaderName::from_static("range"),
                HeaderValue::from_str(&format!("0-{}", len.saturating_sub(1))).unwrap(),
            );
            (StatusCode::ACCEPTED, headers).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

// 4. Blobs Upload Finish: PUT /v2/<name>/blobs/uploads/<uuid>?digest=sha256:...
pub async fn handle_blob_upload_finish(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
    Query(query): Query<UploadQueryParams>,
    body: Bytes,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    let digest = query.digest.unwrap_or_default();
    let extra = if body.is_empty() { None } else { Some(body.as_ref()) };

    match DockerEngine::finish_upload(
        &state.runtime,
        &repo,
        &state.blobstore,
        &name,
        &uuid,
        &digest,
        extra,
    )
    .await
    {
        Ok(digest_res) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::LOCATION,
                HeaderValue::from_str(&format!("/v2/{}/blobs/{}", name, digest_res)).unwrap(),
            );
            headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest_res).unwrap(),
            );
            (StatusCode::CREATED, headers).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

// 5. Blob HEAD / GET: /v2/<name>/blobs/<digest>
pub async fn handle_blob_get(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::get_blob(&state.runtime, &repo, &state.blobstore, &name, &digest).await {
        Ok(Some((data, content_type))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).unwrap(),
            );
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&data.len().to_string()).unwrap(),
            );
            (StatusCode::OK, headers, data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Blob not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_blob_head(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::has_blob(&state.runtime, &repo, &name, &digest).await {
        Ok(Some((size, content_type))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).unwrap(),
            );
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&size.to_string()).unwrap(),
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
    Path((name, reference)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or(DOCKER_MANIFEST_V2_MEDIA_TYPE);

    match DockerEngine::put_manifest(
        &state.runtime,
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
            let mut res_headers = HeaderMap::new();
            res_headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            res_headers.insert(
                header::LOCATION,
                HeaderValue::from_str(&format!("/v2/{}/manifests/{}", name, reference)).unwrap(),
            );
            res_headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).unwrap(),
            );
            (StatusCode::CREATED, res_headers).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

// 7. Manifest GET / HEAD: /v2/<name>/manifests/<reference>
pub async fn handle_manifest_get(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::get_manifest(&state.runtime, &repo, &state.blobstore, &name, &reference).await {
        Ok(Some((data, content_type, digest))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).unwrap(),
            );
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static(DOCKER_MANIFEST_V2_MEDIA_TYPE)),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&data.len().to_string()).unwrap(),
            );
            (StatusCode::OK, headers, data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Manifest not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_manifest_head(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Response {
    let repo = match resolve_docker_repo(&state, None).await {
        Ok(r) => r,
        Err((code, msg)) => return (code, msg).into_response(),
    };

    match DockerEngine::get_manifest(&state.runtime, &repo, &state.blobstore, &name, &reference).await {
        Ok(Some((data, content_type, digest))) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::HeaderName::from_static("docker-distribution-api-version"),
                HeaderValue::from_static("registry/2.0"),
            );
            headers.insert(
                header::HeaderName::from_static("docker-content-digest"),
                HeaderValue::from_str(&digest).unwrap(),
            );
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type).unwrap_or(HeaderValue::from_static(DOCKER_MANIFEST_V2_MEDIA_TYPE)),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&data.len().to_string()).unwrap(),
            );
            (StatusCode::OK, headers).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
