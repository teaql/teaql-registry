use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

#[derive(Debug, Clone)]
struct CacheEntry {
    cached_at: DateTime<Utc>,
    ttl_seconds: i64,
}

#[derive(Clone, Default)]
pub struct ProxyNegativeCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

static GLOBAL_NEGATIVE_CACHE: LazyLock<ProxyNegativeCache> =
    LazyLock::new(|| ProxyNegativeCache {
        entries: Arc::new(RwLock::new(HashMap::new())),
    });

impl ProxyNegativeCache {
    pub fn global() -> &'static ProxyNegativeCache {
        &GLOBAL_NEGATIVE_CACHE
    }

    fn key(repo_name: &str, path: &str) -> String {
        format!("{}:{}", repo_name, path)
    }

    pub fn is_negative_cached(&self, repo_name: &str, path: &str) -> bool {
        let key = Self::key(repo_name, path);
        let entries = self.entries.read().unwrap();
        if let Some(entry) = entries.get(&key) {
            let expires_at = entry.cached_at + Duration::seconds(entry.ttl_seconds);
            if Utc::now() < expires_at {
                return true;
            }
        }
        false
    }

    pub fn record_not_found(&self, repo_name: &str, path: &str, ttl_seconds: i64) {
        let key = Self::key(repo_name, path);
        let mut entries = self.entries.write().unwrap();
        entries.insert(
            key,
            CacheEntry {
                cached_at: Utc::now(),
                ttl_seconds,
            },
        );
    }

    pub fn invalidate(&self, repo_name: &str, path: &str) {
        let key = Self::key(repo_name, path);
        let mut entries = self.entries.write().unwrap();
        entries.remove(&key);
    }

    pub fn clear_all(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }
}
