use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{error, info};

use crate::blobstore::BlobStore;
use crate::engine::RepositoryDispatcher;
use crate::services::RepositoryService;
use crate::services::ServiceLogService;
use teaql_registry_core::{service_runtime_from_pool, DataServicePool, ServiceRuntime};

use crate::context::RegistryContextExt;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<ServiceRuntime>,
    pub blobstore: Arc<dyn BlobStore>,
    runtime_pool: Option<DataServicePool>,
    memory_mode: bool,
    enforce_authentication: bool,
    allow_anonymous_read: bool,
}

impl AppState {
    /// Construct a test/local state backed by one already configured context.
    pub fn new(runtime: Arc<ServiceRuntime>, blobstore: Arc<dyn BlobStore>) -> Self {
        Self {
            runtime,
            blobstore,
            runtime_pool: None,
            memory_mode: false,
            enforce_authentication: false,
            allow_anonymous_read: true,
        }
    }

    /// Construct production state. Every request receives a fresh UserContext
    /// while sharing the database pool and blob storage infrastructure.
    pub fn with_runtime_pool(
        runtime: Arc<ServiceRuntime>,
        runtime_pool: DataServicePool,
        blobstore: Arc<dyn BlobStore>,
        memory_mode: bool,
    ) -> Self {
        Self {
            runtime,
            blobstore,
            runtime_pool: Some(runtime_pool),
            memory_mode,
            enforce_authentication: true,
            allow_anonymous_read: std::env::var("ALLOW_ANONYMOUS_READ")
                .ok()
                .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true")),
        }
    }

    /// Integration-test constructor: retain request-scoped runtimes while the
    /// test itself supplies a trusted harness identity.
    pub fn with_runtime_pool_unsecured_for_tests(
        runtime: Arc<ServiceRuntime>,
        runtime_pool: DataServicePool,
        blobstore: Arc<dyn BlobStore>,
    ) -> Self {
        Self {
            runtime,
            blobstore,
            runtime_pool: Some(runtime_pool),
            memory_mode: false,
            enforce_authentication: false,
            allow_anonymous_read: true,
        }
    }

    pub fn enforces_authentication(&self) -> bool {
        self.enforce_authentication
    }

    pub fn allows_anonymous_read(&self) -> bool {
        self.allow_anonymous_read
    }

    pub async fn request_runtime(
        &self,
        tenant_id: u64,
        tenant_name: &str,
    ) -> Result<Arc<ServiceRuntime>, String> {
        let Some(pool) = &self.runtime_pool else {
            if tenant_id != self.runtime.tenant_id() {
                return Err("request-scoped tenant runtime requires a configured pool".to_string());
            }
            return Ok(self.runtime.clone());
        };

        let mut runtime = service_runtime_from_pool(pool.clone())
            .await
            .map_err(|error| format!("failed to create request runtime: {error}"))?;
        runtime.init_registry_context(self.blobstore.clone());
        runtime.set_memory_mode(self.memory_mode);
        runtime.set_tenant(tenant_id, tenant_name);
        Ok(Arc::new(runtime))
    }
}

#[axum::debug_handler]
pub async fn handle_get_content(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    _headers: HeaderMap,
    Path((repo_name, path)): Path<(String, String)>,
) -> Response {
    let repo_path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();

    info!("GET /repository/{}{}", repo_name, repo_path);

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
                &repo_path,
                "",
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
            error!("Error finding repository: {}", e);
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &repo_path,
                "",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match RepositoryDispatcher::get_stream(&request.runtime, &repo, &state.blobstore, &repo_path)
        .await
    {
        Ok(Some((stream, content_type, size))) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &repo_path,
                "",
                size,
                "success",
                "",
            )
            .await;
            let mut headers = HeaderMap::new();
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                headers.insert(http::header::CONTENT_TYPE, val);
            }
            if let Ok(val) = HeaderValue::from_str(&size.to_string()) {
                headers.insert(http::header::CONTENT_LENGTH, val);
            }
            (StatusCode::OK, headers, Body::from_stream(stream)).into_response()
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
                &repo_path,
                "",
                0,
                "error",
                "Artifact not found",
            )
            .await;
            (StatusCode::NOT_FOUND, "Artifact not found").into_response()
        }
        Err(e) => {
            error!("Error getting repository content: {}", e);
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "download",
                &repo_name,
                &repo_path,
                "",
                0,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn handle_head_content(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, path)): Path<(String, String)>,
) -> Response {
    let repo_path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };

    let repo = match RepositoryService::find_by_name(&request.runtime, &repo_name).await {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match RepositoryDispatcher::get_stream(&request.runtime, &repo, &state.blobstore, &repo_path)
        .await
    {
        Ok(Some((_stream, content_type, size))) => {
            let mut headers = HeaderMap::new();
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                headers.insert(http::header::CONTENT_TYPE, val);
            }
            if let Ok(val) = HeaderValue::from_str(&size.to_string()) {
                headers.insert(http::header::CONTENT_LENGTH, val);
            }
            (StatusCode::OK, headers).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn handle_put_content(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path((repo_name, path)): Path<(String, String)>,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let repo_path = if path.starts_with('/') {
        path
    } else {
        format!("/{}", path)
    };
    let username = request.username.clone();
    let client_ip = request.client_ip.clone();
    let declared_size = headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);

    info!(
        "PUT /repository/{}{} ({} bytes)",
        repo_name, repo_path, declared_size
    );

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
                &repo_path,
                "",
                declared_size,
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
                &repo_path,
                "",
                declared_size,
                "error",
                &e.to_string(),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let content_type = headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/octet-stream");

    use futures_util::StreamExt;
    let stream = body
        .into_data_stream()
        .map(|chunk| chunk.map_err(std::io::Error::other));
    match RepositoryDispatcher::put_stream(
        &request.runtime,
        &repo,
        &state.blobstore,
        &repo_path,
        Box::pin(stream),
        content_type,
    )
    .await
    {
        Ok(body_size) => {
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "service",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &repo_path,
                "",
                body_size,
                "success",
                "",
            )
            .await;
            (StatusCode::CREATED, "Artifact uploaded successfully").into_response()
        }
        Err(e) => {
            error!("Error uploading artifact: {}", e);
            ServiceLogService::log_event(
                &request.runtime,
                request.tenant_id,
                "system",
                request.user_id as i64,
                &username,
                &client_ip,
                "upload",
                &repo_name,
                &repo_path,
                "",
                declared_size,
                "error",
                &e.to_string(),
            )
            .await;
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}
