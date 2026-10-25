use anyhow::Result;
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};
use tracing::info;

use super::hosted::HostedEngine;
use crate::blobstore::BlobStore;

pub struct ProxyEngine;

impl ProxyEngine {
    pub async fn handle_get(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>> {
        // 1. Check local cache first
        if let Some(cached) = HostedEngine::handle_get(ctx, repo, blobstore, path).await? {
            return Ok(Some(cached));
        }

        let cache_key = format!("{}:{}", repo.id(), repo.remote_url());

        // 2. Check Negative Cache
        if crate::engine::proxy_cache::ProxyNegativeCache::global()
            .is_negative_cached(&cache_key, path)
        {
            info!("Proxy negative cache hit for {}:{}", repo.name(), path);
            return Ok(None);
        }

        // 3. Fetch from upstream remote URL
        let remote_url = repo.remote_url();
        if remote_url.is_empty() {
            return Ok(None);
        }

        let target_url = format!(
            "{}/{}",
            remote_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        info!("Proxy fetching from upstream: {}", target_url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client.get(&target_url).send().await?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                crate::engine::proxy_cache::ProxyNegativeCache::global()
                    .record_not_found(&cache_key, path, 300);
                return Ok(None);
            }
            response.error_for_status_ref()?;
            return Ok(None);
        }

        let content_type = response
            .headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = response.bytes().await?;

        // 3. Cache fetched artifact in local blobstore & DB
        HostedEngine::cache_remote_asset(ctx, repo, blobstore, path, &bytes, &content_type).await?;

        Ok(Some((bytes, content_type)))
    }
}
