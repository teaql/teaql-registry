use anyhow::{bail, Context, Result};
use bytes::Bytes;
use flate2::{write::GzEncoder, Compression};
use prost::Message;
use rand::rngs::OsRng;
use rsa::{
    pkcs1::DecodeRsaPrivateKey,
    pkcs1v15::SigningKey,
    pkcs8::{DecodePrivateKey, EncodePublicKey, LineEnding},
    signature::{SignatureEncoding, Signer},
    RsaPrivateKey, RsaPublicKey,
};
use serde_json::{json, Value};
use sha2::Sha512;
use std::io::Write;
use std::sync::OnceLock;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::hex::{extract_hex_package, validate_hex_name, StoredHexPackage};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct HexEngine;

mod wire {
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Signed {
        #[prost(bytes = "vec", required, tag = "1")]
        pub payload: Vec<u8>,
        #[prost(bytes = "vec", optional, tag = "2")]
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Timestamp {
        #[prost(int64, required, tag = "1")]
        pub seconds: i64,
        #[prost(int32, required, tag = "2")]
        pub nanos: i32,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct NamesPackage {
        #[prost(string, required, tag = "1")]
        pub name: String,
        #[prost(message, optional, tag = "3")]
        pub updated_at: Option<Timestamp>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Names {
        #[prost(message, repeated, tag = "1")]
        pub packages: Vec<NamesPackage>,
        #[prost(string, required, tag = "2")]
        pub repository: String,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct VersionsPackage {
        #[prost(string, required, tag = "1")]
        pub name: String,
        #[prost(string, repeated, tag = "2")]
        pub versions: Vec<String>,
        #[prost(int32, repeated, packed = "true", tag = "3")]
        pub retired: Vec<i32>,
        #[prost(int32, repeated, packed = "true", tag = "5")]
        pub with_advisories: Vec<i32>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Versions {
        #[prost(message, repeated, tag = "1")]
        pub packages: Vec<VersionsPackage>,
        #[prost(string, required, tag = "2")]
        pub repository: String,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Dependency {
        #[prost(string, required, tag = "1")]
        pub package: String,
        #[prost(string, required, tag = "2")]
        pub requirement: String,
        #[prost(bool, optional, tag = "3")]
        pub optional: Option<bool>,
        #[prost(string, optional, tag = "4")]
        pub app: Option<String>,
        #[prost(string, optional, tag = "5")]
        pub repository: Option<String>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Release {
        #[prost(string, required, tag = "1")]
        pub version: String,
        #[prost(bytes = "vec", required, tag = "2")]
        pub inner_checksum: Vec<u8>,
        #[prost(message, repeated, tag = "3")]
        pub dependencies: Vec<Dependency>,
        #[prost(bytes = "vec", optional, tag = "5")]
        pub outer_checksum: Option<Vec<u8>>,
        #[prost(uint32, repeated, tag = "6")]
        pub advisory_indexes: Vec<u32>,
        #[prost(message, optional, tag = "7")]
        pub published_at: Option<Timestamp>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Package {
        #[prost(message, repeated, tag = "1")]
        pub releases: Vec<Release>,
        #[prost(string, required, tag = "2")]
        pub name: String,
        #[prost(string, required, tag = "3")]
        pub repository: String,
    }
}

fn private_key() -> Result<&'static RsaPrivateKey> {
    static KEY: OnceLock<Result<RsaPrivateKey, String>> = OnceLock::new();
    KEY.get_or_init(|| {
        let inline_pem = std::env::var("HEX_PRIVATE_KEY_PEM")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let key_path = std::env::var("HEX_PRIVATE_KEY_PATH")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let pem = if let Some(pem) = inline_pem {
            Some(pem)
        } else if let Some(path) = key_path {
            Some(std::fs::read_to_string(path).map_err(|error| error.to_string())?)
        } else {
            None
        };
        if let Some(pem) = pem {
            RsaPrivateKey::from_pkcs8_pem(&pem)
                .or_else(|_| RsaPrivateKey::from_pkcs1_pem(&pem))
                .map_err(|error| error.to_string())
        } else {
            RsaPrivateKey::new(&mut OsRng, 2048).map_err(|error| error.to_string())
        }
    })
    .as_ref()
    .map_err(|error| anyhow::anyhow!("failed to initialize Hex signing key: {error}"))
}

fn signed_registry<M: Message>(message: M) -> Result<Vec<u8>> {
    let payload = message.encode_to_vec();
    let signing_key = SigningKey::<Sha512>::new(private_key()?.clone());
    let signature = signing_key.sign(&payload).to_vec();
    let signed = wire::Signed {
        payload,
        signature: Some(signature),
    }
    .encode_to_vec();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&signed)?;
    Ok(encoder.finish()?)
}

impl HexEngine {
    fn tarball_path(name: &str, version: &str) -> String {
        format!("/tarballs/{name}-{version}.tar")
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
        archive: &[u8],
    ) -> Result<StoredHexPackage> {
        if repo.write_policy_is_read_only() {
            bail!("Repository is read only");
        }
        let package = extract_hex_package(archive)?;
        let content_repository =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "hex").await?;
        let tarball_path = Self::tarball_path(&package.name, &package.version);
        if AssetService::find_by_path(ctx, content_repository.id(), &tarball_path)
            .await?
            .is_some()
        {
            bail!(
                "Hex package {} {} already exists",
                package.name,
                package.version
            );
        }
        let component = ComponentService::find_or_create(
            ctx,
            content_repository.id(),
            "",
            &package.name,
            &package.version,
            "hex-package",
        )
        .await?;
        Self::persist_asset(
            ctx,
            repo,
            blobstore,
            content_repository.id(),
            component.id(),
            &tarball_path,
            "hex-tarball",
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
            &Self::metadata_path(&package.name, &package.version),
            "hex-metadata",
            "application/json",
            &serde_json::to_vec(&package)?,
        )
        .await?;
        Ok(package)
    }

    async fn all_metadata(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
    ) -> Result<Vec<StoredHexPackage>> {
        let Some(content_repository) =
            RepositoryService::get_content_repository(ctx, repo.id()).await?
        else {
            return Ok(Vec::new());
        };
        let mut packages = Vec::new();
        for component in ComponentService::list_by_repository(ctx, content_repository.id())
            .await?
            .into_iter()
            .filter(|component| component.kind() == "hex-package")
        {
            let Some(bytes) = Self::read_asset(
                ctx,
                blobstore,
                content_repository.id(),
                &Self::metadata_path(&component.name(), &component.version_name()),
            )
            .await?
            else {
                continue;
            };
            packages.push(serde_json::from_slice(&bytes).context("invalid stored Hex metadata")?);
        }
        packages.sort_by(|left: &StoredHexPackage, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.version.cmp(&right.version))
        });
        Ok(packages)
    }

    pub async fn tarball(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        filename: &str,
    ) -> Result<Option<Bytes>> {
        if filename.contains('/') || filename.contains("..") || !filename.ends_with(".tar") {
            bail!("invalid Hex tarball filename");
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
            &format!("/tarballs/{filename}"),
        )
        .await
    }

    pub async fn public_key_pem() -> Result<String> {
        Ok(RsaPublicKey::from(private_key()?)
            .to_public_key_pem(LineEnding::LF)?
            .to_string())
    }

    pub async fn names(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
    ) -> Result<Vec<u8>> {
        let all = Self::all_metadata(ctx, repo, blobstore).await?;
        let mut latest = std::collections::BTreeMap::<String, i64>::new();
        for package in all {
            latest
                .entry(package.name)
                .and_modify(|time| *time = (*time).max(package.published_at))
                .or_insert(package.published_at);
        }
        signed_registry(wire::Names {
            packages: latest
                .into_iter()
                .map(|(name, seconds)| wire::NamesPackage {
                    name,
                    updated_at: Some(wire::Timestamp { seconds, nanos: 0 }),
                })
                .collect(),
            repository: repo.name(),
        })
    }

    pub async fn versions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
    ) -> Result<Vec<u8>> {
        let mut grouped = std::collections::BTreeMap::<String, Vec<String>>::new();
        for package in Self::all_metadata(ctx, repo, blobstore).await? {
            grouped
                .entry(package.name)
                .or_default()
                .push(package.version);
        }
        signed_registry(wire::Versions {
            packages: grouped
                .into_iter()
                .map(|(name, mut versions)| {
                    versions.sort();
                    wire::VersionsPackage {
                        name,
                        versions,
                        retired: Vec::new(),
                        with_advisories: Vec::new(),
                    }
                })
                .collect(),
            repository: repo.name(),
        })
    }

    pub async fn package_registry(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        name: &str,
    ) -> Result<Option<Vec<u8>>> {
        validate_hex_name(name)?;
        let packages = Self::all_metadata(ctx, repo, blobstore)
            .await?
            .into_iter()
            .filter(|package| package.name == name)
            .collect::<Vec<_>>();
        if packages.is_empty() {
            return Ok(None);
        }
        let releases = packages
            .into_iter()
            .map(|package| {
                Ok(wire::Release {
                    version: package.version,
                    inner_checksum: hex::decode(package.inner_checksum)?,
                    dependencies: package
                        .dependencies
                        .into_iter()
                        .map(|dependency| wire::Dependency {
                            package: dependency.package,
                            requirement: dependency.requirement,
                            optional: Some(dependency.optional),
                            app: dependency.app,
                            repository: dependency.repository,
                        })
                        .collect(),
                    outer_checksum: Some(hex::decode(package.outer_checksum)?),
                    advisory_indexes: Vec::new(),
                    published_at: Some(wire::Timestamp {
                        seconds: package.published_at,
                        nanos: 0,
                    }),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        signed_registry(wire::Package {
            releases,
            name: name.to_string(),
            repository: repo.name(),
        })
        .map(Some)
    }

    pub fn publish_response(repo: &RepositoryConfiguration, package: &StoredHexPackage) -> Value {
        let requirements = package
            .dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.package.clone(),
                    json!({
                        "requirement": dependency.requirement,
                        "optional": dependency.optional,
                        "app": dependency.app,
                        "repository": dependency.repository,
                    }),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        json!({
            "version": package.version,
            "checksum": package.outer_checksum,
            "has_docs": false,
            "url": format!("/repository/{}/hex/api/packages/{}/releases/{}", repo.name(), package.name, package.version),
            "package_url": format!("/repository/{}/hex/api/packages/{}", repo.name(), package.name),
            "meta": {
                "app": package.app,
                "build_tools": package.build_tools,
                "description": package.description,
                "licenses": package.licenses,
            },
            "requirements": requirements,
            "retirement": null,
        })
    }
}
