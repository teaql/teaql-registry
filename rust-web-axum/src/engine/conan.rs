use anyhow::{bail, Context, Result};
use bytes::Bytes;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use teaql_registry_core::{Component, RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::conan::{
    validate_conan_coordinate, validate_conan_file_path, ConanFileSnapshot, ConanRevision,
    ConanRevisions,
};
use crate::services::{AssetService, ComponentService, RepositoryService};

const CONAN_TIMESTAMP: &str = "2026-01-01T00:00:00.000000Z";

pub struct ConanEngine;

impl ConanEngine {
    fn recipe_namespace(user: &str, channel: &str) -> String {
        format!("{user}/{channel}")
    }

    fn recipe_component_version(version: &str, revision: &str) -> String {
        format!("{version}#{revision}")
    }

    fn recipe_prefix(
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        revision: &str,
    ) -> String {
        format!("/recipes/{name}/{version}/{user}/{channel}/{revision}/")
    }

    fn package_namespace(
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
    ) -> String {
        format!("{user}/{channel}/{name}/{version}#{recipe_revision}")
    }

    #[allow(clippy::too_many_arguments)]
    fn package_prefix(
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
        package_id: &str,
        package_revision: &str,
    ) -> String {
        format!(
            "/packages/{name}/{version}/{user}/{channel}/{recipe_revision}/{package_id}/{package_revision}/"
        )
    }

    fn validate_reference(
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        revision: Option<&str>,
    ) -> Result<()> {
        validate_conan_coordinate(name, "name")?;
        validate_conan_coordinate(version, "version")?;
        validate_conan_coordinate(user, "user")?;
        validate_conan_coordinate(channel, "channel")?;
        if let Some(revision) = revision {
            validate_conan_coordinate(revision, "revision")?;
        }
        Ok(())
    }

    async fn content_repository(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
    ) -> Result<teaql_registry_core::ContentRepository> {
        RepositoryService::ensure_content_repository(ctx, repo.id(), "conan").await
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
    async fn persist_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        component: &Component,
        content_repository_id: u64,
        path: &str,
        kind: &str,
        data: &[u8],
    ) -> Result<()> {
        if repo.write_policy_is_read_only() {
            bail!("Repository is read only");
        }
        if repo.write_policy_is_allow_once()
            && AssetService::find_by_path(ctx, content_repository_id, path)
                .await?
                .is_some()
        {
            bail!("Conan file already exists");
        }
        let info = blobstore.create_blob(data).await?;
        let blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &info.blob_ref,
            info.size,
            "application/octet-stream",
            &info.checksums.sha1,
            &info.checksums.sha256,
            &info.checksums.md5,
        )
        .await?;
        AssetService::upsert_asset(
            ctx,
            content_repository_id,
            Some(component.id()),
            blob.id(),
            path,
            kind,
        )
        .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn put_recipe_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        revision: &str,
        file_path: &str,
        data: &[u8],
    ) -> Result<()> {
        Self::validate_reference(name, version, user, channel, Some(revision))?;
        validate_conan_file_path(file_path)?;
        let content_repository = Self::content_repository(ctx, repo).await?;
        let component = ComponentService::find_or_create(
            ctx,
            content_repository.id(),
            &Self::recipe_namespace(user, channel),
            name,
            &Self::recipe_component_version(version, revision),
            "conan-recipe",
        )
        .await?;
        let path = format!(
            "{}{file_path}",
            Self::recipe_prefix(name, version, user, channel, revision)
        );
        Self::persist_file(
            ctx,
            repo,
            blobstore,
            &component,
            content_repository.id(),
            &path,
            "conan-recipe-file",
            data,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn put_package_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
        package_id: &str,
        package_revision: &str,
        file_path: &str,
        data: &[u8],
    ) -> Result<()> {
        Self::validate_reference(name, version, user, channel, Some(recipe_revision))?;
        validate_conan_coordinate(package_id, "package ID")?;
        validate_conan_coordinate(package_revision, "package revision")?;
        validate_conan_file_path(file_path)?;
        let content_repository = Self::content_repository(ctx, repo).await?;
        let component = ComponentService::find_or_create(
            ctx,
            content_repository.id(),
            &Self::package_namespace(name, version, user, channel, recipe_revision),
            package_id,
            package_revision,
            "conan-package",
        )
        .await?;
        let path = format!(
            "{}{file_path}",
            Self::package_prefix(
                name,
                version,
                user,
                channel,
                recipe_revision,
                package_id,
                package_revision,
            )
        );
        Self::persist_file(
            ctx,
            repo,
            blobstore,
            &component,
            content_repository.id(),
            &path,
            "conan-package-file",
            data,
        )
        .await
    }

    async fn components(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
    ) -> Result<Vec<Component>> {
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(Vec::new());
        };
        ComponentService::list_by_repository(ctx, content_repository.id()).await
    }

    pub async fn search_recipes(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        pattern: Option<&str>,
    ) -> Result<Vec<String>> {
        let pattern = pattern.unwrap_or("*");
        let mut results = BTreeSet::new();
        for component in Self::components(ctx, repo)
            .await?
            .into_iter()
            .filter(|component| component.kind() == "conan-recipe")
        {
            let component_version = component.version_name();
            let Some((version, _)) = component_version.split_once('#') else {
                continue;
            };
            let component_namespace = component.namespace();
            let Some((user, channel)) = component_namespace.split_once('/') else {
                continue;
            };
            let reference = format!("{}/{version}@{user}/{channel}", component.name());
            if Self::wildcard_match(pattern, &reference)
                || Self::wildcard_match(pattern, &format!("{}/{version}", component.name()))
            {
                results.insert(reference);
            }
        }
        Ok(results.into_iter().collect())
    }

    fn wildcard_match(pattern: &str, value: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        let parts = pattern.split('*').collect::<Vec<_>>();
        if parts.len() == 1 {
            return pattern == value;
        }
        let mut remaining = value;
        for (index, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if index == 0 && !pattern.starts_with('*') {
                let Some(rest) = remaining.strip_prefix(part) else {
                    return false;
                };
                remaining = rest;
            } else if let Some(position) = remaining.find(part) {
                remaining = &remaining[position + part.len()..];
            } else {
                return false;
            }
        }
        pattern.ends_with('*') || remaining.is_empty()
    }

    pub async fn recipe_revisions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
    ) -> Result<ConanRevisions> {
        Self::validate_reference(name, version, user, channel, None)?;
        let mut revisions = Self::components(ctx, repo)
            .await?
            .into_iter()
            .filter(|component| {
                component.kind() == "conan-recipe"
                    && component.name() == name
                    && component.namespace() == Self::recipe_namespace(user, channel)
            })
            .filter_map(|component| {
                let component_version = component.version_name();
                let (stored_version, revision) = component_version.split_once('#')?;
                (stored_version == version).then(|| (component.id(), revision.to_string()))
            })
            .collect::<Vec<_>>();
        revisions.sort_by_key(|(id, _)| std::cmp::Reverse(*id));
        Ok(ConanRevisions {
            revisions: revisions
                .into_iter()
                .map(|(_, revision)| ConanRevision {
                    revision,
                    time: CONAN_TIMESTAMP.to_string(),
                })
                .collect(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn find_recipe_component(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        revision: &str,
    ) -> Result<Option<Component>> {
        Self::validate_reference(name, version, user, channel, Some(revision))?;
        Ok(Self::components(ctx, repo)
            .await?
            .into_iter()
            .find(|component| {
                component.kind() == "conan-recipe"
                    && component.name() == name
                    && component.namespace() == Self::recipe_namespace(user, channel)
                    && component.version_name() == Self::recipe_component_version(version, revision)
            }))
    }

    async fn snapshot_for_component(
        ctx: &ServiceRuntime,
        component: &Component,
        prefix: &str,
    ) -> Result<ConanFileSnapshot> {
        let files = AssetService::list_by_component(ctx, component.id())
            .await?
            .into_iter()
            .filter_map(|asset| {
                asset
                    .path()
                    .strip_prefix(prefix)
                    .map(|path| (path.to_string(), json!({})))
            })
            .collect::<BTreeMap<_, _>>();
        Ok(ConanFileSnapshot { files })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn recipe_snapshot(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        revision: &str,
    ) -> Result<Option<ConanFileSnapshot>> {
        let Some(component) =
            Self::find_recipe_component(ctx, repo, name, version, user, channel, revision).await?
        else {
            return Ok(None);
        };
        Self::snapshot_for_component(
            ctx,
            &component,
            &Self::recipe_prefix(name, version, user, channel, revision),
        )
        .await
        .map(Some)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn recipe_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        revision: &str,
        file_path: &str,
    ) -> Result<Option<Bytes>> {
        Self::validate_reference(name, version, user, channel, Some(revision))?;
        validate_conan_file_path(file_path)?;
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(None);
        };
        Self::read_asset(
            ctx,
            blobstore,
            content_repository.id(),
            &format!(
                "{}{file_path}",
                Self::recipe_prefix(name, version, user, channel, revision)
            ),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn matching_package_components(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
        package_id: Option<&str>,
    ) -> Result<Vec<Component>> {
        Self::validate_reference(name, version, user, channel, Some(recipe_revision))?;
        if let Some(package_id) = package_id {
            validate_conan_coordinate(package_id, "package ID")?;
        }
        let namespace = Self::package_namespace(name, version, user, channel, recipe_revision);
        Ok(Self::components(ctx, repo)
            .await?
            .into_iter()
            .filter(|component| {
                component.kind() == "conan-package"
                    && component.namespace() == namespace
                    && package_id.map_or(true, |package_id| component.name() == package_id)
            })
            .collect())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn package_revisions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
        package_id: &str,
    ) -> Result<ConanRevisions> {
        let mut revisions = Self::matching_package_components(
            ctx,
            repo,
            name,
            version,
            user,
            channel,
            recipe_revision,
            Some(package_id),
        )
        .await?
        .into_iter()
        .map(|component| (component.id(), component.version_name()))
        .collect::<Vec<_>>();
        revisions.sort_by_key(|(id, _)| std::cmp::Reverse(*id));
        Ok(ConanRevisions {
            revisions: revisions
                .into_iter()
                .map(|(_, revision)| ConanRevision {
                    revision,
                    time: CONAN_TIMESTAMP.to_string(),
                })
                .collect(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn find_package_component(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
        package_id: &str,
        package_revision: &str,
    ) -> Result<Option<Component>> {
        Ok(Self::matching_package_components(
            ctx,
            repo,
            name,
            version,
            user,
            channel,
            recipe_revision,
            Some(package_id),
        )
        .await?
        .into_iter()
        .find(|component| component.version_name() == package_revision))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn package_snapshot(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
        package_id: &str,
        package_revision: &str,
    ) -> Result<Option<ConanFileSnapshot>> {
        let Some(component) = Self::find_package_component(
            ctx,
            repo,
            name,
            version,
            user,
            channel,
            recipe_revision,
            package_id,
            package_revision,
        )
        .await?
        else {
            return Ok(None);
        };
        Self::snapshot_for_component(
            ctx,
            &component,
            &Self::package_prefix(
                name,
                version,
                user,
                channel,
                recipe_revision,
                package_id,
                package_revision,
            ),
        )
        .await
        .map(Some)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn package_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
        package_id: &str,
        package_revision: &str,
        file_path: &str,
    ) -> Result<Option<Bytes>> {
        Self::validate_reference(name, version, user, channel, Some(recipe_revision))?;
        validate_conan_coordinate(package_id, "package ID")?;
        validate_conan_coordinate(package_revision, "package revision")?;
        validate_conan_file_path(file_path)?;
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(None);
        };
        Self::read_asset(
            ctx,
            blobstore,
            content_repository.id(),
            &format!(
                "{}{file_path}",
                Self::package_prefix(
                    name,
                    version,
                    user,
                    channel,
                    recipe_revision,
                    package_id,
                    package_revision,
                )
            ),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn package_search(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
        version: &str,
        user: &str,
        channel: &str,
        recipe_revision: &str,
    ) -> Result<Value> {
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(json!({}));
        };
        let components = Self::matching_package_components(
            ctx,
            repo,
            name,
            version,
            user,
            channel,
            recipe_revision,
            None,
        )
        .await?;
        let mut packages = Map::new();
        for component in components {
            if packages.contains_key(&component.name()) {
                continue;
            }
            let path = format!(
                "{}conaninfo.txt",
                Self::package_prefix(
                    name,
                    version,
                    user,
                    channel,
                    recipe_revision,
                    &component.name(),
                    &component.version_name(),
                )
            );
            let content = Self::read_asset(ctx, blobstore, content_repository.id(), &path)
                .await?
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_default();
            packages.insert(component.name(), json!({ "content": content }));
        }
        Ok(Value::Object(packages))
    }

    pub fn latest(revisions: &ConanRevisions) -> Result<ConanRevision> {
        revisions
            .revisions
            .first()
            .cloned()
            .context("Conan revision not found")
    }
}
