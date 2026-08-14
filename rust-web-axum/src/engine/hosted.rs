use anyhow::{anyhow, Result};
use bytes::Bytes;
use nexus_repository_service_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::S3BlobStore;
use crate::format::maven::parse_maven_path;
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct HostedEngine;

impl HostedEngine {
    pub async fn handle_get(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &S3BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let asset = match AssetService::find_by_path(ctx, content_repo.id(), path).await? {
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

    pub async fn handle_put(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &S3BlobStore,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<()> {
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

        // Check ALLOW_ONCE write policy
        if repo.write_policy_is_allow_once() {
            let is_metadata = path.ends_with("maven-metadata.xml") || path.contains("maven-metadata.xml.");
            let is_snapshot = path.to_uppercase().contains("-SNAPSHOT");
            let is_checksum = path.ends_with(".sha1") || path.ends_with(".md5") || path.ends_with(".sha256") || path.ends_with(".sha512");
            if !is_metadata && !is_snapshot && !is_checksum {
                let existing = AssetService::find_by_path(ctx, content_repo.id(), path).await?;
                if existing.is_some() {
                    return Err(anyhow!("Repository does not allow updating assets: {}", path));
                }
            }
        }

        // Write binary to BlobStore
        let blob_info = blobstore.create_blob(data).await?;

        // Save AssetBlob in DB
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

        // Optional component coordinate if maven
        let mut component_id = None;
        let mut kind = "generic".to_string();

        if let Some(coords) = parse_maven_path(path) {
            kind = coords.extension.clone();
            if !coords.is_metadata && !coords.version.is_empty() {
                let comp = ComponentService::find_or_create(
                    ctx,
                    content_repo.id(),
                    &coords.group_id,
                    &coords.artifact_id,
                    &coords.version,
                    &coords.extension,
                )
                .await?;
                component_id = Some(comp.id());
            }
        }

        // Upsert Asset record in DB
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
}
