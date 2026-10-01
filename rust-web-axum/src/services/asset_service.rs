use crate::services::SaveAuditedExt;
use anyhow::{anyhow, Result};
use teaql_core::Entity;
use teaql_registry_core::{Asset, AssetBlob, ServiceRuntime, E, Q};

/// Application-owned read model built only from an explicitly selected
/// `AssetBlob`. Constructing it through `E` preserves TeaQL's distinction
/// between a loaded value, a loaded null, and a missing projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedAssetBlob {
    id: u64,
    blob_ref: String,
    blob_size: i64,
    content_type: String,
    sha1_checksum: String,
    sha256_checksum: String,
    md5_checksum: String,
}

impl LoadedAssetBlob {
    fn from_selected(entity: &AssetBlob) -> Self {
        Self {
            id: E::asset_blob(entity).get_id().unwrap(),
            blob_ref: E::asset_blob(entity).get_blob_ref().unwrap(),
            blob_size: E::asset_blob(entity).get_blob_size().unwrap(),
            content_type: E::asset_blob(entity).get_content_type().unwrap(),
            sha1_checksum: E::asset_blob(entity).get_sha1_checksum().unwrap(),
            sha256_checksum: E::asset_blob(entity).get_sha256_checksum().unwrap(),
            md5_checksum: E::asset_blob(entity).get_md5_checksum().unwrap(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn blob_ref(&self) -> &str {
        &self.blob_ref
    }

    pub fn blob_size(&self) -> i64 {
        self.blob_size
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn sha1_checksum(&self) -> &str {
        &self.sha1_checksum
    }

    pub fn sha256_checksum(&self) -> &str {
        &self.sha256_checksum
    }

    pub fn md5_checksum(&self) -> &str {
        &self.md5_checksum
    }
}

/// Read-only Asset projection with its optional, explicitly loaded blob.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedAsset {
    id: u64,
    component_id: i64,
    path: String,
    kind: String,
    blob: Option<LoadedAssetBlob>,
}

impl LoadedAsset {
    fn from_selected(entity: Asset) -> Self {
        let blob = E::asset(&entity)
            .get_asset_blob()
            .eval()
            .map(LoadedAssetBlob::from_selected);
        Self {
            id: E::asset(&entity).get_id().unwrap(),
            component_id: E::asset(&entity).get_component_id().unwrap(),
            path: E::asset(&entity).get_path().unwrap(),
            kind: E::asset(&entity).get_kind().unwrap(),
            blob,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn component_id(&self) -> i64 {
        self.component_id
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn blob(&self) -> Option<&LoadedAssetBlob> {
        self.blob.as_ref()
    }
}

pub struct AssetService;

impl AssetService {
    pub async fn find_by_path(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        path: &str,
    ) -> Result<Option<Asset>> {
        let rows = Q::assets()
            .select_self_fields()
            .filter_by_content_repository(content_repo_id)
            .with_path_is(path)
            .limit(1)
            .comment("what: query registry asset metadata")
            .purpose("why: resolve package content and integrity records")
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

    /// Load an asset together with the blob metadata required by download
    /// paths. Callers must traverse the selected relation through `E` so an
    /// accidental projection regression remains a visible programming error.
    pub async fn find_by_path_with_blob(
        context: &ServiceRuntime,
        content_repo_id: u64,
        path: &str,
    ) -> Result<Option<LoadedAsset>> {
        let rows = Q::assets_minimal()
            .select_component_id()
            .select_path()
            .select_kind()
            .select_asset_blob_with(
                Q::asset_blobs_minimal()
                    .select_blob_ref()
                    .select_blob_size()
                    .select_content_type()
                    .select_sha1_checksum()
                    .select_sha256_checksum()
                    .select_md5_checksum(),
            )
            .filter_by_content_repository(content_repo_id)
            .with_path_is(path)
            .limit(1)
            .comment("what: load one registry asset with its blob metadata")
            .purpose("why: serve package content without a follow-up blob query")
            .execute_for_list(context)
            .await
            .map_err(|e| anyhow!("Failed to find asset with blob metadata: {}", e))?;

        Ok(rows
            .into_iter()
            .next()
            .map(LoadedAsset::from_selected)
            .filter(|asset| !asset.path().is_empty()))
    }

    pub async fn list_by_content_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<LoadedAsset>> {
        Self::list_by_content_repository_with_blobs(ctx, content_repo_id, limit, offset).await
    }

    pub async fn list_by_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
    ) -> Result<Vec<LoadedAsset>> {
        Self::list_by_content_repository(ctx, content_repo_id, 1000, 0).await
    }

    pub async fn list_by_component(
        ctx: &ServiceRuntime,
        component_id: u64,
    ) -> Result<Vec<LoadedAsset>> {
        Self::list_by_component_with_blobs(ctx, component_id).await
    }

    /// Load a component's assets and blob metadata in one relation-aware
    /// query. This is the preferred read path for search and cleanup loops.
    pub async fn list_by_component_with_blobs(
        context: &ServiceRuntime,
        component_id: u64,
    ) -> Result<Vec<LoadedAsset>> {
        let rows = Q::assets_minimal()
            .select_component_id()
            .select_path()
            .select_kind()
            .select_asset_blob_with(
                Q::asset_blobs_minimal()
                    .select_blob_ref()
                    .select_blob_size()
                    .select_content_type()
                    .select_sha1_checksum()
                    .select_sha256_checksum()
                    .select_md5_checksum(),
            )
            .with_component_id_is(component_id)
            .order_by_id_asc()
            .comment("what: query component assets with blob metadata")
            .purpose("why: avoid per-asset blob queries in bounded package workflows")
            .execute_for_page(context, 0, 1000)
            .await
            .map_err(|e| anyhow!("Failed to list assets with blob metadata: {}", e))?;

        Ok(rows
            .into_iter()
            .map(LoadedAsset::from_selected)
            .filter(|asset| !asset.path().is_empty())
            .collect())
    }

    /// Load a bounded page of repository assets with their blob relation.
    pub async fn list_by_content_repository_with_blobs(
        context: &ServiceRuntime,
        content_repo_id: u64,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<LoadedAsset>> {
        let rows = Q::assets_minimal()
            .select_component_id()
            .select_path()
            .select_kind()
            .select_asset_blob_with(
                Q::asset_blobs_minimal()
                    .select_blob_ref()
                    .select_blob_size()
                    .select_content_type()
                    .select_sha1_checksum()
                    .select_sha256_checksum()
                    .select_md5_checksum(),
            )
            .filter_by_content_repository(content_repo_id)
            .order_by_id_asc()
            .comment("what: query a bounded asset page with blob metadata")
            .purpose("why: render package search results without N+1 blob queries")
            .execute_for_page(context, offset, limit)
            .await
            .map_err(|e| anyhow!("Failed to list repository assets with blobs: {}", e))?;

        Ok(rows
            .into_iter()
            .map(LoadedAsset::from_selected)
            .filter(|asset| !asset.path().is_empty())
            .collect())
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
            .comment("what: create registry asset metadata")
            .purpose("why: persist package content and integrity records")
            .new_entity(ctx);
        asset.update_content_repository_id(content_repo_id);
        asset.update_component_id(component_id);
        asset.update_asset_blob_id(asset_blob_id);
        asset.update_path(path);
        asset.update_kind(kind);

        let asset = asset
            .audit_as("Creating new asset record")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save asset: {}", e))?;

        Ok(asset)
    }

    #[allow(clippy::too_many_arguments)]
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
            .comment("what: create registry asset metadata")
            .purpose("why: persist package content and integrity records")
            .new_entity(ctx);
        blob.update_blob_store_id(blob_store_id);
        blob.update_blob_ref(blob_ref);
        blob.update_blob_size(size);
        blob.update_content_type(content_type);
        blob.update_sha1_checksum(sha1);
        blob.update_sha256_checksum(sha256);
        blob.update_md5_checksum(md5);

        let blob = blob
            .audit_as("Creating asset blob record")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save asset blob: {}", e))?;

        Ok(blob)
    }

    pub async fn get_asset_blob(
        ctx: &ServiceRuntime,
        blob_id: u64,
    ) -> Result<Option<LoadedAssetBlob>> {
        let rows = Q::asset_blobs_minimal()
            .select_blob_ref()
            .select_blob_size()
            .select_content_type()
            .select_sha1_checksum()
            .select_sha256_checksum()
            .select_md5_checksum()
            .with_id_is(blob_id)
            .limit(1)
            .comment("what: query registry asset metadata")
            .purpose("why: resolve package content and integrity records")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to load asset blob: {}", e))?;

        Ok(rows
            .into_iter()
            .next()
            .map(|blob| LoadedAssetBlob::from_selected(&blob))
            .filter(|blob| !blob.blob_ref().is_empty() && blob.blob_size() > 0))
    }

    pub async fn list_all_blobs(ctx: &ServiceRuntime) -> Result<Vec<LoadedAssetBlob>> {
        let rows = Q::asset_blobs_minimal()
            .select_blob_ref()
            .select_blob_size()
            .select_content_type()
            .select_sha1_checksum()
            .select_sha256_checksum()
            .select_md5_checksum()
            .limit(10000)
            .comment("what: query registry asset metadata")
            .purpose("why: resolve package content and integrity records")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list all asset blobs: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|blob| LoadedAssetBlob::from_selected(&blob))
            .filter(|b| !b.blob_ref().is_empty() && b.blob_size() > 0)
            .collect())
    }

    pub async fn list_all_referenced_blob_ids(ctx: &ServiceRuntime) -> Result<Vec<u64>> {
        let rows = Q::assets_minimal()
            .select_path()
            .select_asset_blob_with(Q::asset_blobs_minimal())
            .limit(10000)
            .comment("what: query registry asset metadata")
            .purpose("why: resolve package content and integrity records")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list all assets: {}", e))?;

        Ok(rows
            .into_iter()
            .filter_map(|asset| {
                let path = E::asset(&asset).get_path().unwrap();
                (!path.is_empty())
                    .then(|| E::asset(&asset).get_asset_blob().eval())
                    .flatten()
                    .map(|blob| E::asset_blob(blob).get_id().unwrap())
            })
            .collect())
    }

    pub async fn delete(ctx: &ServiceRuntime, asset_id: u64) -> Result<()> {
        let rows = Q::assets()
            .select_self_fields()
            .with_id_is(asset_id)
            .limit(1)
            .comment("what: query registry asset metadata")
            .purpose("why: resolve package content and integrity records")
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
            .select_self_fields()
            .with_id_is(blob_id)
            .limit(1)
            .comment("what: query registry asset metadata")
            .purpose("why: resolve package content and integrity records")
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

            let asset = asset
                .audit_as("Updating existing asset record")
                .save_with(ctx)
                .await
                .map_err(|e| anyhow!("Failed to update asset: {}", e))?;

            return Ok(asset);
        }

        let mut asset = Q::assets()
            .comment("what: create registry asset metadata")
            .purpose("why: persist package content and integrity records")
            .new_entity(ctx);
        asset.update_content_repository_id(content_repo_id);
        asset.update_component_id(component_id.unwrap_or(0));
        asset.update_asset_blob_id(asset_blob_id);
        asset.update_path(path);
        asset.update_kind(kind);

        let asset = asset
            .audit_as("Creating new asset record")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save asset: {}", e))?;

        Ok(asset)
    }
}
