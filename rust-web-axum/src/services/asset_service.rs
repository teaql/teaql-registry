use anyhow::{anyhow, Result};
use teaql_registry_core::{
    Asset, AssetBlob, Q, ServiceRuntime,
};
use teaql_core::{Entity, SmartList};
use crate::services::SaveAuditedExt;

pub struct AssetService;

impl AssetService {
    pub async fn find_by_path(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        path: &str,
    ) -> Result<Option<Asset>> {
        let records = Q::assets_minimal()
            .select_self_fields()
            .filter_by_content_repository(content_repo_id)
            .with_path_is(path)
            .limit(1)
            .comment("what: Load asset by content repository and path")
            .purpose("why: Serve or check asset existence")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find asset: {}", e))?;

        if let Some(record) = records.into_iter().next() {
            let asset = Asset::from_record(record)
                .map_err(|e| anyhow!("Failed to parse asset: {}", e))?;
            Ok(Some(asset))
        } else {
            Ok(None)
        }
    }

    pub async fn list_by_content_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        limit: u64,
        offset: u64,
    ) -> Result<SmartList<Asset>> {
        let records = Q::assets_minimal()
            .select_self_fields()
            .filter_by_content_repository(content_repo_id)
            .offset(offset, limit)
            .comment("what: Load assets for content repository")
            .purpose("why: REST assets query API")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list assets: {}", e))?;

        let mut items = Vec::new();
        for rec in records {
            let asset = Asset::from_record(rec).map_err(|e| anyhow!("Failed to parse asset: {}", e))?;
            items.push(asset);
        }
        Ok(SmartList::new(items))
    }

    pub async fn create_asset_blob(
        ctx: &ServiceRuntime,
        blob_store_id: u64,
        blob_ref: &str,
        size: i64,
        content_type: &str,
        sha1: &str,
        sha256: &str,
        md5: &str,
    ) -> Result<AssetBlob> {
        let mut blob = Q::asset_blobs()
            .purpose("why: Create new asset blob record")
            .new_entity(ctx);

        blob.update_blob_store_id(blob_store_id);
        blob.update_blob_ref(blob_ref);
        blob.update_blob_size(size);
        blob.update_content_type(content_type);
        blob.update_sha1_checksum(sha1);
        blob.update_sha256_checksum(sha256);
        blob.update_md5_checksum(md5);

        blob.clone()
            .audit_as("Creating asset blob record")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save asset blob: {}", e))?;

        Ok(blob)
    }

    pub async fn get_asset_blob(
        ctx: &ServiceRuntime,
        blob_id: u64,
    ) -> Result<Option<AssetBlob>> {
        let records = Q::asset_blobs_minimal()
            .select_self_fields()
            .with_id_is(blob_id)
            .limit(1)
            .comment("what: Load asset blob by id")
            .purpose("why: Fetch blob metadata for download")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to load asset blob: {}", e))?;

        if let Some(record) = records.into_iter().next() {
            let blob = AssetBlob::from_record(record).map_err(|e| anyhow!("Failed to parse asset blob: {}", e))?;
            Ok(Some(blob))
        } else {
            Ok(None)
        }
    }

    pub async fn upsert_asset(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        component_id: Option<u64>,
        asset_blob_id: u64,
        path: &str,
        kind: &str,
    ) -> Result<Asset> {
        let existing = Self::find_by_path(ctx, content_repo_id, path).await?;

        if let Some(mut asset) = existing {
            asset.update_kind(kind);
            if let Some(cid) = component_id {
                asset.update_component_id(cid);
            }
            asset.update_asset_blob_id(asset_blob_id);

            asset.clone()
                .audit_as("Updating existing asset record")
                .save_with(ctx)
                .await
                .map_err(|e| anyhow!("Failed to update asset: {}", e))?;

            return Ok(asset);
        }

        let mut asset = Q::assets()
            .purpose("why: Create new asset record")
            .new_entity(ctx);

        asset.update_content_repository_id(content_repo_id);
        asset.update_component_id(component_id.unwrap_or(0));
        asset.update_asset_blob_id(asset_blob_id);
        asset.update_path(path);
        asset.update_kind(kind);

        asset.clone()
            .audit_as("Creating new asset record")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save asset: {}", e))?;

        Ok(asset)
    }
}
