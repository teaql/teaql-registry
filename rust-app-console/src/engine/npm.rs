use anyhow::{anyhow, Result};
use base64::Engine;
use bytes::Bytes;
use nexus_repository_service_core::{RepositoryConfiguration, ServiceRuntime};
use std::collections::HashMap;

use crate::blobstore::FileBlobStore;
use crate::format::npm::{NpmDist, NpmPackageDocument, NpmVersionDetail};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct NpmEngine;

impl NpmEngine {
    pub async fn publish_package(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &FileBlobStore,
        doc: &NpmPackageDocument,
    ) -> Result<()> {
        let content_repo = RepositoryService::ensure_content_repository(ctx, repo.id(), "npm").await?;

        for (filename, attachment) in &doc.attachments {
            let data = base64::engine::general_purpose::STANDARD.decode(&attachment.data)
                .map_err(|e| anyhow!("Invalid base64 attachment data: {}", e))?;

            let blob_info = blobstore.create_blob(&data).await?;

            let asset_blob = AssetService::create_asset_blob(
                ctx,
                repo.blob_store_id(),
                &blob_info.blob_ref,
                blob_info.size,
                attachment.content_type.as_deref().unwrap_or("application/gzip"),
                &blob_info.checksums.sha1,
                &blob_info.checksums.sha256,
                &blob_info.checksums.md5,
            )
            .await?;

            // Find version for this attachment
            let version = doc
                .versions
                .iter()
                .find(|(_, v)| v.dist.tarball.ends_with(filename) || filename.contains(v.version.as_str()))
                .map(|(ver, _)| ver.as_str())
                .unwrap_or("1.0.0");

            let (namespace, name) = if doc.name.starts_with('@') {
                doc.name.split_once('/').unwrap_or(("", &doc.name))
            } else {
                ("", doc.name.as_str())
            };

            let comp = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                namespace,
                name,
                version,
                "tgz",
            )
            .await?;

            let path = format!("/{}/-/{}", doc.name, filename);
            AssetService::upsert_asset(
                ctx,
                content_repo.id(),
                Some(comp.id()),
                asset_blob.id(),
                &path,
                "tgz",
            )
            .await?;
        }

        Ok(())
    }

    pub async fn get_package_document(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        package_name: &str,
        base_url: &str,
    ) -> Result<Option<NpmPackageDocument>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let (namespace, name) = if package_name.starts_with('@') {
            package_name.split_once('/').unwrap_or(("", package_name))
        } else {
            ("", package_name)
        };

        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let matching: Vec<_> = comps
            .into_iter()
            .filter(|c| c.name() == name && (namespace.is_empty() || c.namespace() == namespace))
            .collect();

        if matching.is_empty() {
            return Ok(None);
        }

        let mut versions = HashMap::new();
        let mut dist_tags = HashMap::new();
        let mut latest_ver = "1.0.0".to_string();

        for c in matching {
            let ver = c.version_name().to_string();
            latest_ver = ver.clone();
            let tarball_url = format!("{}/{}/-/{}-{}.tgz", base_url.trim_end_matches('/'), package_name, name, ver);
            versions.insert(
                ver.clone(),
                NpmVersionDetail {
                    name: package_name.to_string(),
                    version: ver,
                    description: None,
                    dist: NpmDist {
                        shasum: String::new(),
                        tarball: tarball_url,
                        integrity: None,
                    },
                },
            );
        }

        dist_tags.insert("latest".to_string(), latest_ver);

        Ok(Some(NpmPackageDocument {
            id: package_name.to_string(),
            name: package_name.to_string(),
            description: None,
            dist_tags,
            versions,
            attachments: HashMap::new(),
        }))
    }

    pub async fn get_tarball(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &FileBlobStore,
        path: &str,
    ) -> Result<Option<Bytes>> {
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
            Ok(data) => Ok(Some(data)),
            Err(_) => Ok(None),
        }
    }
}
