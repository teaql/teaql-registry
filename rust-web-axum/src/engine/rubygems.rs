use anyhow::{bail, Context, Result};
use bytes::Bytes;
use md5::{Digest as _, Md5};
use sha2::Sha256;
use std::collections::BTreeMap;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::rubygems::{extract_rubygem_metadata, RubyGemMetadata};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct RubyGemsEngine;

impl RubyGemsEngine {
    fn gem_path(filename: &str) -> String {
        format!("/gems/{filename}")
    }

    fn metadata_path(name: &str, version_platform: &str) -> String {
        format!("/metadata/{name}-{version_platform}.json")
    }

    async fn read_asset(
        ctx: &ServiceRuntime,
        blobstore: &dyn BlobStore,
        content_repository_id: u64,
        path: &str,
    ) -> Result<Option<Bytes>> {
        let Some(asset) = AssetService::find_by_path(ctx, content_repository_id, path).await?
        else {
            return Ok(None);
        };
        let Some(blob) = AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? else {
            return Ok(None);
        };
        Ok(Some(blobstore.read_blob(&blob.blob_ref()).await?))
    }

    #[allow(clippy::too_many_arguments)]
    async fn persist_asset(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        content_repository_id: u64,
        component_id: u64,
        path: &str,
        kind: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<()> {
        let info = blobstore.create_blob(data).await?;
        let blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &info.blob_ref,
            info.size,
            content_type,
            &info.checksums.sha1,
            &info.checksums.sha256,
            &info.checksums.md5,
        )
        .await?;
        AssetService::create(
            ctx,
            content_repository_id,
            component_id,
            blob.id(),
            path,
            kind,
        )
        .await?;
        Ok(())
    }

    pub async fn publish(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        gem: &[u8],
    ) -> Result<RubyGemMetadata> {
        if repo.write_policy_is_read_only() {
            bail!("Repository is read only");
        }
        let mut metadata = extract_rubygem_metadata(gem)?;
        metadata.sha256 = hex::encode(Sha256::digest(gem));
        metadata.published_at =
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let content_repository =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "rubygems").await?;
        let gem_path = Self::gem_path(&metadata.filename());
        if AssetService::find_by_path(ctx, content_repository.id(), &gem_path)
            .await?
            .is_some()
        {
            bail!("gem {} already exists", metadata.filename());
        }
        let component = ComponentService::find_or_create(
            ctx,
            content_repository.id(),
            &metadata.platform,
            &metadata.name,
            &metadata.version,
            "ruby-gem",
        )
        .await?;
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            component.id(),
            &gem_path,
            "ruby-gem",
            "application/octet-stream",
            gem,
        )
        .await?;
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            component.id(),
            &Self::metadata_path(&metadata.name, &metadata.version_platform()),
            "rubygem-metadata",
            "application/json",
            &serde_json::to_vec(&metadata)?,
        )
        .await?;
        Ok(metadata)
    }

    pub async fn gem(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        filename: &str,
    ) -> Result<Option<Bytes>> {
        if filename.contains('/') || filename.contains("..") || !filename.ends_with(".gem") {
            bail!("invalid gem filename");
        }
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(None);
        };
        Self::read_asset(
            ctx,
            blobstore,
            content_repository.id(),
            &Self::gem_path(filename),
        )
        .await
    }

    async fn all_metadata(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
    ) -> Result<Vec<RubyGemMetadata>> {
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(Vec::new());
        };
        let mut metadata = Vec::new();
        for component in ComponentService::list_by_repository(ctx, content_repository.id())
            .await?
            .into_iter()
            .filter(|component| component.kind() == "ruby-gem")
        {
            let platform = component.namespace();
            let version_platform = if platform.is_empty() || platform == "ruby" {
                component.version_name()
            } else {
                format!("{}-{platform}", component.version_name())
            };
            let Some(bytes) = Self::read_asset(
                ctx,
                blobstore,
                content_repository.id(),
                &Self::metadata_path(&component.name(), &version_platform),
            )
            .await?
            else {
                continue;
            };
            metadata
                .push(serde_json::from_slice(&bytes).context("invalid stored RubyGem metadata")?);
        }
        Ok(metadata)
    }

    pub async fn compact_info(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
    ) -> Result<Option<String>> {
        let mut matching = Self::all_metadata(ctx, repo, blobstore)
            .await?
            .into_iter()
            .filter(|metadata| metadata.name == name)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Ok(None);
        }
        matching.sort_by_key(RubyGemMetadata::version_platform);
        let mut output = String::from("---\n");
        for metadata in matching {
            output.push_str(&metadata.compact_info_line());
            output.push('\n');
        }
        Ok(Some(output))
    }

    pub async fn compact_versions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
    ) -> Result<String> {
        let all = Self::all_metadata(ctx, repo, blobstore).await?;
        let mut grouped = BTreeMap::<String, Vec<RubyGemMetadata>>::new();
        for metadata in all {
            grouped
                .entry(metadata.name.clone())
                .or_default()
                .push(metadata);
        }
        let mut output = format!(
            "created_at: {}\n---\n",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        );
        for (name, mut versions) in grouped {
            versions.sort_by_key(RubyGemMetadata::version_platform);
            let info = format!(
                "---\n{}",
                versions
                    .iter()
                    .map(|metadata| format!("{}\n", metadata.compact_info_line()))
                    .collect::<String>()
            );
            let info_md5 = hex::encode(Md5::digest(info.as_bytes()));
            output.push_str(&format!(
                "{} {} {}\n",
                name,
                versions
                    .iter()
                    .map(RubyGemMetadata::version_platform)
                    .collect::<Vec<_>>()
                    .join(","),
                info_md5
            ));
        }
        Ok(output)
    }

    pub async fn names(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
    ) -> Result<String> {
        let mut names = Self::all_metadata(ctx, repo, blobstore)
            .await?
            .into_iter()
            .map(|metadata| metadata.name)
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        Ok(format!(
            "---\n{}",
            names
                .into_iter()
                .map(|name| format!("{name}\n"))
                .collect::<String>()
        ))
    }
}
