use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::info;

use crate::blobstore::BlobStore;
use crate::services::AssetService;
use teaql_registry_core::ServiceRuntime;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GcReport {
    pub scanned_blobs_count: usize,
    pub orphaned_blobs_deleted: usize,
    pub freed_bytes: i64,
}

pub struct BlobStoreGcService;

impl BlobStoreGcService {
    pub async fn run_gc(ctx: &ServiceRuntime, blobstore: &dyn BlobStore) -> Result<GcReport> {
        let all_blobs = AssetService::list_all_blobs(ctx).await?;
        let all_assets = AssetService::list_all_assets(ctx).await?;

        let referenced_blob_ids: HashSet<u64> = all_assets.into_iter().map(|a| a.asset_blob_id()).collect();

        let mut report = GcReport {
            scanned_blobs_count: all_blobs.len(),
            orphaned_blobs_deleted: 0,
            freed_bytes: 0,
        };

        for blob in all_blobs {
            if !referenced_blob_ids.contains(&blob.id()) {
                report.freed_bytes += blob.blob_size();
                report.orphaned_blobs_deleted += 1;

                // 1. Delete physical object in BlobStore
                let _ = blobstore.delete_blob(&blob.blob_ref()).await;

                // 2. Delete database record
                let _ = AssetService::delete_asset_blob(ctx, blob.id()).await;

                info!("Garbage collected orphaned blob: id={} ref={}", blob.id(), blob.blob_ref());
            }
        }

        Ok(report)
    }
}
