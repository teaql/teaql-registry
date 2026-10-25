//! Persistent upstream responses and metadata-discovered download links.
use super::HostedEngine;
use crate::blobstore::BlobStore;
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock, Mutex, Weak},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .expect("upstream HTTP client")
});
type DownloadLock = tokio::sync::Mutex<()>;
static LOCKS: LazyLock<Mutex<HashMap<String, Weak<DownloadLock>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn lock(key: &str) -> Arc<DownloadLock> {
    let mut locks = LOCKS.lock().expect("download locks");
    if locks.len() > 1024 {
        locks.retain(|_, weak| weak.strong_count() > 0);
    }
    if let Some(existing) = locks.get(key).and_then(Weak::upgrade) {
        return existing;
    }
    let new = Arc::new(tokio::sync::Mutex::new(()));
    locks.insert(key.into(), Arc::downgrade(&new));
    new
}
fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Serialize, Deserialize)]
struct ResponseMetadata {
    status: u16,
    fetched_at: u64,
    headers: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
struct DownloadLink {
    url: String,
    directory: bool,
    filename: String,
}

pub struct RemoteAsset {
    pub body: Bytes,
    pub headers: HeaderMap,
}
pub struct RemoteCache;
impl RemoteCache {
    /// The cache identity includes upstream URL and Accept representation.
    pub async fn fetch(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        store: &dyn BlobStore,
        url: &str,
        accept: &str,
        docker: bool,
    ) -> Result<Option<RemoteAsset>> {
        let parsed = reqwest::Url::parse(url)?;
        anyhow::ensure!(
            matches!(parsed.scheme(), "http" | "https"),
            "Upstream must use HTTP(S)"
        );
        let identity = format!("{}:{url}:{accept}", repo.id());
        let mutex = lock(&identity);
        let _guard = mutex.lock().await;
        let cache_path = format!("/_proxy/cache/{}", hash(&format!("{url}:{accept}")));
        let metadata_path = format!("{cache_path}.json");
        if let Some((data, _)) = HostedEngine::handle_get(ctx, repo, store, &metadata_path).await? {
            let metadata: ResponseMetadata = serde_json::from_slice(&data)?;
            if metadata.status == 404 && now().saturating_sub(metadata.fetched_at) < 300 {
                return Ok(None);
            }
            if metadata.status == 200 {
                if let Some((body, _)) =
                    HostedEngine::handle_get(ctx, repo, store, &cache_path).await?
                {
                    return Ok(Some(RemoteAsset {
                        body,
                        headers: decode_headers(metadata.headers),
                    }));
                }
            }
        }
        let mut response = CLIENT
            .get(url)
            .header("accept", accept)
            .header("accept-encoding", "identity")
            .send()
            .await?;
        if docker && response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let challenge = response
                .headers()
                .get("www-authenticate")
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| {
                    anyhow!("Upstream Docker registry did not provide a bearer challenge")
                })?;
            let token = docker_token(challenge).await?;
            response = CLIENT
                .get(url)
                .header("accept", accept)
                .header("accept-encoding", "identity")
                .bearer_auth(token)
                .send()
                .await?;
        }
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            let metadata = ResponseMetadata {
                status: 404,
                fetched_at: now(),
                headers: BTreeMap::new(),
            };
            HostedEngine::cache_remote_asset(
                ctx,
                repo,
                store,
                &metadata_path,
                &serde_json::to_vec(&metadata)?,
                "application/json",
            )
            .await?;
            return Ok(None);
        }
        response.error_for_status_ref()?;
        anyhow::ensure!(
            response.status().is_success(),
            "Unexpected upstream HTTP status {}",
            response.status()
        );
        // Preserve protocol headers, never upstream cookies, auth or redirect headers.
        let mut headers = BTreeMap::new();
        for name in [
            "content-type",
            "content-encoding",
            "content-disposition",
            "content-version",
            "digest",
            "etag",
            "last-modified",
            "link",
            "docker-content-digest",
            "docker-distribution-api-version",
            "x-conan-server-capabilities",
        ] {
            if let Some(value) = response.headers().get(name).and_then(|v| v.to_str().ok()) {
                headers.insert(name.into(), value.into());
            }
        }
        let body = response.bytes().await?;
        let content_type = headers
            .get("content-type")
            .map(String::as_str)
            .unwrap_or("application/octet-stream");
        HostedEngine::cache_remote_asset(ctx, repo, store, &cache_path, &body, content_type)
            .await?;
        let metadata = ResponseMetadata {
            status: 200,
            fetched_at: now(),
            headers: headers.clone(),
        };
        HostedEngine::cache_remote_asset(
            ctx,
            repo,
            store,
            &metadata_path,
            &serde_json::to_vec(&metadata)?,
            "application/json",
        )
        .await?;
        Ok(Some(RemoteAsset {
            body,
            headers: decode_headers(headers),
        }))
    }

    /// Only links discovered in upstream metadata can be resolved by clients.
    /// Persist the mapping with the artifact metadata, so a restart needs no network.
    pub async fn bind_link(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        store: &dyn BlobStore,
        url: &str,
        local_base: &str,
        directory: bool,
    ) -> Result<String> {
        let mut parsed = reqwest::Url::parse(url)?;
        anyhow::ensure!(
            matches!(parsed.scheme(), "http" | "https"),
            "Download URL must use HTTP(S)"
        );
        let fragment = parsed.fragment().map(str::to_string);
        parsed.set_fragment(None);
        let url = parsed.to_string();
        let filename = parsed
            .path()
            .rsplit('/')
            .next()
            .filter(|p| !p.is_empty())
            .unwrap_or("download")
            .to_string();
        let id = hash(&format!("{url}:{directory}"));
        let path = format!("/_proxy/links/{id}");
        let mutex = lock(&format!("{}:link:{id}", repo.id()));
        let _guard = mutex.lock().await;
        if HostedEngine::handle_get(ctx, repo, store, &path)
            .await?
            .is_none()
        {
            let link = DownloadLink {
                url,
                directory,
                filename: filename.clone(),
            };
            HostedEngine::cache_remote_asset(
                ctx,
                repo,
                store,
                &path,
                &serde_json::to_vec(&link)?,
                "application/json",
            )
            .await?;
        }
        let tail = if directory { "" } else { &filename };
        Ok(format!(
            "{}/_proxy/link/{id}/{tail}{}",
            local_base.trim_end_matches('/'),
            fragment.map(|f| format!("#{f}")).unwrap_or_default()
        ))
    }

    pub async fn resolve_link(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        store: &dyn BlobStore,
        id: &str,
        tail: &str,
        query: Option<&str>,
    ) -> Result<Option<String>> {
        if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Ok(None);
        }
        let Some((bytes, _)) =
            HostedEngine::handle_get(ctx, repo, store, &format!("/_proxy/links/{id}")).await?
        else {
            return Ok(None);
        };
        let link: DownloadLink = serde_json::from_slice(&bytes)?;
        let mut url = reqwest::Url::parse(&link.url)?;
        if link.directory {
            // Relative tails cannot select another upstream origin.
            let relative = tail.trim_start_matches('/');
            anyhow::ensure!(
                !relative.split('/').any(|p| matches!(p, "." | "..")),
                "Invalid download path"
            );
            url = url.join(relative)?;
        } else if tail == format!("{}.metadata", link.filename) {
            url.set_path(&format!("{}.metadata", url.path()));
        } else if tail != link.filename {
            return Ok(None);
        }
        if let Some(query) = query {
            url.set_query(Some(query));
        }
        Ok(Some(url.to_string()))
    }
}
fn decode_headers(values: BTreeMap<String, String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in values {
        if let (Ok(name), Ok(value)) = (
            http::header::HeaderName::try_from(name),
            HeaderValue::from_str(&value),
        ) {
            headers.insert(name, value);
        }
    }
    headers
}
async fn docker_token(challenge: &str) -> Result<String> {
    let params = challenge
        .strip_prefix("Bearer ")
        .or_else(|| challenge.strip_prefix("bearer "))
        .ok_or_else(|| anyhow!("Unsupported upstream Docker authentication scheme"))?;
    let re = regex::Regex::new(r#"([a-z_]+)="([^"]*)""#)?;
    let values = re
        .captures_iter(params)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect::<HashMap<_, _>>();
    let realm = values
        .get("realm")
        .ok_or_else(|| anyhow!("Missing Docker token realm"))?;
    let mut url = reqwest::Url::parse(realm)?;
    if !matches!(url.scheme(), "http" | "https") {
        bail!("Docker token realm must use HTTP(S)");
    }
    for key in ["service", "scope"] {
        if let Some(value) = values.get(key) {
            url.query_pairs_mut().append_pair(key, value);
        }
    }
    let value: serde_json::Value = CLIENT
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    value
        .get("token")
        .or_else(|| value.get("access_token"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Docker upstream returned no bearer token"))
}
