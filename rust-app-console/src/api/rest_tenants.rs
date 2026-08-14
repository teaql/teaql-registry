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
}

pub async fn list_tenants(State(state): State<AppState>) -> Response {
    match TenantService::list_tenants(&state.runtime).await {
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
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Response {
    match TenantService::get_tenant(&state.runtime, id).await {
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
    Json(payload): Json<CreateTenantRequest>,
) -> Response {
    let code = payload.code.unwrap_or_else(|| payload.name.to_lowercase().replace(' ', "-"));
    let description = payload.description.unwrap_or_default();
    match TenantService::create_tenant_with_platform(&state.runtime, 1_u64, &payload.name, &code, &description).await {
        Ok(t) => {
            let blob_root = payload.blob_root.unwrap_or_else(|| "/tmp/nexus_blobs".to_string());
            if let Err(e) = TenantService::provision_tenant(&state.runtime, t.id(), &blob_root).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to provision tenant defaults: {}", e)).into_response();
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
