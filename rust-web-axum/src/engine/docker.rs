use anyhow::{anyhow, Result};
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, LazyLock};

use crate::blobstore::BlobStore;
use crate::format::docker::{
    compute_sha256_digest, DOCKER_MANIFEST_V2_MEDIA_TYPE,
};
use crate::services::{AssetService, ComponentService, RepositoryService};

static UPLOAD_SESSIONS: LazyLock<Arc<Mutex<HashMap<String, Vec<u8>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct DockerEngine;

impl DockerEngine {
    pub fn start_upload(_image_name: &str) -> String {
        let upload_uuid = uuid::Uuid::new_v4().to_string();
        let mut sessions = UPLOAD_SESSIONS.lock().unwrap();
        sessions.insert(upload_uuid.clone(), Vec::new());
        upload_uuid
    }

    pub fn append_chunk(upload_uuid: &str, chunk: &[u8]) -> Result<usize> {
        let mut sessions = UPLOAD_SESSIONS.lock().unwrap();
        let buf = sessions
            .get_mut(upload_uuid)
            .ok_or_else(|| anyhow!("Upload session not found: {}", upload_uuid))?;
        buf.extend_from_slice(chunk);
        Ok(buf.len())
    }

    pub async fn finish_upload(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        image_name: &str,
        upload_uuid: &str,
        expected_digest: &str,
        extra_data: Option<&[u8]>,
    ) -> Result<String> {
        let mut data = {
            let mut sessions = UPLOAD_SESSIONS.lock().unwrap();
            sessions
                .remove(upload_uuid)
                .unwrap_or_default()
        };

        if let Some(extra) = extra_data {
            data.extend_from_slice(extra);
        }

        let computed_digest = compute_sha256_digest(&data);
        if !expected_digest.is_empty() && expected_digest != computed_digest {
            return Err(anyhow!(
                "Digest mismatch: expected {}, computed {}",
                expected_digest,
                computed_digest
            ));
        }

        let content_repo = RepositoryService::ensure_content_repository(ctx, repo.id(), "docker").await?;

        // Write binary to BlobStore
        let blob_info = blobstore.create_blob(&data).await?;

        // Save AssetBlob record in PostgreSQL via TeaQL
        let asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            "application/octet-stream",
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;

        // Save Asset record for /v2/<name>/blobs/<digest>
        let path = format!("/v2/{}/blobs/{}", image_name, computed_digest);
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            None,
            asset_blob.id(),
            &path,
            "layer",
        )
        .await?;

        Ok(computed_digest)
    }

    pub async fn get_blob(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        image_name: &str,
        digest: &str,
    ) -> Result<Option<(Bytes, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let path = format!("/v2/{}/blobs/{}", image_name, digest);
        let asset = match AssetService::find_by_path(ctx, content_repo.id(), &path).await? {
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

    pub async fn has_blob(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        image_name: &str,
        digest: &str,
    ) -> Result<Option<(i64, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let path = format!("/v2/{}/blobs/{}", image_name, digest);
        let asset = match AssetService::find_by_path(ctx, content_repo.id(), &path).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        let asset_blob = match AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        Ok(Some((asset_blob.blob_size(), asset_blob.content_type().to_string())))
    }

    pub async fn put_manifest(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        image_name: &str,
        reference: &str,
        manifest_data: &[u8],
        content_type: &str,
    ) -> Result<String> {
        let digest = compute_sha256_digest(manifest_data);
        let content_repo = RepositoryService::ensure_content_repository(ctx, repo.id(), "docker").await?;

        // Write binary to BlobStore
        let blob_info = blobstore.create_blob(manifest_data).await?;

        let ct = if content_type.is_empty() || content_type == "application/octet-stream" {
            DOCKER_MANIFEST_V2_MEDIA_TYPE
        } else {
            content_type
        };

        let asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            ct,
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;

        // Split image_name into namespace and name if contains slash
        let (namespace, name) = if let Some((ns, n)) = image_name.split_once('/') {
            (ns, n)
        } else {
            ("", image_name)
        };

        let comp = ComponentService::find_or_create(
            ctx,
            content_repo.id(),
            namespace,
            name,
            reference,
            "manifest",
        )
        .await?;

        // Save Asset for reference (e.g. tag or digest)
        let ref_path = format!("/v2/{}/manifests/{}", image_name, reference);
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            Some(comp.id()),
            asset_blob.id(),
            &ref_path,
            "manifest",
        )
        .await?;

        // If reference is a tag (not a digest), also save by digest
        if !reference.starts_with("sha256:") {
            let digest_path = format!("/v2/{}/manifests/{}", image_name, digest);
            AssetService::upsert_asset(
                ctx,
                content_repo.id(),
                Some(comp.id()),
                asset_blob.id(),
                &digest_path,
                "manifest",
            )
            .await?;
        }

        Ok(digest)
    }

    pub async fn get_manifest(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        image_name: &str,
        reference: &str,
    ) -> Result<Option<(Bytes, String, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let path = format!("/v2/{}/manifests/{}", image_name, reference);
        let asset = match AssetService::find_by_path(ctx, content_repo.id(), &path).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        let asset_blob = match AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        match blobstore.read_blob(&asset_blob.blob_ref()).await {
            Ok(data) => {
                let digest = compute_sha256_digest(&data);
                Ok(Some((data, asset_blob.content_type().to_string(), digest)))
            }
            Err(_) => Ok(None),
        }
    }

    pub async fn list_tags(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        image_name: &str,
    ) -> Result<Vec<String>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(Vec::new()),
        };

        let (namespace, name) = if let Some((ns, n)) = image_name.split_once('/') {
            (ns, n)
        } else {
            ("", image_name)
        };

        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let tags: Vec<String> = comps
            .into_iter()
            .filter(|c| c.name() == name && (namespace.is_empty() || c.namespace() == namespace))
            .map(|c| c.version_name().to_string())
            .filter(|v| !v.is_empty() && !v.starts_with("sha256:"))
            .collect();

        Ok(tags)
    }
}
