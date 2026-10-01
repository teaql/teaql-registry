use anyhow::{bail, Context, Result};
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::dart::{
    dart_archive_sha256, extract_dart_pubspec, validate_dart_package_name, DartPackageVersion,
    DartPackageVersions, StoredDartPackage,
};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct DartEngine;

impl DartEngine {
    fn archive_path(name: &str, version: &str) -> String {
        format!("/packages/{name}/versions/{version}.tar.gz")
    }

    fn metadata_path(name: &str, version: &str) -> String {
        format!("/packages/{name}/versions/{version}.json")
    }

    async fn read_asset(
        ctx: &ServiceRuntime,
        blobstore: &dyn BlobStore,
        content_repository_id: u64,
        path: &str,
    ) -> Result<Option<Bytes>> {
        let Some(asset) =
            AssetService::find_by_path_with_blob(ctx, content_repository_id, path).await?
        else {
            return Ok(None);
        };
        let Some(blob) = asset.blob() else {
            return Ok(None);
        };
        Ok(Some(blobstore.read_blob(blob.blob_ref()).await?))
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
        archive: &[u8],
    ) -> Result<StoredDartPackage> {
        if repo.write_policy_is_read_only() {
            bail!("Repository is read only");
        }
        let (name, version, pubspec) = extract_dart_pubspec(archive)?;
        let content_repository =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "dart").await?;
        let archive_path = Self::archive_path(&name, &version);
        if AssetService::find_by_path(ctx, content_repository.id(), &archive_path)
            .await?
            .is_some()
        {
            bail!("package {name} {version} already exists");
        }
        let component = ComponentService::find_or_create(
            ctx,
            content_repository.id(),
            "dart",
            &name,
            &version,
            "dart-package",
        )
        .await?;
        let stored = StoredDartPackage {
            name: name.clone(),
            version: version.clone(),
            pubspec,
            archive_sha256: dart_archive_sha256(archive),
            published_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            component.id(),
            &archive_path,
            "dart-archive",
            "application/octet-stream",
            archive,
        )
        .await?;
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            component.id(),
            &Self::metadata_path(&name, &version),
            "dart-package-metadata",
            "application/json",
            &serde_json::to_vec(&stored)?,
        )
        .await?;
        Ok(stored)
    }

    pub async fn package_versions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        base_url: &str,
        name: &str,
    ) -> Result<Option<DartPackageVersions>> {
        validate_dart_package_name(name)?;
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(None);
        };
        let mut stored_versions = Vec::new();
        for component in ComponentService::list_by_repository(ctx, content_repository.id())
            .await?
            .into_iter()
            .filter(|component| component.kind() == "dart-package" && component.name() == name)
        {
            let version = component.version_name();
            let Some(bytes) = Self::read_asset(
                ctx,
                blobstore,
                content_repository.id(),
                &Self::metadata_path(name, version),
            )
            .await?
            else {
                continue;
            };
            let stored: StoredDartPackage =
                serde_json::from_slice(&bytes).context("invalid stored Dart package metadata")?;
            stored_versions.push(stored);
        }
        if stored_versions.is_empty() {
            return Ok(None);
        }
        stored_versions.sort_by(|left, right| {
            semver::Version::parse(&left.version)
                .ok()
                .cmp(&semver::Version::parse(&right.version).ok())
        });
        let versions = stored_versions
            .into_iter()
            .map(|stored| DartPackageVersion {
                archive_url: format!(
                    "{}/packages/{}/versions/{}.tar.gz",
                    base_url.trim_end_matches('/'),
                    stored.name,
                    stored.version
                ),
                version: stored.version,
                archive_sha256: stored.archive_sha256,
                pubspec: stored.pubspec,
            })
            .collect::<Vec<_>>();
        let latest = versions
            .last()
            .cloned()
            .context("Dart package has no versions")?;
        Ok(Some(DartPackageVersions {
            name: name.to_string(),
            latest,
            versions,
        }))
    }

    pub async fn archive(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        version: &str,
    ) -> Result<Option<Bytes>> {
        validate_dart_package_name(name)?;
        semver::Version::parse(version)?;
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(None);
        };
        Self::read_asset(
            ctx,
            blobstore,
            content_repository.id(),
            &Self::archive_path(name, version),
        )
        .await
    }
}
