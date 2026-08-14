use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

use super::repository_content::AppState;
use crate::services::RepositoryService;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryItemXO {
    pub name: String,
    pub format: String,
    pub r#type: String,
    pub url: String,
    pub online: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub online: Option<bool>,
    pub storage: Option<StorageConfigXO>,
    pub proxy: Option<ProxyConfigXO>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfigXO {
    pub blob_store_name: Option<String>,
    pub write_policy: Option<String>,
    pub strict_content_type_validation: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfigXO {
    pub remote_url: Option<String>,
    pub content_max_age: Option<i64>,
    pub metadata_max_age: Option<i64>,
}

pub async fn list_repositories(State(state): State<AppState>) -> Response {
    match RepositoryService::list(&state.runtime).await {
        Ok(repos) => {
            let items: Vec<RepositoryItemXO> = repos
                .into_iter()
                .map(|r| {
                    let format = if r.recipe_name().contains("maven") {
                        "maven2"
                    } else {
                        "raw"
                    };
                    let r_type = if r.recipe_name().ends_with("hosted") {
                        "hosted"
                    } else if r.recipe_name().ends_with("proxy") {
                        "proxy"
                    } else {
                        "group"
                    };
                    RepositoryItemXO {
                        name: r.name().to_string(),
                        format: format.to_string(),
                        r#type: r_type.to_string(),
                        url: format!("/repository/{}", r.name()),
                        online: r.online(),
                    }
                })
                .collect();
            Json(items).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_repository(
    State(state): State<AppState>,
    Path((_format, _type, name)): Path<(String, String, String)>,
) -> Response {
    match RepositoryService::find_by_name(&state.runtime, &name).await {
        Ok(Some(r)) => {
            let format = if r.recipe_name().contains("maven") {
                "maven2"
            } else {
                "raw"
            };
            let r_type = if r.recipe_name().ends_with("hosted") {
                "hosted"
            } else if r.recipe_name().ends_with("proxy") {
                "proxy"
            } else {
                "group"
            };
            Json(RepositoryItemXO {
                name: r.name().to_string(),
                format: format.to_string(),
                r#type: r_type.to_string(),
                url: format!("/repository/{}", r.name()),
                online: r.online(),
            })
            .into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_repository(
    State(state): State<AppState>,
    Path((format, r_type)): Path<(String, String)>,
    Json(payload): Json<CreateRepositoryRequest>,
) -> Response {
    let recipe_name = format!("{}-{}", format, r_type);
    let remote_url = payload
        .proxy
        .and_then(|p| p.remote_url)
        .unwrap_or_default();
    let write_policy = payload
        .storage
        .and_then(|s| s.write_policy)
        .unwrap_or_else(|| "ALLOW_WRITE".to_string());

    match RepositoryService::create(
        &state.runtime,
        &payload.name,
        &recipe_name,
        &r_type,
        &format,
        &write_policy,
        1,
        payload.online.unwrap_or(true),
        &remote_url,
    )
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
