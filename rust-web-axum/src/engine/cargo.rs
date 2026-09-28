use anyhow::{anyhow, Result};
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::cargo::{CargoIndexConfig, CargoIndexRecord};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct CargoEngine;

impl CargoEngine {
    pub async fn get_config(repo_url: &str) -> CargoIndexConfig {
        CargoIndexConfig {
            dl: format!(
                "{}/api/v1/crates/{{crate}}/{{version}}/download",
                repo_url.trim_end_matches('/')
            ),
            api: repo_url.trim_end_matches('/').to_string(),
            auth_required: true,
        }
    }

    pub async fn upload_crate(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        crate_name: &str,
        version: &str,
        publish_metadata: &serde_json::Value,
        crate_data: &[u8],
    ) -> Result<()> {
        let content_repo =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "cargo").await?;
        let blob_info = blobstore.create_blob(crate_data).await?;

        let asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            "application/gzip",
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;

        let comp = ComponentService::find_or_create(
            ctx,
            content_repo.id(),
            "",
            crate_name,
            version,
            "crate",
        )
        .await?;

        let dl_path = format!("/api/v1/crates/{}/{}/download", crate_name, version);
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            Some(comp.id()),
            asset_blob.id(),
            &dl_path,
            "crate",
        )
        .await?;

        // Preserve the Publish API metadata in its sparse-index form. Cargo
        // cannot resolve a crate from an entry that drops its dependencies.
        let record = CargoIndexRecord::from_publish_metadata(
            publish_metadata,
            blob_info.checksums.sha256.clone(),
        )?;
        let index_data = serde_json::to_vec(&record)?;
        let index_blob = blobstore.create_blob(&index_data).await?;
        let index_asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &index_blob.blob_ref,
            index_blob.size,
            "application/json",
            &index_blob.checksums.sha1,
            &index_blob.checksums.sha256,
            &index_blob.checksums.md5,
        )
        .await?;
        let index_path = format!("/cargo/index-record/{crate_name}/{version}");
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            Some(comp.id()),
            index_asset_blob.id(),
            &index_path,
            "index",
        )
        .await?;

        Ok(())
    }

    pub async fn get_sparse_index(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        crate_name: &str,
    ) -> Result<Option<String>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let comps =
            ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let matching: Vec<_> = comps
            .into_iter()
            .filter(|c| c.name() == crate_name)
            .collect();

        if matching.is_empty() {
            return Ok(None);
        }

        let mut lines = Vec::new();
        for c in matching {
            let download_path = format!(
                "/api/v1/crates/{}/{}/download",
                crate_name,
                c.version_name()
            );
            let checksum =
                match AssetService::find_by_path(ctx, content_repo.id(), &download_path).await? {
                    Some(asset) => AssetService::get_asset_blob(ctx, asset.asset_blob_id())
                        .await?
                        .map(|blob| blob.sha256_checksum().to_string())
                        .unwrap_or_default(),
                    None => String::new(),
                };
            let index_path = format!("/cargo/index-record/{crate_name}/{}", c.version_name());
            let asset = AssetService::find_by_path(ctx, content_repo.id(), &index_path)
                .await?
                .ok_or_else(|| anyhow!(
                    "Cargo index metadata missing for {crate_name}@{}; republish or migrate this legacy artifact before consumption",
                    c.version_name()
                ))?;
            let index_blob = AssetService::get_asset_blob(ctx, asset.asset_blob_id())
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "Cargo index blob missing for {crate_name}@{}",
                        c.version_name()
                    )
                })?;
            let index_data = blobstore.read_blob(&index_blob.blob_ref()).await?;
            let record: CargoIndexRecord = serde_json::from_slice(&index_data)?;
            anyhow::ensure!(
                record.name == crate_name
                    && record.vers == c.version_name()
                    && record.cksum == checksum,
                "Cargo index metadata disagrees with archive for {crate_name}@{}",
                c.version_name()
            );
            lines.push(serde_json::to_string(&record)?);
        }

        Ok(Some(lines.join("\n")))
    }

    pub async fn get_crate_tarball(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        crate_name: &str,
        version: &str,
    ) -> Result<Option<Bytes>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let path = format!("/api/v1/crates/{}/{}/download", crate_name, version);
        let asset = match AssetService::find_by_path(ctx, content_repo.id(), &path).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        let asset_blob = match AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        match blobstore.read_blob(&asset_blob.blob_ref()).await {
            Ok(data) => Ok(Some(data)),
            Err(_) => Ok(None),
        }
    }
}
