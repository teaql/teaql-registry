use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use teaql_registry_core::Q;

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
    pub faceted: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FacetQueryParams {
    pub name: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RepositoryFacet {
    pub name: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct FacetResponse {
    pub repositories: Vec<RepositoryFacet>,
}

/// TeaQL's relation Facet finds content repositories represented in the full
/// matching component set, independently of the visible result page.
pub async fn handle_search_facets(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Query(params): Query<FacetQueryParams>,
) -> Response {
    let mut query = Q::components_minimal()
        .select_name()
        .with_name_is_not("")
        .with_name_not_containing("[DELETED")
        .with_kind_is_not("deleted");

    if let Some(name) = params
        .name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        query = query.with_name_containing(name.to_lowercase());
    }
    if let Some(group) = params
        .group
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        query = query.with_namespace_containing(group.to_lowercase());
    }

    let rows = match query
        .facet_by_content_repository_as_with_options(
            "repositories",
            Q::content_repositories_minimal()
                .select_repository_id()
                .select_format_name()
                .order_by_repository_id_asc()
                .limit(1000),
            false,
        )
        .order_by_id_asc()
        .limit(1)
        .comment("what: find content repository facets for matching package names")
        .purpose("why: let users narrow artifact search by repository and format")
        .execute_for_list(request.runtime.as_ref())
        .await
    {
        Ok(rows) => rows,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    };

    let matched: HashMap<u64, String> = rows
        .facet("repositories")
        .into_iter()
        .flat_map(|facet| facet.iter())
        .filter_map(|row| {
            Some((
                row.get("repository_id")?.try_u64()?,
                row.get("format_name")?.try_text()?.to_string(),
            ))
        })
        .collect();

    let repos = match RepositoryService::list(&request.runtime).await {
        Ok(repos) => repos,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    };
    let mut repositories: Vec<_> = repos
        .iter()
        .filter_map(|repo| {
            matched.get(&repo.id()).map(|format| RepositoryFacet {
                name: repo.name().to_string(),
                format: format.clone(),
            })
        })
        .collect();
    repositories.sort_by(|left, right| left.name.cmp(&right.name));

    Json(FacetResponse { repositories }).into_response()
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

// Compare numeric version segments before qualifiers so that 1.169 sorts after
// 1.132, while a final release sorts after a snapshot or release candidate.
fn compare_versions(left: &str, right: &str) -> Ordering {
    match (numeric_version(left), numeric_version(right)) {
        (Some((left_numbers, left_qualifier)), Some((right_numbers, right_qualifier))) => {
            for index in 0..left_numbers.len().max(right_numbers.len()) {
                let order = left_numbers
                    .get(index)
                    .unwrap_or(&0)
                    .cmp(right_numbers.get(index).unwrap_or(&0));
                if order != Ordering::Equal {
                    return order;
                }
            }
            qualifier_rank(left_qualifier)
                .cmp(&qualifier_rank(right_qualifier))
                .then_with(|| natural_version_cmp(left_qualifier, right_qualifier))
        }
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => natural_version_cmp(left, right),
    }
}

fn numeric_version(version: &str) -> Option<(Vec<u64>, &str)> {
    let version = version.strip_prefix(['v', 'V']).unwrap_or(version);
    let end = version
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(version.len());
    let core = &version[..end];
    if core.is_empty() || core.ends_with('.') {
        return None;
    }
    let numbers = core
        .split('.')
        .map(str::parse)
        .collect::<Result<Vec<u64>, _>>()
        .ok()?;
    let qualifier = version[end..].trim_start_matches(['-', '.', '_', '+']);
    Some((numbers, qualifier))
}

fn qualifier_rank(qualifier: &str) -> u8 {
    let qualifier = qualifier.to_ascii_lowercase();
    if qualifier.is_empty() || ["release", "final", "ga"].contains(&qualifier.as_str()) {
        5
    } else if qualifier.starts_with("sp") {
        6
    } else if qualifier.starts_with("rc") || qualifier.starts_with("cr") {
        4
    } else if qualifier.starts_with("beta") {
        3
    } else if qualifier.starts_with("alpha") {
        2
    } else if qualifier.starts_with("snapshot")
        || qualifier.starts_with("dev")
        || qualifier.starts_with("pre")
    {
        1
    } else {
        3
    }
}

fn natural_version_cmp(left: &str, right: &str) -> Ordering {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        if left[i].is_ascii_digit() && right[j].is_ascii_digit() {
            let (mut left_start, mut right_start) = (i, j);
            while i < left.len() && left[i].is_ascii_digit() {
                i += 1;
            }
            while j < right.len() && right[j].is_ascii_digit() {
                j += 1;
            }
            while left_start < i && left[left_start] == b'0' {
                left_start += 1;
            }
            while right_start < j && right[right_start] == b'0' {
                right_start += 1;
            }
            let left_number = &left[left_start..i];
            let right_number = &right[right_start..j];
            let order = left_number
                .len()
                .cmp(&right_number.len())
                .then_with(|| left_number.cmp(right_number));
            if order != Ordering::Equal {
                return order;
            }
        } else {
            let order = left[i]
                .to_ascii_lowercase()
                .cmp(&right[j].to_ascii_lowercase());
            if order != Ordering::Equal {
                return order;
            }
            i += 1;
            j += 1;
        }
    }
    left.len().cmp(&right.len())
}

pub async fn handle_search_components(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Query(params): Query<SearchQueryParams>,
) -> Response {
    let repos = match RepositoryService::list(&request.runtime).await {
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
            if !repo
                .recipe_name()
                .to_lowercase()
                .contains(&target_format.to_lowercase())
            {
                continue;
            }
        }

        let content_repo =
            match RepositoryService::get_content_repository(&request.runtime, repo.id()).await {
                Ok(Some(cr)) => cr,
                _ => continue,
            };

        let has_filter = params.faceted.unwrap_or(false)
            || params.name.is_some()
            || params.version.is_some()
            || params.group.is_some()
            || params.keyword.is_some();
        let mut components = Vec::new();
        let mut offset = 0;
        loop {
            let (page, has_more) = match ComponentService::list_search_page(
                &request.runtime,
                content_repo.id(),
                1000,
                offset,
            )
            .await
            {
                Ok(page) => page,
                Err(error) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
                }
            };
            components.extend(page);
            if !has_more || !has_filter {
                break;
            }
            offset += 1000;
            if offset >= 100_000 {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Search exceeds 100,000 components in a repository",
                )
                    .into_response();
            }
        }

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
                if !comp
                    .namespace()
                    .to_lowercase()
                    .contains(&grp.to_lowercase())
                {
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

            component_results.push(SearchComponentItem {
                id: comp.id(),
                name: comp.name().to_string(),
                group: comp.namespace().to_string(),
                version: comp.version_name().to_string(),
                format: content_repo.format_name().to_string(),
                repository: repo.name().to_string(),
                assets: Vec::new(),
            });
        }
    }

    // Sort before pagination so the first page shows recent versions of each package.
    component_results.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| {
                left.group
                    .to_ascii_lowercase()
                    .cmp(&right.group.to_ascii_lowercase())
            })
            .then_with(|| compare_versions(&right.version, &left.version))
            .then_with(|| left.repository.cmp(&right.repository))
            .then_with(|| right.id.cmp(&left.id))
    });

    let total = component_results.len();
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let start = (page - 1) * page_size;

    let mut paged_items: Vec<SearchComponentItem> = component_results
        .into_iter()
        .skip(start)
        .take(page_size)
        .collect();

    // Fetch files only for the visible page, not every matching version.
    for item in &mut paged_items {
        let assets = AssetService::list_by_component_with_blobs(&request.runtime, item.id)
            .await
            .unwrap_or_default();
        for asset in assets {
            let mut checksums = HashMap::new();
            let (size, content_type) = if let Some(blob) = asset.blob() {
                checksums.insert("sha1".to_string(), blob.sha1_checksum().to_owned());
                checksums.insert("sha256".to_string(), blob.sha256_checksum().to_owned());
                checksums.insert("md5".to_string(), blob.md5_checksum().to_owned());
                (blob.blob_size(), blob.content_type().to_owned())
            } else {
                (0, "application/octet-stream".to_string())
            };

            item.assets.push(SearchAssetItem {
                id: asset.id(),
                path: asset.path().to_owned(),
                download_url: format!("/repository/{}{}", item.repository, asset.path()),
                format: item.format.clone(),
                repository: item.repository.clone(),
                content_type,
                size,
                checksum: checksums,
            });
        }
    }

    Json(SearchResponse {
        items: paged_items,
        total,
        page,
        page_size,
    })
    .into_response()
}

pub async fn handle_search_assets(
    State(_state): State<AppState>,
    axum::extract::Extension(request): axum::extract::Extension<
        std::sync::Arc<crate::security::RequestContext>,
    >,
    Query(params): Query<SearchQueryParams>,
) -> Response {
    let repos = match RepositoryService::list(&request.runtime).await {
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
            if !repo
                .recipe_name()
                .to_lowercase()
                .contains(&target_format.to_lowercase())
            {
                continue;
            }
        }

        let content_repo =
            match RepositoryService::get_content_repository(&request.runtime, repo.id()).await {
                Ok(Some(cr)) => cr,
                _ => continue,
            };

        let assets = match AssetService::list_by_content_repository_with_blobs(
            &request.runtime,
            content_repo.id(),
            1000,
            0,
        )
        .await
        {
            Ok(a) => a,
            _ => continue,
        };

        for asset in assets {
            if let Some(ref kw) = params.keyword {
                if !asset.path().to_lowercase().contains(&kw.to_lowercase()) {
                    continue;
                }
            }

            let mut checksums = HashMap::new();
            let (size, content_type) = if let Some(blob) = asset.blob() {
                checksums.insert("sha1".to_string(), blob.sha1_checksum().to_owned());
                checksums.insert("sha256".to_string(), blob.sha256_checksum().to_owned());
                checksums.insert("md5".to_string(), blob.md5_checksum().to_owned());
                (blob.blob_size(), blob.content_type().to_owned())
            } else {
                (0, "application/octet-stream".to_string())
            };

            asset_results.push(SearchAssetItem {
                id: asset.id(),
                path: asset.path().to_owned(),
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

#[cfg(test)]
mod version_order_tests {
    use super::compare_versions;
    use std::cmp::Ordering;

    #[test]
    fn maven_release_versions_sort_numerically() {
        assert_eq!(
            compare_versions("1.169-RELEASE", "1.132-RELEASE"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.10-RELEASE", "1.9-RELEASE"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.10-RELEASE", "1.10-SNAPSHOT"),
            Ordering::Greater
        );
    }

    #[test]
    fn release_and_numeric_suffix_ordering() {
        assert_eq!(compare_versions("2.0.0", "2.0.0-rc.1"), Ordering::Greater);
        assert_eq!(
            compare_versions("2.0.0-rc.10", "2.0.0-rc.9"),
            Ordering::Greater
        );
        assert_eq!(
            compare_versions("release-10", "release-9"),
            Ordering::Greater
        );
    }
}
