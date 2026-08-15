#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RepositoryItem {
    pub name: String,
    pub format: String,
    #[serde(rename = "type")]
    pub repo_type: String,
    pub url: String,
    pub online: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ChecksumMap {
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub md5: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AssetItem {
    pub id: i64,
    pub path: String,
    pub download_url: String,
    pub format: String,
    pub repository: String,
    pub content_type: String,
    pub size: i64,
    pub checksum: Option<ChecksumMap>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ComponentItem {
    pub id: i64,
    pub name: String,
    pub group: String,
    pub version: String,
    pub format: String,
    pub repository: String,
    pub assets: Vec<AssetItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SearchResponse {
    pub items: Vec<ComponentItem>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GcReport {
    pub scanned_blobs_count: usize,
    pub orphaned_blobs_deleted: usize,
    pub freed_bytes: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CleanupReport {
    pub deleted_components_count: usize,
    pub deleted_assets_count: usize,
    pub freed_bytes: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CreateTokenResponse {
    pub token: String,
}

#[derive(Debug, Clone, Default)]
pub struct ServerOverview {
    pub status: String,
    pub is_online: bool,
    pub total_repositories: usize,
    pub hosted_count: usize,
    pub proxy_count: usize,
    pub group_count: usize,
    pub format_counts: Vec<(String, usize)>,
    pub total_components: usize,
    pub metrics_raw: String,
}
