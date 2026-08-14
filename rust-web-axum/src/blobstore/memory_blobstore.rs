use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes::Bytes;
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use super::traits::{BlobChecksums, BlobInfo, BlobStore};

#[derive(Clone, Default)]
pub struct MemoryBlobStore {
    store_name: String,
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryBlobStore {
    pub fn new(store_name: impl Into<String>) -> Self {
        Self {
            store_name: store_name.into(),
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl BlobStore for MemoryBlobStore {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo> {
        let blob_id = Uuid::new_v4().to_string();
        let blob_ref = format!("{}@{}", self.store_name, blob_id);

        let mut sha1_h = Sha1::new();
        sha1_h.update(data);
        let sha1 = hex::encode(sha1_h.finalize());

        let mut sha256_h = Sha256::new();
        sha256_h.update(data);
        let sha256 = hex::encode(sha256_h.finalize());

        let mut md5_h = Md5::new();
        md5_h.update(data);
        let md5 = hex::encode(md5_h.finalize());

        {
            let mut lock = self.storage.write().unwrap();
            lock.insert(blob_id.clone(), data.to_vec());
        }

        Ok(BlobInfo {
            blob_id,
            blob_ref,
            size: data.len() as i64,
            checksums: BlobChecksums { sha1, sha256, md5 },
        })
    }

    async fn read_blob(&self, blob_ref: &str) -> Result<Bytes> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let lock = self.storage.read().unwrap();
        if let Some(bytes) = lock.get(blob_id) {
            Ok(Bytes::copy_from_slice(bytes))
        } else {
            Err(anyhow!("Blob not found in memory: {}", blob_ref))
        }
    }

    async fn delete_blob(&self, blob_ref: &str) -> Result<()> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let mut lock = self.storage.write().unwrap();
        lock.remove(blob_id);
        Ok(())
    }

    async fn exists_blob(&self, blob_ref: &str) -> Result<bool> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let lock = self.storage.read().unwrap();
        Ok(lock.contains_key(blob_id))
    }

    fn store_name(&self) -> &str {
        &self.store_name
    }
}
