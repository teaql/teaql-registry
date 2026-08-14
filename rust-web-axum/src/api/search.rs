use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::AppState;
use crate::services::{AssetService, ComponentService, RepositoryService};

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub name: Option<String>,
    pub version: Option<String>,
    pub group: Option<String>,
    pub repository: Option<String>,
    pub format: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchAssetItem {
    pub id: u64,
    pub path: String,
    pub download_url: String,
    pub format: String,
    pub repository: String,
    pub content_type: String,
    pub size: i64,
    pub checksum: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SearchComponentItem {
    pub id: u64,
    pub name: String,
    pub group: String,
    pub version: String,
    pub format: String,
    pub repository: String,
    pub assets: Vec<SearchAssetItem>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

pub async fn handle_search_components(
    State(state): State<AppState>,
    Query(params): Query<SearchQueryParams>,
) -> Response {
    let repos = match RepositoryService::list(&state.runtime).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut component_results = Vec::new();

    for repo in &repos {
        if let Some(ref target_repo) = params.repository {
            if repo.name() != *target_repo {
                continue;
            }
        }
        if let Some(ref target_format) = params.format {
            if !repo.recipe_name().to_lowercase().contains(&target_format.to_lowercase()) {
                continue;
            }
        }

        let content_repo = match RepositoryService::get_content_repository(&state.runtime, repo.id()).await {
            Ok(Some(cr)) => cr,
            _ => continue,
        };

        let components = match ComponentService::list_by_repository(&state.runtime, content_repo.id()).await {
            Ok(c) => c,
            _ => continue,
        };

        for comp in components {
            // Filter by name
            if let Some(ref name) = params.name {
                if !comp.name().to_lowercase().contains(&name.to_lowercase()) {
                    continue;
                }
            }
            // Filter by version
            if let Some(ref ver) = params.version {
                if comp.version_name() != *ver && comp.normalized_version() != *ver {
                    continue;
                }
            }
            // Filter by group/namespace
            if let Some(ref grp) = params.group {
                if !comp.namespace().to_lowercase().contains(&grp.to_lowercase()) {
                    continue;
                }
            }
            // Filter by general keyword
            if let Some(ref kw) = params.keyword {
                let kw_lower = kw.to_lowercase();
                let match_name = comp.name().to_lowercase().contains(&kw_lower);
                let match_group = comp.namespace().to_lowercase().contains(&kw_lower);
                let match_ver = comp.version_name().to_lowercase().contains(&kw_lower);
                if !match_name && !match_group && !match_ver {
                    continue;
                }
            }

            // Fetch assets for this component
            let assets = match AssetService::list_by_component(&state.runtime, comp.id()).await {
                Ok(a) => a,
                _ => vec![],
            };

            let mut asset_items = Vec::new();
            for asset in assets {
                let asset_blob = AssetService::get_asset_blob(&state.runtime, asset.asset_blob_id())
                    .await
                    .ok()
                    .flatten();

                let mut checksums = HashMap::new();
                let (size, content_type) = if let Some(ref blob) = asset_blob {
                    checksums.insert("sha1".to_string(), blob.sha1_checksum().to_string());
                    checksums.insert("sha256".to_string(), blob.sha256_checksum().to_string());
                    checksums.insert("md5".to_string(), blob.md5_checksum().to_string());
                    (blob.blob_size(), blob.content_type().to_string())
                } else {
                    (0, "application/octet-stream".to_string())
                };

                asset_items.push(SearchAssetItem {
                    id: asset.id(),
                    path: asset.path().to_string(),
                    download_url: format!("/repository/{}{}", repo.name(), asset.path()),
                    format: content_repo.format_name().to_string(),
                    repository: repo.name().to_string(),
                    content_type,
                    size,
                    checksum: checksums,
                });
            }

            component_results.push(SearchComponentItem {
                id: comp.id(),
                name: comp.name().to_string(),
                group: comp.namespace().to_string(),
                version: comp.version_name().to_string(),
                format: content_repo.format_name().to_string(),
                repository: repo.name().to_string(),
                assets: asset_items,
            });
        }
    }

    let total = component_results.len();
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let start = (page - 1) * page_size;

    let paged_items: Vec<SearchComponentItem> = component_results
        .into_iter()
        .skip(start)
        .take(page_size)
        .collect();

    Json(SearchResponse {
        items: paged_items,
        total,
        page,
        page_size,
    })
    .into_response()
}

pub async fn handle_search_assets(
    State(state): State<AppState>,
    Query(params): Query<SearchQueryParams>,
) -> Response {
    let repos = match RepositoryService::list(&state.runtime).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut asset_results = Vec::new();

    for repo in &repos {
        if let Some(ref target_repo) = params.repository {
            if repo.name() != *target_repo {
                continue;
            }
        }
        if let Some(ref target_format) = params.format {
            if !repo.recipe_name().to_lowercase().contains(&target_format.to_lowercase()) {
                continue;
            }
        }

        let content_repo = match RepositoryService::get_content_repository(&state.runtime, repo.id()).await {
            Ok(Some(cr)) => cr,
            _ => continue,
        };

        let assets = match AssetService::list_by_repository(&state.runtime, content_repo.id()).await {
            Ok(a) => a,
            _ => continue,
        };

        for asset in assets {
            if let Some(ref kw) = params.keyword {
                if !asset.path().to_lowercase().contains(&kw.to_lowercase()) {
                    continue;
                }
            }

            let asset_blob = AssetService::get_asset_blob(&state.runtime, asset.asset_blob_id())
                .await
                .ok()
                .flatten();

            let mut checksums = HashMap::new();
            let (size, content_type) = if let Some(ref blob) = asset_blob {
                checksums.insert("sha1".to_string(), blob.sha1_checksum().to_string());
                checksums.insert("sha256".to_string(), blob.sha256_checksum().to_string());
                checksums.insert("md5".to_string(), blob.md5_checksum().to_string());
                (blob.blob_size(), blob.content_type().to_string())
            } else {
                (0, "application/octet-stream".to_string())
            };

            asset_results.push(SearchAssetItem {
                id: asset.id(),
                path: asset.path().to_string(),
                download_url: format!("/repository/{}{}", repo.name(), asset.path()),
                format: content_repo.format_name().to_string(),
                repository: repo.name().to_string(),
                content_type,
                size,
                checksum: checksums,
            });
        }
    }

    let total = asset_results.len();
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let start = (page - 1) * page_size;

    let paged_items: Vec<SearchAssetItem> = asset_results
        .into_iter()
        .skip(start)
        .take(page_size)
        .collect();

    Json(SearchResponse {
        items: paged_items,
        total,
        page,
        page_size,
    })
    .into_response()
}
