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
        let rows = Q::assets()
            .filter_by_content_repository(content_repo_id)
            .with_path_is(path)
            .limit(1)
            .comment("what: Load asset by content repository and path")
            .purpose("why: Serve or check asset existence")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find asset: {}", e))?;

        if let Some(asset) = rows.into_iter().next() {
            if asset.path().is_empty() {
                Ok(None)
            } else {
                Ok(Some(asset))
            }
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
        let rows = Q::assets()
            .filter_by_content_repository(content_repo_id)
            .offset(offset, limit)
            .comment("what: Load assets for content repository")
            .purpose("why: REST assets query API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list assets: {}", e))?;

        let filtered: Vec<Asset> = rows.into_iter().filter(|a| !a.path().is_empty()).collect();
        Ok(SmartList::new(filtered))
    }

    pub async fn list_by_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
    ) -> Result<Vec<Asset>> {
        let smart_list = Self::list_by_content_repository(ctx, content_repo_id, 1000, 0).await?;
        Ok(smart_list.into_iter().collect())
    }

    pub async fn list_by_component(
        ctx: &ServiceRuntime,
        component_id: u64,
    ) -> Result<Vec<Asset>> {
        let rows = Q::assets()
            .with_component_id_is(component_id)
            .limit(1000)
            .comment("what: List assets for component")
            .purpose("why: List assets for component")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list assets by component: {}", e))?;

        Ok(rows.into_iter().filter(|a| !a.path().is_empty()).collect())
    }

    pub async fn create(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        component_id: u64,
        asset_blob_id: u64,
        path: &str,
        kind: &str,
    ) -> Result<Asset> {
        let mut asset = Q::assets()
            .purpose("why: Create new asset record")
            .new_entity(ctx);

        asset.update_content_repository_id(content_repo_id);
        asset.update_component_id(component_id);
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
        let rows = Q::asset_blobs()
            .with_id_is(blob_id)
            .limit(1)
            .comment("what: Load asset blob by id")
            .purpose("why: Fetch blob metadata for download")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to load asset blob: {}", e))?;

        if let Some(blob) = rows.into_iter().next() {
            if blob.blob_ref().is_empty() || blob.blob_size() <= 0 {
                Ok(None)
            } else {
                Ok(Some(blob))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn list_all_blobs(ctx: &ServiceRuntime) -> Result<Vec<AssetBlob>> {
        let rows = Q::asset_blobs()
            .limit(10000)
            .comment("what: List all asset blobs")
            .purpose("why: List all asset blobs for metrics and GC")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list all asset blobs: {}", e))?;

        Ok(rows.into_iter().filter(|b| !b.blob_ref().is_empty() && b.blob_size() > 0).collect())
    }

    pub async fn list_all_assets(ctx: &ServiceRuntime) -> Result<Vec<Asset>> {
        let rows = Q::assets()
            .limit(10000)
            .comment("what: List all assets")
            .purpose("why: List all assets for metrics and GC")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list all assets: {}", e))?;

        Ok(rows.into_iter().filter(|a| !a.path().is_empty()).collect())
    }

    pub async fn delete(ctx: &ServiceRuntime, asset_id: u64) -> Result<()> {
        let rows = Q::assets()
            .with_id_is(asset_id)
            .limit(1)
            .comment("what: Find asset to delete")
            .purpose("why: Find asset to delete")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find asset for delete: {}", e))?;

        if let Some(mut asset) = rows.into_iter().next() {
            asset.update_path("");
            let _ = asset.audit_as("Deleting asset").save_with(ctx).await?;
        }
        Ok(())
    }

    pub async fn delete_asset_blob(ctx: &ServiceRuntime, blob_id: u64) -> Result<()> {
        let rows = Q::asset_blobs()
            .with_id_is(blob_id)
            .limit(1)
            .comment("what: Find asset blob to delete")
            .purpose("why: Find asset blob to delete")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find asset blob for delete: {}", e))?;

        if let Some(mut blob) = rows.into_iter().next() {
            blob.update_blob_ref("");
            blob.update_blob_size(0);
            let _ = blob.audit_as("Deleting asset blob").save_with(ctx).await?;
        }
        Ok(())
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
