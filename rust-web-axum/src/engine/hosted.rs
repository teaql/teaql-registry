use anyhow::{anyhow, Result};
use bytes::Bytes;
use teaql_registry_core::{ContentRepository, RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::{BlobInfo, BlobStore, ByteStream};
use crate::format::maven::parse_maven_path;
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct HostedEngine;

impl HostedEngine {
    async fn prepare_write(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        path: &str,
    ) -> Result<ContentRepository> {
        if repo.write_policy_is_read_only() {
            return Err(anyhow!("Repository is read only"));
        }
        let content_repo = RepositoryService::ensure_content_repository(
            ctx,
            repo.id(),
            if repo.recipe_name().contains("maven") {
                "maven2"
            } else {
                "raw"
            },
        )
        .await?;
        if repo.write_policy_is_allow_once() {
            let is_metadata =
                path.ends_with("maven-metadata.xml") || path.contains("maven-metadata.xml.");
            let is_snapshot = path.to_uppercase().contains("-SNAPSHOT");
            let is_checksum = path.ends_with(".sha1")
                || path.ends_with(".md5")
                || path.ends_with(".sha256")
                || path.ends_with(".sha512");
            if !is_metadata
                && !is_snapshot
                && !is_checksum
                && AssetService::find_by_path(ctx, content_repo.id(), path)
                    .await?
                    .is_some()
            {
                return Err(anyhow!(
                    "Repository does not allow updating assets: {}",
                    path
                ));
            }
        }
        Ok(content_repo)
    }

    async fn persist_blob_metadata(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        content_repo: &ContentRepository,
        path: &str,
        content_type: &str,
        blob_info: BlobInfo,
    ) -> Result<()> {
        let asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            content_type,
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;

        let mut component_id = None;
        let mut kind = "generic".to_string();
        if let Some(coords) = parse_maven_path(path) {
            kind = coords.extension.clone();
            if !coords.is_metadata && !coords.version.is_empty() {
                let component = ComponentService::find_or_create(
                    ctx,
                    content_repo.id(),
                    &coords.group_id,
                    &coords.artifact_id,
                    &coords.version,
                    &coords.extension,
                )
                .await?;
                component_id = Some(component.id());
            }
        }
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            component_id,
            asset_blob.id(),
            path,
            &kind,
        )
        .await?;
        Ok(())
    }

    pub async fn handle_get(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let asset = match AssetService::find_by_path_with_blob(ctx, content_repo.id(), path).await?
        {
            Some(a) => a,
            None => return Ok(None),
        };
        let Some(blob) = asset.blob() else {
            return Ok(None);
        };

        match blobstore.read_blob(blob.blob_ref()).await {
            Ok(data) => Ok(Some((data, blob.content_type().to_owned()))),
            Err(_) => Ok(None),
        }
    }

    pub async fn handle_get_stream(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(ByteStream, String, i64)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(repository) => repository,
            None => return Ok(None),
        };
        let asset = match AssetService::find_by_path_with_blob(ctx, content_repo.id(), path).await?
        {
            Some(asset) => asset,
            None => return Ok(None),
        };
        let Some(blob) = asset.blob() else {
            return Ok(None);
        };
        match blobstore.read_blob_stream(blob.blob_ref()).await {
            Ok(stream) => Ok(Some((
                stream,
                blob.content_type().to_owned(),
                blob.blob_size(),
            ))),
            Err(_) => Ok(None),
        }
    }

    pub async fn handle_put(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<()> {
        let content_repo = Self::prepare_write(ctx, repo, path).await?;
        let blob_info = blobstore.create_blob(data).await?;
        Self::persist_blob_metadata(ctx, repo, &content_repo, path, content_type, blob_info).await
    }

    pub async fn handle_put_stream(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
        stream: ByteStream,
        content_type: &str,
    ) -> Result<i64> {
        let content_repo = Self::prepare_write(ctx, repo, path).await?;
        let blob_info = blobstore.create_blob_from_stream(stream).await?;
        let size = blob_info.size;
        Self::persist_blob_metadata(ctx, repo, &content_repo, path, content_type, blob_info)
            .await?;
        Ok(size)
    }
}
