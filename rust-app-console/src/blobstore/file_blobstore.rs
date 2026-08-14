use anyhow::{Context, Result};
use bytes::Bytes;
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BlobChecksums {
    pub sha1: String,
    pub sha256: String,
    pub md5: String,
}

#[derive(Debug, Clone)]
pub struct BlobInfo {
    pub blob_id: String,
    pub blob_ref: String,
    pub size: i64,
    pub checksums: BlobChecksums,
}

#[derive(Clone)]
pub struct FileBlobStore {
    base_path: PathBuf,
    store_name: String,
}

impl FileBlobStore {
    pub fn new(base_path: impl AsRef<Path>, store_name: impl Into<String>) -> Self {
        let store_name = store_name.into();
        let base_path = base_path.as_ref().join(&store_name);
        Self {
            base_path,
            store_name,
        }
    }

    pub async fn init(&self) -> Result<()> {
        let content_dir = self.base_path.join("content");
        fs::create_dir_all(&content_dir)
            .await
            .context("Failed to create blobstore content directory")?;
        Ok(())
    }

    pub fn get_blob_path(&self, blob_id: &str) -> PathBuf {
        let prefix = if blob_id.len() >= 2 {
            &blob_id[0..2]
        } else {
            "00"
        };
        self.base_path.join("content").join(prefix).join(blob_id)
    }

    pub async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo> {
        let blob_id = Uuid::new_v4().to_string();
        let blob_ref = format!("{}@{}", self.store_name, blob_id);
        let blob_path = self.get_blob_path(&blob_id);

        if let Some(parent) = blob_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = File::create(&blob_path).await?;
        file.write_all(data).await?;
        file.flush().await?;

        // Calculate Checksums
        let mut sha1_hasher = Sha1::new();
        sha1_hasher.update(data);
        let sha1 = hex::encode(sha1_hasher.finalize());

        let mut sha256_hasher = Sha256::new();
        sha256_hasher.update(data);
        let sha256 = hex::encode(sha256_hasher.finalize());

        let mut md5_hasher = Md5::new();
        md5_hasher.update(data);
        let md5 = hex::encode(md5_hasher.finalize());

        Ok(BlobInfo {
            blob_id,
            blob_ref,
            size: data.len() as i64,
            checksums: BlobChecksums { sha1, sha256, md5 },
        })
    }

    pub async fn read_blob(&self, blob_ref: &str) -> Result<Bytes> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let blob_path = self.get_blob_path(blob_id);

        let mut file = File::open(&blob_path)
            .await
            .with_context(|| format!("Blob not found: {}", blob_ref))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(Bytes::from(buffer))
    }

    pub async fn delete_blob(&self, blob_ref: &str) -> Result<()> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let blob_path = self.get_blob_path(blob_id);
        if blob_path.exists() {
            fs::remove_file(&blob_path).await?;
        }
        Ok(())
    }
}
