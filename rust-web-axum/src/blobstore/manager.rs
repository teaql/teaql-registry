use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::traits::BlobStore;

#[derive(Clone)]
pub struct BlobStoreManager {
    default_store: Arc<dyn BlobStore>,
    stores: Arc<RwLock<HashMap<String, Arc<dyn BlobStore>>>>,
}

impl BlobStoreManager {
    pub fn new(default_store: Arc<dyn BlobStore>) -> Self {
        let mut map = HashMap::new();
        map.insert(default_store.store_name().to_string(), default_store.clone());
        Self {
            default_store,
            stores: Arc::new(RwLock::new(map)),
        }
    }

    pub fn default_store(&self) -> Arc<dyn BlobStore> {
        self.default_store.clone()
    }

    pub fn register(&self, store: Arc<dyn BlobStore>) {
        let mut map = self.stores.write().unwrap();
        map.insert(store.store_name().to_string(), store);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn BlobStore>> {
        let map = self.stores.read().unwrap();
        map.get(name).cloned().or_else(|| {
            if name == "default" {
                Some(self.default_store.clone())
            } else {
                None
            }
        })
    }
}
