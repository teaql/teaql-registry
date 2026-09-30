use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::{BlobStore, ByteStream};

#[async_trait]
pub trait RepositoryHandler: Send + Sync {
    /// Supported format or recipe name (e.g., "maven2", "raw", "docker", "npm")
    fn format_name(&self) -> &'static str;

    /// Handle content retrieval
    async fn get(
        &self,
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>>;

    /// Handle content publishing / upload
    async fn put(
        &self,
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<()>;

    /// Handle streaming content upload. Default collects to bytes and calls put.
    async fn put_stream(
        &self,
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
        stream: ByteStream,
        content_type: &str,
    ) -> Result<()> {
        use futures_util::StreamExt;
        let mut buf = Vec::new();
        let mut stream = stream;
        while let Some(chunk) = stream.next().await {
            buf.extend_from_slice(&chunk.map_err(|e| anyhow::anyhow!("stream error: {}", e))?);
        }
        self.put(ctx, repo, blobstore, path, &buf, content_type)
            .await
    }
}
