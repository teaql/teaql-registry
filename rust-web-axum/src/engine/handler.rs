use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use nexus_repository_service_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;

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
}
