use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::services::TenantService;

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantResponseItem {
    pub id: String,
    pub name: String,
    pub code: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "blobRoot")]
    pub blob_root: Option<String>,
    #[serde(rename = "adminPassword")]
    pub admin_password: String,
}

pub async fn list_tenants(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
) -> Response {
    match TenantService::list_tenants(&request.runtime).await {
        Ok(tenants) => {
            let items: Vec<TenantResponseItem> = tenants
                .into_iter()
                .map(|t| TenantResponseItem {
                    id: t.id().to_string(),
                    name: t.name().to_string(),
                    code: t.code().to_string(),
                    description: t.description().to_string(),
                    enabled: t.enabled(),
                })
                .collect();
            Json(items).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_tenant(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Path(id): Path<u64>,
) -> Response {
    match TenantService::get_tenant(&request.runtime, id).await {
        Ok(Some(t)) => {
            let item = TenantResponseItem {
                id: t.id().to_string(),
                name: t.name().to_string(),
                code: t.code().to_string(),
                description: t.description().to_string(),
                enabled: t.enabled(),
            };
            Json(item).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Tenant not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_tenant(
    State(state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Json(payload): Json<CreateTenantRequest>,
) -> Response {
    if let Err(error) = crate::security::validate_password_strength(
        &payload.admin_password,
        &["admin", "teaql", "registry", &payload.name],
    ) {
        return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
    }
    let code = payload
        .code
        .unwrap_or_else(|| payload.name.to_lowercase().replace(' ', "-"));
    let description = payload.description.unwrap_or_default();
    match TenantService::create_tenant_with_platform(
        &request.runtime,
        1_u64,
        &payload.name,
        &code,
        &description,
    )
    .await
    {
        Ok(t) => {
            let blob_root = payload.blob_root.unwrap_or_else(|| {
                std::env::var("BLOB_STORAGE_PATH")
                    .unwrap_or_else(|_| "/var/lib/teaql-registry/blobs".to_string())
            });
            let pass_hash = crate::security::hash_password(&payload.admin_password);
            let admin_email = format!("admin@{}.local", code);
            let users = vec![(
                "admin",
                "Administrator",
                "User",
                admin_email.as_str(),
                pass_hash.as_str(),
            )];
            let tenant_runtime = match state.request_runtime(t.id(), &payload.name).await {
                Ok(runtime) => runtime,
                Err(error) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, error).into_response();
                }
            };
            if let Err(e) =
                TenantService::provision_tenant(&tenant_runtime, t.id(), &blob_root, &users).await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to provision tenant defaults: {}", e),
                )
                    .into_response();
            }

            let item = TenantResponseItem {
                id: t.id().to_string(),
                name: t.name().to_string(),
                code: t.code().to_string(),
                description: t.description().to_string(),
                enabled: t.enabled(),
            };
            (StatusCode::CREATED, Json(item)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
