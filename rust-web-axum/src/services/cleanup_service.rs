use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::blobstore::BlobStore;
use crate::services::{AssetService, ComponentService, RepositoryService};
use teaql_registry_core::ServiceRuntime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    pub max_versions_per_component: Option<usize>,
    pub snapshot_only: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CleanupReport {
    pub deleted_components_count: usize,
    pub deleted_assets_count: usize,
    pub freed_bytes: i64,
}

pub struct CleanupService;

impl CleanupService {
    pub async fn run_cleanup(
        ctx: &ServiceRuntime,
        blobstore: &dyn BlobStore,
        repo_name: &str,
        policy: &CleanupPolicy,
    ) -> Result<CleanupReport> {
        let repo = RepositoryService::find_by_name(ctx, repo_name)
            .await?
            .ok_or_else(|| anyhow!("Repository not found: {}", repo_name))?;

        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(CleanupReport::default()),
        };

        let components = ComponentService::list_by_repository(ctx, content_repo.id()).await?;
        let mut grouped_by_name: HashMap<String, Vec<_>> = HashMap::new();

        for comp in components {
            let key = format!("{}:{}", comp.namespace(), comp.name());
            grouped_by_name.entry(key).or_default().push(comp);
        }

        let mut report = CleanupReport::default();

        for (_key, mut comp_list) in grouped_by_name {
            if let Some(max_versions) = policy.max_versions_per_component {
                // Sort newest to oldest by version or ID
                comp_list.sort_by(|a, b| b.id().cmp(&a.id()));

                if comp_list.len() > max_versions {
                    let to_delete = &comp_list[max_versions..];
                    for comp in to_delete {
                        if policy.snapshot_only && !comp.version_name().contains("SNAPSHOT") {
                            continue;
                        }

                        let assets = AssetService::list_by_component(ctx, comp.id()).await?;
                        for asset in assets {
                            if let Ok(Some(blob)) = AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await {
                                report.freed_bytes += blob.blob_size();
                                let _ = blobstore.delete_blob(&blob.blob_ref()).await;
                                let _ = AssetService::delete_asset_blob(ctx, blob.id()).await;
                            }
                            let _ = AssetService::delete(ctx, asset.id()).await;
                            report.deleted_assets_count += 1;
                        }

                        let _ = ComponentService::delete(ctx, comp.id()).await;
                        report.deleted_components_count += 1;
                        info!("Cleaned up old component version: {}/{}", comp.name(), comp.version_name());
                    }
                }
            }
        }

        Ok(report)
    }
}
