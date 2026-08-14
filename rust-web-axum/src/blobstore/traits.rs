use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobChecksums {
    pub sha1: String,
    pub sha256: String,
    pub md5: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobInfo {
    pub blob_id: String,
    pub blob_ref: String,
    pub size: i64,
    pub checksums: BlobChecksums,
}

#[async_trait]
pub trait BlobStore: Send + Sync {
    /// Initialize storage backend (e.g. bucket or directory creation)
    async fn init(&self) -> Result<()>;

    /// Create and persist a new blob with computed hashes
    async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo>;

    /// Read blob content by reference identifier
    async fn read_blob(&self, blob_ref: &str) -> Result<Bytes>;

    /// Delete blob by reference identifier
    async fn delete_blob(&self, blob_ref: &str) -> Result<()>;

    /// Check if a blob exists
    async fn exists_blob(&self, blob_ref: &str) -> Result<bool>;

    /// Name/label of this blobstore instance
    fn store_name(&self) -> &str;
}

#[async_trait]
impl<T: ?Sized + BlobStore> BlobStore for Arc<T> {
    async fn init(&self) -> Result<()> {
        (**self).init().await
    }
    async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo> {
        (**self).create_blob(data).await
    }
    async fn read_blob(&self, blob_ref: &str) -> Result<Bytes> {
        (**self).read_blob(blob_ref).await
    }
    async fn delete_blob(&self, blob_ref: &str) -> Result<()> {
        (**self).delete_blob(blob_ref).await
    }
    async fn exists_blob(&self, blob_ref: &str) -> Result<bool> {
        (**self).exists_blob(blob_ref).await
    }
    fn store_name(&self) -> &str {
        (**self).store_name()
    }
}

#[async_trait]
impl<T: ?Sized + BlobStore> BlobStore for Box<T> {
    async fn init(&self) -> Result<()> {
        (**self).init().await
    }
    async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo> {
        (**self).create_blob(data).await
    }
    async fn read_blob(&self, blob_ref: &str) -> Result<Bytes> {
        (**self).read_blob(blob_ref).await
    }
    async fn delete_blob(&self, blob_ref: &str) -> Result<()> {
        (**self).delete_blob(blob_ref).await
    }
    async fn exists_blob(&self, blob_ref: &str) -> Result<bool> {
        (**self).exists_blob(blob_ref).await
    }
    fn store_name(&self) -> &str {
        (**self).store_name()
    }
}
