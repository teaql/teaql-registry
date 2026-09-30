use anyhow::{bail, Context, Result};
use bytes::Bytes;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::composer::{
    extract_composer_json, validate_composer_name, validate_composer_version, StoredComposerPackage,
};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct ComposerEngine;

impl ComposerEngine {
    fn is_development_version(version: &str) -> bool {
        version.starts_with("dev-") || version.ends_with("-dev")
    }

    fn archive_path(name: &str, version: &str) -> String {
        format!("/dist/{name}/{version}.zip")
    }

    fn metadata_path(name: &str, version: &str) -> String {
        format!("/metadata/{name}/{version}.json")
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
        expected_name: &str,
        version: &str,
        archive: &[u8],
    ) -> Result<StoredComposerPackage> {
        if repo.write_policy_is_read_only() {
            bail!("Repository is read only");
        }
        let (vendor, package_name) = validate_composer_name(expected_name)?;
        validate_composer_version(version)?;
        let mut manifest = extract_composer_json(archive)?;
        let declared_name = manifest
            .get("name")
            .and_then(Value::as_str)
            .context("composer.json does not declare name")?;
        if declared_name != expected_name {
            bail!("composer.json name {declared_name} does not match {expected_name}");
        }
        if let Some(declared_version) = manifest.get("version").and_then(Value::as_str) {
            if declared_version != version {
                bail!("composer.json version {declared_version} does not match {version}");
            }
        }
        manifest
            .as_object_mut()
            .context("composer.json must be an object")?
            .insert("version".to_string(), Value::String(version.to_string()));
        let stored = StoredComposerPackage {
            name: expected_name.to_string(),
            version: version.to_string(),
            sha256: hex::encode(Sha256::digest(archive)),
            package: manifest,
        };
        let content_repository =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "composer").await?;
        let archive_path = Self::archive_path(expected_name, version);
        if AssetService::find_by_path(ctx, content_repository.id(), &archive_path)
            .await?
            .is_some()
        {
            bail!("Composer package {expected_name} {version} already exists");
        }
        let component = ComponentService::find_or_create(
            ctx,
            content_repository.id(),
            vendor,
            package_name,
            version,
            "composer-package",
        )
        .await?;
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            component.id(),
            &archive_path,
            "composer-dist",
            "application/zip",
            archive,
        )
        .await?;
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            component.id(),
            &Self::metadata_path(expected_name, version),
            "composer-metadata",
            "application/json",
            &serde_json::to_vec(&stored)?,
        )
        .await?;
        Ok(stored)
    }

    async fn package_versions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
    ) -> Result<Vec<StoredComposerPackage>> {
        validate_composer_name(name)?;
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(Vec::new());
        };
        let (vendor, package_name) = name.split_once('/').context("invalid package name")?;
        let mut versions = Vec::new();
        for component in ComponentService::list_by_repository(ctx, content_repository.id())
            .await?
            .into_iter()
            .filter(|component| {
                component.kind() == "composer-package"
                    && component.namespace() == vendor
                    && component.name() == package_name
            })
        {
            let Some(bytes) = Self::read_asset(
                ctx,
                blobstore,
                content_repository.id(),
                &Self::metadata_path(name, &component.version_name()),
            )
            .await?
            else {
                continue;
            };
            versions
                .push(serde_json::from_slice(&bytes).context("invalid stored Composer metadata")?);
        }
        versions.sort_by(|left: &StoredComposerPackage, right| left.version.cmp(&right.version));
        Ok(versions)
    }

    pub async fn root_metadata(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
    ) -> Result<Value> {
        let names = if let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        {
            let mut names = ComponentService::list_by_repository(ctx, content_repository.id())
                .await?
                .into_iter()
                .filter(|component| component.kind() == "composer-package")
                .map(|component| format!("{}/{}", component.namespace(), component.name()))
                .collect::<Vec<_>>();
            names.sort();
            names.dedup();
            names
        } else {
            Vec::new()
        };
        Ok(json!({
            "packages": {},
            "metadata-url": format!("/repository/{}/composer/p2/%package%.json", repo.name()),
            "available-packages": names,
        }))
    }

    pub async fn p2_metadata(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        base_url: &str,
        development_only: bool,
    ) -> Result<Option<Value>> {
        let versions = Self::package_versions(ctx, repo, blobstore, name).await?;
        let packages = versions
            .into_iter()
            .filter(|stored| Self::is_development_version(&stored.version) == development_only)
            .map(|stored| {
                let mut package = stored.package.as_object().cloned().unwrap_or_default();
                package.insert("name".to_string(), Value::String(stored.name.clone()));
                package.insert("version".to_string(), Value::String(stored.version.clone()));
                package.insert(
                    "dist".to_string(),
                    json!({
                        "type": "zip",
                        "url": format!(
                            "{base_url}/repository/{}/composer/dist/{}/{}.zip",
                            repo.name(), stored.name, stored.version
                        ),
                        "reference": stored.sha256,
                        "shasum": "",
                    }),
                );
                Value::Object(package)
            })
            .collect::<Vec<_>>();
        if packages.is_empty() {
            return Ok(None);
        }
        let mut package_map = Map::new();
        package_map.insert(name.to_string(), Value::Array(packages));
        Ok(Some(json!({ "packages": package_map })))
    }

    pub async fn archive(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        version: &str,
    ) -> Result<Option<Bytes>> {
        validate_composer_name(name)?;
        validate_composer_version(version)?;
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
