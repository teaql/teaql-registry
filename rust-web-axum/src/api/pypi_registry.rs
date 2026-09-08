use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};

use crate::api::AppState;
use crate::engine::PyPiEngine;
use crate::services::RepositoryService;
use crate::services::ServiceLogService;

async fn remove_temporary_upload(path: &Option<std::path::PathBuf>) {
    if let Some(path) = path {
        let _ = tokio::fs::remove_file(path).await;
    }
}

pub async fn handle_pypi_simple_root(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(repo_name): Path<String>,
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

    match PyPiEngine::get_simple_root(&request.runtime, &repo).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_pypi_simple_package(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, project_name)): Path<(String, String)>,
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

    match PyPiEngine::get_simple_package(&request.runtime, &repo, &project_name).await {
        Ok(Some(html)) => Html(html).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Project not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn handle_pypi_get_package_file(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, filename)): Path<(String, String)>,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();

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
                &filename,
                "pypi",
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
                &filename,
                "pypi",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match PyPiEngine::get_package_file(&request.runtime, &repo, &state.blobstore, &filename).await {
        Ok(Some((data, content_type))) => {
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
                &filename,
                "pypi",
                size,
                "success",
                "",
            )
            .await;
            let mut res_headers = HeaderMap::new();
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                res_headers.insert(header::CONTENT_TYPE, val);
            }
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
                &filename,
                "pypi",
                0,
                "error",
                "Package file not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Package file not found").into_response()
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
                &filename,
                "pypi",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_pypi_upload(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path(repo_name): Path<String>,
    mut multipart: Multipart,
) -> Response {
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();

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
                "pypi",
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
                "upload",
                &repo_name,
                "",
                "pypi",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let mut name = String::new();
    let mut version = String::new();
    let mut filename = String::new();
    let mut content_path = None;
    let mut content_size = 0_i64;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                remove_temporary_upload(&content_path).await;
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid PyPI multipart upload: {error}"),
                )
                    .into_response();
            }
        };
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "name" {
            name = match field.text().await {
                Ok(text) => text,
                Err(error) => {
                    remove_temporary_upload(&content_path).await;
                    return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
                }
            };
        } else if field_name == "version" {
            version = match field.text().await {
                Ok(text) => text,
                Err(error) => {
                    remove_temporary_upload(&content_path).await;
                    return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
                }
            };
        } else if field_name == "content" {
            if content_path.is_some() {
                remove_temporary_upload(&content_path).await;
                return (StatusCode::BAD_REQUEST, "Duplicate PyPI content field").into_response();
            }
            if let Some(fname) = field.file_name() {
                filename = fname.to_string();
            }
            let path =
                std::env::temp_dir().join(format!("teaql-pypi-{}.upload", uuid::Uuid::new_v4()));
            let mut file = match tokio::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
                .await
            {
                Ok(file) => file,
                Err(error) => {
                    remove_temporary_upload(&content_path).await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
                }
            };
            let mut field = field;
            loop {
                match field.chunk().await {
                    Ok(Some(chunk)) => {
                        use tokio::io::AsyncWriteExt;
                        if let Err(error) = file.write_all(&chunk).await {
                            let _ = tokio::fs::remove_file(&path).await;
                            remove_temporary_upload(&content_path).await;
                            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
                                .into_response();
                        }
                        content_size += chunk.len() as i64;
                    }
                    Ok(None) => break,
                    Err(error) => {
                        let _ = tokio::fs::remove_file(&path).await;
                        remove_temporary_upload(&content_path).await;
                        return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
                    }
                }
            }
            content_path = Some(path);
        }
    }

    if name.is_empty() || version.is_empty() || content_size == 0 || content_path.is_none() {
        remove_temporary_upload(&content_path).await;
        ServiceLogService::log_event(
            &request.runtime,
            request.tenant_id,
            "service",
            request.user_id as i64,
            &username,
            &client_ip,
            "upload",
            &repo_name,
            &filename,
            "pypi",
            0,
            "error",
            "Missing required upload fields",
        )
        .await;
        return (StatusCode::BAD_REQUEST, "Missing required upload fields").into_response();
    }

    if filename.is_empty() {
        filename = format!("{}-{}.whl", name, version);
    }

    let path = content_path.expect("validated temporary upload path");
    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(error) => {
            let _ = tokio::fs::remove_file(&path).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
        }
    };
    let stream = tokio_util::io::ReaderStream::new(tokio::io::BufReader::new(file));
    let result = PyPiEngine::upload_file_from_stream(
        &request.runtime,
        &repo,
        &state.blobstore,
        &name,
        &version,
        &filename,
        Box::pin(stream),
    )
    .await;
    let _ = tokio::fs::remove_file(path).await;

    match result {
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
                &filename,
                "pypi",
                content_size,
                "success",
                "",
            )
            .await;
            (StatusCode::OK, "Upload successful").into_response()
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
                &filename,
                "pypi",
                content_size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}
