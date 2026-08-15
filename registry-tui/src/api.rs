use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

use crate::types::{
    CleanupReport, CreateTokenResponse, GcReport, RepositoryItem, SearchResponse, ServerOverview,
};

pub struct RegistryClient {
    client: Client,
    endpoint: String,
    auth_header: Option<String>,
}

impl RegistryClient {
    pub fn new(endpoint: &str, username: Option<&str>, password: Option<&str>, token: Option<&str>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let auth_header = if let Some(tok) = token {
            Some(format!("Bearer {}", tok))
        } else if let (Some(u), Some(p)) = (username, password) {
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                format!("{}:{}", u, p),
            );
            Some(format!("Basic {}", encoded))
        } else {
            None
        };

        Self {
            client,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            auth_header,
        }
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(auth) = &self.auth_header {
            req.header(reqwest::header::AUTHORIZATION, auth)
        } else {
            req
        }
    }

    pub async fn fetch_overview(&self) -> Result<ServerOverview> {
        let status_url = format!("{}/service/rest/v1/status", self.endpoint);
        let is_online = match self.client.get(&status_url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        };

        let repos = self.fetch_repositories().await.unwrap_or_default();
        let total_repositories = repos.len();
        let mut hosted_count = 0;
        let mut proxy_count = 0;
        let mut group_count = 0;
        let mut fmt_map = std::collections::HashMap::new();

        for r in &repos {
            match r.repo_type.to_lowercase().as_str() {
                "hosted" => hosted_count += 1,
                "proxy" => proxy_count += 1,
                "group" => group_count += 1,
                _ => {}
            }
            *fmt_map.entry(r.format.to_uppercase()).or_insert(0) += 1;
        }

        let mut format_counts: Vec<(String, usize)> = fmt_map.into_iter().collect();
        format_counts.sort_by(|a, b| b.1.cmp(&a.1));

        let search = self.search_components("").await.unwrap_or_default();
        let total_components = search.total;

        let metrics_raw = self.fetch_metrics().await.unwrap_or_default();

        Ok(ServerOverview {
            status: if is_online { "HEALTHY / ONLINE".to_string() } else { "OFFLINE".to_string() },
            is_online,
            total_repositories,
            hosted_count,
            proxy_count,
            group_count,
            format_counts,
            total_components,
            metrics_raw,
        })
    }

    pub async fn fetch_repositories(&self) -> Result<Vec<RepositoryItem>> {
        let url = format!("{}/service/rest/v1/repositories", self.endpoint);
        let req = self.apply_auth(self.client.get(&url));
        let resp = req.send().await.context("Failed to connect to registry")?;
        let repos = resp.json::<Vec<RepositoryItem>>().await?;
        Ok(repos)
    }

    pub async fn search_components(&self, keyword: &str) -> Result<SearchResponse> {
        let url = if keyword.is_empty() {
            format!("{}/service/rest/v1/search?page_size=50", self.endpoint)
        } else {
            format!(
                "{}/service/rest/v1/search?keyword={}&page_size=50",
                self.endpoint,
                urlencoding_encode(keyword)
            )
        };
        let req = self.apply_auth(self.client.get(&url));
        let resp = req.send().await.context("Search failed")?;
        let search = resp.json::<SearchResponse>().await?;
        Ok(search)
    }

    pub async fn fetch_metrics(&self) -> Result<String> {
        let url = format!("{}/metrics", self.endpoint);
        let req = self.apply_auth(self.client.get(&url));
        let resp = req.send().await.context("Failed to fetch metrics")?;
        let text = resp.text().await?;
        Ok(text)
    }

    pub async fn run_gc(&self) -> Result<GcReport> {
        let url = format!("{}/service/rest/v1/gc/run", self.endpoint);
        let req = self.apply_auth(self.client.post(&url));
        let resp = req.send().await.context("GC failed")?;
        let report = resp.json::<GcReport>().await?;
        Ok(report)
    }

    pub async fn run_cleanup(&self, repo_name: &str, max_versions: usize) -> Result<CleanupReport> {
        let url = format!("{}/service/rest/v1/cleanup/run", self.endpoint);
        let body = serde_json::json!({
            "repository": repo_name,
            "max_versions_per_component": max_versions,
            "snapshot_only": false
        });
        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req.send().await.context("Cleanup failed")?;
        let report = resp.json::<CleanupReport>().await?;
        Ok(report)
    }

    pub async fn create_temp_token(&self, description: &str) -> Result<String> {
        let url = format!("{}/service/rest/v1/tokens", self.endpoint);
        let body = serde_json::json!({
            "username": "admin",
            "description": description,
            "scopes": ["read", "write", "admin"],
            "expires_in_days": 7
        });
        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req.send().await.context("Create token failed")?;
        let token_resp = resp.json::<CreateTokenResponse>().await?;
        Ok(token_resp.token)
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}
