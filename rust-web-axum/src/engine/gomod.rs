use anyhow::Result;
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct GoModEngine;

impl GoModEngine {
    pub async fn upload_artifact(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        module: &str,
        version: &str,
        ext: &str,
        data: &[u8],
    ) -> Result<()> {
        let content_repo =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "gomod").await?;
        let blob_info = blobstore.create_blob(data).await?;

        let ct = match ext {
            "zip" => "application/zip",
            "info" => "application/json",
            "mod" => "text/plain; charset=utf-8",
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

        let comp =
            ComponentService::find_or_create(ctx, content_repo.id(), "", module, version, ext)
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

        let comps =
            ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let mut vers: Vec<String> = comps
            .into_iter()
            .filter(|c| c.name() == module && !c.version_name().is_empty())
            .map(|c| c.version_name().to_string())
            .collect();

        vers.sort();
        vers.dedup();
        Ok(vers.join("\n"))
    }

    pub async fn get_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let clean_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        let asset = match AssetService::find_by_path_with_blob(ctx, content_repo.id(), &clean_path)
            .await?
        {
            Some(a) => a,
            None => return Ok(None),
        };
        let blob = match asset.blob() {
            Some(blob) => blob,
            None => return Ok(None),
        };

        match blobstore.read_blob(blob.blob_ref()).await {
            Ok(data) => Ok(Some((data, blob.content_type().to_owned()))),
            Err(_) => Ok(None),
        }
    }
}
