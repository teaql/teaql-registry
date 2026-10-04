use anyhow::{anyhow, Result};
use base64::Engine;
use bytes::Bytes;
use std::collections::HashMap;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use super::publication::PublicationGuard;
use crate::blobstore::BlobStore;
use crate::format::npm::{NpmDist, NpmPackageDocument, NpmVersionDetail};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct NpmEngine;

impl NpmEngine {
    pub async fn publish_package(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        doc: &NpmPackageDocument,
    ) -> Result<()> {
        // Preflight the entire document, rather than writing one attachment
        // before discovering another is invalid or already published.
        let mut prepared = Vec::new();
        let mut paths = Vec::new();
        let (namespace, name) = if doc.name.starts_with('@') {
            doc.name.split_once('/').unwrap_or(("", &doc.name))
        } else {
            ("", doc.name.as_str())
        };
        for (filename, attachment) in &doc.attachments {
            let version = attachment_version(doc, filename)?;
            let data = base64::engine::general_purpose::STANDARD
                .decode(&attachment.data)
                .map_err(|e| anyhow!("Invalid base64 attachment data: {}", e))?;
            let path = format!("/{}/-/{}-{}.tgz", doc.name, name, version);
            anyhow::ensure!(
                !paths.contains(&path),
                "Duplicate npm attachment identity: {path}"
            );
            paths.push(path.clone());
            prepared.push((attachment, version, data, path));
        }
        let publication = PublicationGuard::begin(ctx, repo, "npm", &paths).await?;
        let content_repo = &publication.content_repository;

        for (attachment, version, data, path) in prepared {
            let blob_info = blobstore.create_blob(&data).await?;

            let asset_blob = AssetService::create_asset_blob(
                ctx,
                repo.blob_store_id(),
                &blob_info.blob_ref,
                blob_info.size,
                attachment
                    .content_type
                    .as_deref()
                    .unwrap_or("application/gzip"),
                &blob_info.checksums.sha1,
                &blob_info.checksums.sha256,
                &blob_info.checksums.md5,
            )
            .await?;

            let comp = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                namespace,
                name,
                version,
                "tgz",
            )
            .await?;

            // npm may submit scoped attachment keys such as
            // @scope/package-version.tgz. The download route has one tarball
            // segment, so store the canonical basename returned in metadata.
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

        let comps =
            ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
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
            let encoded_package_name = package_name.replace('/', "%2F");
            let tarball_url = format!(
                "{}/{}/-/{}-{}.tgz",
                base_url.trim_end_matches('/'),
                encoded_package_name,
                name,
                ver
            );
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
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<Bytes>> {
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
            Ok(data) => Ok(Some(data)),
            Err(_) => Ok(None),
        }
    }
}

fn attachment_version<'a>(doc: &'a NpmPackageDocument, filename: &str) -> Result<&'a str> {
    let basename = filename.rsplit('/').next().unwrap_or(filename);
    let mut matching = doc
        .versions
        .iter()
        .filter(|(_, detail)| detail.dist.tarball.rsplit('/').next() == Some(basename));
    let (version, detail) = matching
        .next()
        .ok_or_else(|| anyhow!("npm attachment has no matching version metadata: {filename}"))?;
    anyhow::ensure!(
        matching.next().is_none(),
        "Ambiguous npm attachment version: {filename}"
    );
    anyhow::ensure!(
        detail.name == doc.name && detail.version == *version,
        "npm publish metadata does not match attachment identity: {filename}"
    );
    Ok(version)
}
