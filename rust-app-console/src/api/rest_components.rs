use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

use super::repository_content::AppState;
use crate::services::{AssetService, ComponentService, RepositoryService};

#[derive(Debug, Deserialize)]
pub struct RepoQueryParams {
    pub repository: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentXO {
    pub id: String,
    pub repository: String,
    pub format: String,
    pub group: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetXO {
    pub id: String,
    pub path: String,
    pub repository: String,
    pub format: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageXO<T> {
    pub items: Vec<T>,
    pub continuation_token: Option<String>,
}

pub async fn list_components(
    State(state): State<AppState>,
    Query(params): Query<RepoQueryParams>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &params.repository).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Repository not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let content_repo = match RepositoryService::get_content_repository(&state.runtime, repo.id()).await {
        Ok(Some(cr)) => cr,
        Ok(None) => return Json(PageXO::<ComponentXO> { items: vec![], continuation_token: None }).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match ComponentService::list_by_content_repository(&state.runtime, content_repo.id(), 50, 0).await {
        Ok(comps) => {
            let items: Vec<ComponentXO> = comps
                .into_iter()
                .map(|c| ComponentXO {
                    id: c.id().to_string(),
                    repository: params.repository.clone(),
                    format: content_repo.format_name().to_string(),
                    group: c.namespace().to_string(),
                    name: c.name().to_string(),
                    version: c.version_name().to_string(),
                })
                .collect();
            Json(PageXO { items, continuation_token: None }).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn list_assets(
    State(state): State<AppState>,
    Query(params): Query<RepoQueryParams>,
) -> Response {
    let repo = match RepositoryService::find_by_name(&state.runtime, &params.repository).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "Repository not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let content_repo = match RepositoryService::get_content_repository(&state.runtime, repo.id()).await {
        Ok(Some(cr)) => cr,
        Ok(None) => return Json(PageXO::<AssetXO> { items: vec![], continuation_token: None }).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    match AssetService::list_by_content_repository(&state.runtime, content_repo.id(), 50, 0).await {
        Ok(assets) => {
            let items: Vec<AssetXO> = assets
                .into_iter()
                .map(|a| AssetXO {
                    id: a.id().to_string(),
                    path: a.path().to_string(),
                    repository: params.repository.clone(),
                    format: content_repo.format_name().to_string(),
                    download_url: format!("/repository/{}{}", params.repository, a.path()),
                })
                .collect();
            Json(PageXO { items, continuation_token: None }).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
