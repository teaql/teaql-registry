use anyhow::{anyhow, bail, Context, Result};
use base64::Engine as _;
use bytes::Bytes;
use sha2::Digest;
use std::collections::BTreeMap;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::swift::{
    extract_swift_manifests, validate_package_identity, validate_version, StoredSwiftRelease,
    SwiftManifest, SwiftPackageRelease, SwiftPackageReleases, SwiftReleaseMetadata,
    SwiftReleaseResource, SwiftReleaseSignature,
};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct SwiftEngine;

struct SwiftAsset<'a> {
    component_id: u64,
    path: &'a str,
    kind: &'a str,
    content_type: &'a str,
    data: &'a [u8],
}

impl SwiftEngine {
    fn archive_path(scope: &str, name: &str, version: &str) -> String {
        format!("/{scope}/{name}/{version}.zip")
    }

    fn release_path(scope: &str, name: &str, version: &str) -> String {
        format!("/{scope}/{name}/{version}/teaql-release.json")
    }

    fn manifest_path(scope: &str, name: &str, version: &str, filename: &str) -> String {
        format!("/{scope}/{name}/{version}/{filename}")
    }

    async fn content_repository_id(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
    ) -> Result<Option<u64>> {
        Ok(RepositoryService::get_content_repository(ctx, repo.id())
            .await?
            .map(|repository| repository.id()))
    }

    async fn read_asset(
        ctx: &ServiceRuntime,
        blobstore: &dyn BlobStore,
        content_repository_id: u64,
        path: &str,
    ) -> Result<Option<(Bytes, String, String)>> {
        let Some(asset) = AssetService::find_by_path(ctx, content_repository_id, path).await?
        else {
            return Ok(None);
        };
        let Some(blob) = AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? else {
            return Ok(None);
        };
        let bytes = blobstore.read_blob(&blob.blob_ref()).await?;
        Ok(Some((bytes, blob.content_type(), blob.sha256_checksum())))
    }

    async fn persist_asset(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        content_repository_id: u64,
        asset: SwiftAsset<'_>,
    ) -> Result<()> {
        let blob_info = blobstore.create_blob(asset.data).await?;
        let blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            asset.content_type,
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;
        AssetService::create(
            ctx,
            content_repository_id,
            asset.component_id,
            blob.id(),
            asset.path,
            asset.kind,
        )
        .await?;
        Ok(())
    }

    pub async fn release_exists(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        scope: &str,
        name: &str,
        version: &str,
    ) -> Result<bool> {
        let Some(content_repository_id) = Self::content_repository_id(ctx, repo).await? else {
            return Ok(false);
        };
        Ok(AssetService::find_by_path(
            ctx,
            content_repository_id,
            &Self::archive_path(scope, name, version),
        )
        .await?
        .is_some())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn publish_release(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        scope: &str,
        name: &str,
        version: &str,
        source_archive: &[u8],
        metadata: serde_json::Value,
        archive_signature: Option<(&str, &[u8])>,
    ) -> Result<()> {
        validate_package_identity(scope, name)?;
        validate_version(version)?;
        if repo.write_policy_is_read_only() {
            bail!("Repository is read only");
        }
        let manifests = extract_swift_manifests(source_archive)?;
        if Self::release_exists(ctx, repo, scope, name, version).await? {
            bail!("a release with version {version} already exists");
        }

        let content_repository =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "swift").await?;
        let component = ComponentService::find_or_create(
            ctx,
            content_repository.id(),
            &scope.to_ascii_lowercase(),
            &name.to_ascii_lowercase(),
            version,
            "swift-package",
        )
        .await?;

        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            SwiftAsset {
                component_id: component.id(),
                path: &Self::archive_path(scope, name, version),
                kind: "swift-source-archive",
                content_type: "application/zip",
                data: source_archive,
            },
        )
        .await?;

        for manifest in manifests {
            Self::persist_manifest(
                ctx,
                repo,
                blobstore,
                content_repository.id(),
                component.id(),
                scope,
                name,
                version,
                &manifest,
            )
            .await?;
        }

        let source_archive_signature =
            archive_signature.map(|(format, bytes)| SwiftReleaseSignature {
                signature_base64_encoded: base64::engine::general_purpose::STANDARD.encode(bytes),
                signature_format: format.to_string(),
            });
        let stored = StoredSwiftRelease {
            metadata,
            published_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            source_archive_signature,
        };
        let stored_json = serde_json::to_vec(&stored)?;
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            SwiftAsset {
                component_id: component.id(),
                path: &Self::release_path(scope, name, version),
                kind: "swift-release-metadata",
                content_type: "application/json",
                data: &stored_json,
            },
        )
        .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn persist_manifest(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        content_repository_id: u64,
        component_id: u64,
        scope: &str,
        name: &str,
        version: &str,
        manifest: &SwiftManifest,
    ) -> Result<()> {
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository_id,
            SwiftAsset {
                component_id,
                path: &Self::manifest_path(scope, name, version, &manifest.filename),
                kind: "swift-manifest",
                content_type: "text/x-swift",
                data: &manifest.contents,
            },
        )
        .await
    }

    pub async fn list_releases(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        base_url: &str,
        scope: &str,
        name: &str,
    ) -> Result<Option<SwiftPackageReleases>> {
        validate_package_identity(scope, name)?;
        let Some(content_repository_id) = Self::content_repository_id(ctx, repo).await? else {
            return Ok(None);
        };
        let mut matching: Vec<_> = ComponentService::list_by_repository(ctx, content_repository_id)
            .await?
            .into_iter()
            .filter(|component| {
                component.kind() == "swift-package"
                    && component.namespace().eq_ignore_ascii_case(scope)
                    && component.name().eq_ignore_ascii_case(name)
            })
            .collect();
        if matching.is_empty() {
            return Ok(None);
        }
        matching.sort_by(|left, right| {
            let left = semver::Version::parse(&left.version_name()).ok();
            let right = semver::Version::parse(&right.version_name()).ok();
            right.cmp(&left)
        });
        let releases = matching
            .into_iter()
            .map(|component| {
                let version = component.version_name();
                let url = format!(
                    "{}/{}/{}/{}",
                    base_url.trim_end_matches('/'),
                    scope,
                    name,
                    version
                );
                (version, SwiftPackageRelease { url })
            })
            .collect::<BTreeMap<_, _>>();
        Ok(Some(SwiftPackageReleases { releases }))
    }

    pub async fn get_release_metadata(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        scope: &str,
        name: &str,
        version: &str,
    ) -> Result<Option<SwiftReleaseMetadata>> {
        validate_package_identity(scope, name)?;
        validate_version(version)?;
        let Some(content_repository_id) = Self::content_repository_id(ctx, repo).await? else {
            return Ok(None);
        };
        let Some((archive, _, checksum)) = Self::read_asset(
            ctx,
            blobstore,
            content_repository_id,
            &Self::archive_path(scope, name, version),
        )
        .await?
        else {
            return Ok(None);
        };
        let Some((stored, _, _)) = Self::read_asset(
            ctx,
            blobstore,
            content_repository_id,
            &Self::release_path(scope, name, version),
        )
        .await?
        else {
            return Err(anyhow!("Swift release metadata is missing"));
        };
        let stored: StoredSwiftRelease =
            serde_json::from_slice(&stored).context("invalid stored Swift release metadata")?;
        debug_assert_eq!(checksum, hex::encode(sha2::Sha256::digest(&archive)));
        Ok(Some(SwiftReleaseMetadata {
            id: format!("{scope}.{name}"),
            version: version.to_string(),
            resources: vec![SwiftReleaseResource {
                name: "source-archive".to_string(),
                content_type: "application/zip".to_string(),
                checksum,
                signing: stored.source_archive_signature,
            }],
            metadata: stored.metadata,
            published_at: stored.published_at,
        }))
    }

    pub async fn get_manifest(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        scope: &str,
        name: &str,
        version: &str,
        filename: &str,
    ) -> Result<Option<Bytes>> {
        let Some(content_repository_id) = Self::content_repository_id(ctx, repo).await? else {
            return Ok(None);
        };
        Ok(Self::read_asset(
            ctx,
            blobstore,
            content_repository_id,
            &Self::manifest_path(scope, name, version, filename),
        )
        .await?
        .map(|item| item.0))
    }

    pub async fn list_manifests(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        scope: &str,
        name: &str,
        version: &str,
    ) -> Result<Vec<SwiftManifest>> {
        let Some(archive) =
            Self::get_source_archive(ctx, repo, blobstore, scope, name, version).await?
        else {
            return Ok(Vec::new());
        };
        extract_swift_manifests(&archive)
    }

    pub async fn get_source_archive(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        scope: &str,
        name: &str,
        version: &str,
    ) -> Result<Option<Bytes>> {
        let Some(content_repository_id) = Self::content_repository_id(ctx, repo).await? else {
            return Ok(None);
        };
        Ok(Self::read_asset(
            ctx,
            blobstore,
            content_repository_id,
            &Self::archive_path(scope, name, version),
        )
        .await?
        .map(|item| item.0))
    }

    pub async fn get_archive_signature(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        scope: &str,
        name: &str,
        version: &str,
    ) -> Result<Option<SwiftReleaseSignature>> {
        Ok(
            Self::get_release_metadata(ctx, repo, blobstore, scope, name, version)
                .await?
                .and_then(|metadata| metadata.resources.into_iter().next())
                .and_then(|resource| resource.signing),
        )
    }

    pub async fn lookup_identifiers(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        repository_url: &str,
    ) -> Result<Vec<String>> {
        let Some(content_repository_id) = Self::content_repository_id(ctx, repo).await? else {
            return Ok(Vec::new());
        };
        let components = ComponentService::list_by_repository(ctx, content_repository_id).await?;
        let mut identifiers = Vec::new();
        for component in components
            .into_iter()
            .filter(|component| component.kind() == "swift-package")
        {
            let path = Self::release_path(
                &component.namespace(),
                &component.name(),
                &component.version_name(),
            );
            let Ok(Some((stored, _, _))) =
                Self::read_asset(ctx, blobstore, content_repository_id, &path).await
            else {
                continue;
            };
            let Ok(stored) = serde_json::from_slice::<StoredSwiftRelease>(&stored) else {
                continue;
            };
            let matches = stored
                .metadata
                .get("repositoryURLs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|urls| {
                    urls.iter()
                        .filter_map(serde_json::Value::as_str)
                        .any(|url| url == repository_url)
                });
            if matches {
                let id = format!("{}.{}", component.namespace(), component.name());
                if !identifiers.contains(&id) {
                    identifiers.push(id);
                }
            }
        }
        identifiers.sort();
        Ok(identifiers)
    }
}
