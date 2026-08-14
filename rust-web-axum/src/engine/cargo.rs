use anyhow::Result;
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::cargo::{get_cargo_index_path, CargoIndexConfig, CargoIndexRecord};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct CargoEngine;

impl CargoEngine {
    pub async fn get_config(repo_url: &str) -> CargoIndexConfig {
        CargoIndexConfig {
            dl: format!("{}/api/v1/crates/{{crate}}/{{version}}/download", repo_url.trim_end_matches('/')),
            api: repo_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn upload_crate(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        crate_name: &str,
        version: &str,
        crate_data: &[u8],
    ) -> Result<()> {
        let content_repo = RepositoryService::ensure_content_repository(ctx, repo.id(), "cargo").await?;
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

        // Also record index entry
        let index_subpath = get_cargo_index_path(crate_name);
        let index_path = format!("/{}", index_subpath);
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            Some(comp.id()),
            asset_blob.id(),
            &index_path,
            "index",
        )
        .await?;

        Ok(())
    }

    pub async fn get_sparse_index(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        crate_name: &str,
    ) -> Result<Option<String>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let matching: Vec<_> = comps.into_iter().filter(|c| c.name() == crate_name).collect();

        if matching.is_empty() {
            return Ok(None);
        }

        let mut lines = Vec::new();
        for c in matching {
            let record = CargoIndexRecord {
                name: crate_name.to_string(),
                vers: c.version_name().to_string(),
                deps: Vec::new(),
                cksum: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
                features: serde_json::json!({}),
                yanked: false,
            };
            if let Ok(json_str) = serde_json::to_string(&record) {
                lines.push(json_str);
            }
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
