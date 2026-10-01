use anyhow::{anyhow, Result};
use bytes::Bytes;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};
use tokio::io::AsyncWriteExt;

use crate::blobstore::{BlobStore, ByteStream};
use crate::format::docker::{compute_sha256_digest, DOCKER_MANIFEST_V2_MEDIA_TYPE};
use crate::services::{AssetService, ComponentService, RepositoryService};

struct UploadSession {
    path: PathBuf,
    size: i64,
}

type SharedUploadSession = Arc<tokio::sync::Mutex<UploadSession>>;
type UploadSessions = Arc<Mutex<HashMap<String, SharedUploadSession>>>;

static UPLOAD_SESSIONS: LazyLock<UploadSessions> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct DockerEngine;

impl DockerEngine {
    pub async fn start_upload(_image_name: &str) -> Result<String> {
        let upload_uuid = uuid::Uuid::new_v4().to_string();
        let path = std::env::temp_dir().join(format!("teaql-docker-{upload_uuid}.upload"));
        tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await?;
        let mut sessions = UPLOAD_SESSIONS.lock().expect("lock poisoned");
        sessions.insert(
            upload_uuid.clone(),
            Arc::new(tokio::sync::Mutex::new(UploadSession { path, size: 0 })),
        );
        Ok(upload_uuid)
    }

    pub async fn append_chunk(upload_uuid: &str, stream: ByteStream) -> Result<i64> {
        let session = UPLOAD_SESSIONS
            .lock()
            .expect("lock poisoned")
            .get(upload_uuid)
            .cloned()
            .ok_or_else(|| anyhow!("Upload session not found: {}", upload_uuid))?;
        Self::append_to_session(&session, stream).await
    }

    pub async fn cancel_upload(upload_uuid: &str) {
        let session = UPLOAD_SESSIONS
            .lock()
            .expect("lock poisoned")
            .remove(upload_uuid);
        if let Some(session) = session {
            let path = session.lock().await.path.clone();
            let _ = tokio::fs::remove_file(path).await;
        }
    }

    async fn append_to_session(
        session: &SharedUploadSession,
        mut stream: ByteStream,
    ) -> Result<i64> {
        use futures_util::StreamExt;

        let mut session = session.lock().await;
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&session.path)
            .await?;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| anyhow!("stream read error: {error}"))?;
            file.write_all(&chunk).await?;
            session.size += chunk.len() as i64;
        }
        file.flush().await?;
        Ok(session.size)
    }

    pub async fn store_blob_stream(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        image_name: &str,
        expected_digest: &str,
        stream: ByteStream,
    ) -> Result<(String, i64)> {
        let blob_info = blobstore.create_blob_from_stream(stream).await?;
        let computed_digest = format!("sha256:{}", blob_info.checksums.sha256);
        if !expected_digest.is_empty() && expected_digest != computed_digest {
            let _ = blobstore.delete_blob(&blob_info.blob_ref).await;
            return Err(anyhow!(
                "Digest mismatch: expected {}, computed {}",
                expected_digest,
                computed_digest
            ));
        }

        let content_repo =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "docker").await?;

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

        Ok((computed_digest, blob_info.size))
    }

    pub async fn finish_upload(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        image_name: &str,
        upload_uuid: &str,
        expected_digest: &str,
        extra_data: Option<ByteStream>,
    ) -> Result<(String, i64)> {
        let session = UPLOAD_SESSIONS
            .lock()
            .expect("lock poisoned")
            .remove(upload_uuid)
            .ok_or_else(|| anyhow!("Upload session not found: {}", upload_uuid))?;

        if let Some(stream) = extra_data {
            if let Err(error) = Self::append_to_session(&session, stream).await {
                let path = session.lock().await.path.clone();
                let _ = tokio::fs::remove_file(path).await;
                return Err(error);
            }
        }

        let path = session.lock().await.path.clone();
        let result = async {
            let file = tokio::fs::File::open(&path).await?;
            let stream = tokio_util::io::ReaderStream::new(tokio::io::BufReader::new(file));
            Self::store_blob_stream(
                ctx,
                repo,
                blobstore,
                image_name,
                expected_digest,
                Box::pin(stream),
            )
            .await
        }
        .await;
        let _ = tokio::fs::remove_file(path).await;
        result
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
        let asset =
            match AssetService::find_by_path_with_blob(ctx, content_repo.id(), &path).await? {
                Some(a) => a,
                None => return Ok(None),
            };
        let blob = match asset.blob() {
            Some(blob) => blob,
            None => return Ok(None),
        };

        match blobstore.read_blob(blob.blob_ref()).await {
            Ok(data) => Ok(Some((data, blob.content_type().to_owned()))),
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
        let asset =
            match AssetService::find_by_path_with_blob(ctx, content_repo.id(), &path).await? {
                Some(a) => a,
                None => return Ok(None),
            };
        let blob = match asset.blob() {
            Some(blob) => blob,
            None => return Ok(None),
        };

        Ok(Some((blob.blob_size(), blob.content_type().to_owned())))
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
        let content_repo =
            RepositoryService::ensure_content_repository(ctx, repo.id(), "docker").await?;

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
        let asset =
            match AssetService::find_by_path_with_blob(ctx, content_repo.id(), &path).await? {
                Some(a) => a,
                None => return Ok(None),
            };
        let blob = match asset.blob() {
            Some(blob) => blob,
            None => return Ok(None),
        };

        match blobstore.read_blob(blob.blob_ref()).await {
            Ok(data) => {
                let digest = compute_sha256_digest(&data);
                Ok(Some((data, blob.content_type().to_owned(), digest)))
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

        let comps =
            ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let tags: Vec<String> = comps
            .into_iter()
            .filter(|c| c.name() == name && (namespace.is_empty() || c.namespace() == namespace))
            .map(|c| c.version_name().to_string())
            .filter(|v| !v.is_empty() && !v.starts_with("sha256:"))
            .collect();

        Ok(tags)
    }
}
