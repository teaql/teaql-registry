use anyhow::Result;
use bytes::Bytes;
use nexus_repository_service_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::S3BlobStore;
use crate::format::gomod::GoModuleVersionInfo;
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct GoModEngine;

impl GoModEngine {
    pub async fn upload_artifact(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &S3BlobStore,
        module: &str,
        version: &str,
        ext: &str,
        data: &[u8],
    ) -> Result<()> {
        let content_repo = RepositoryService::ensure_content_repository(ctx, repo.id(), "gomod").await?;
        let blob_info = blobstore.create_blob(data).await?;

        let ct = match ext {
            "zip" => "application/zip",
            "mod" | "info" => "text/plain; charset=utf-8",
            _ => "application/octet-stream",
        };

        let asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            ct,
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;

        let comp = ComponentService::find_or_create(
            ctx,
            content_repo.id(),
            "",
            module,
            version,
            ext,
        )
        .await?;

        let path = format!("/{}/@v/{}.{}", module, version, ext);
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            Some(comp.id()),
            asset_blob.id(),
            &path,
            ext,
        )
        .await?;

        Ok(())
    }

    pub async fn list_versions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        module: &str,
    ) -> Result<String> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(String::new()),
        };

        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let mut vers: Vec<String> = comps
            .into_iter()
            .filter(|c| c.name() == module && !c.version_name().is_empty())
            .map(|c| c.version_name().to_string())
            .collect();

        vers.sort();
        vers.dedup();
        Ok(vers.join("\n"))
    }

    pub async fn get_version_info(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        module: &str,
        version: &str,
    ) -> Result<Option<GoModuleVersionInfo>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        if comps.iter().any(|c| c.name() == module && c.version_name() == version) {
            Ok(Some(GoModuleVersionInfo {
                version: version.to_string(),
                time: "2026-08-14T10:00:00Z".to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &S3BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let clean_path = if path.starts_with('/') { path.to_string() } else { format!("/{}", path) };
        let asset = match AssetService::find_by_path(ctx, content_repo.id(), &clean_path).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        let asset_blob = match AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        match blobstore.read_blob(&asset_blob.blob_ref()).await {
            Ok(data) => Ok(Some((data, asset_blob.content_type().to_string()))),
            Err(_) => Ok(None),
        }
    }
}
