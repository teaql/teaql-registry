use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bytes::Bytes;
use hmac::{Hmac, Mac};
use md5::Md5;
use reqwest::{Client, Method, StatusCode};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::env;
use uuid::Uuid;

use super::traits::{BlobChecksums, BlobInfo, BlobStore};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            endpoint: env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9010".to_string()),
            region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: env::var("S3_ACCESS_KEY")
                .or_else(|_| env::var("AWS_ACCESS_KEY_ID"))
                .unwrap_or_else(|_| "rustfsadmin".to_string()),
            secret_key: env::var("S3_SECRET_KEY")
                .or_else(|_| env::var("AWS_SECRET_ACCESS_KEY"))
                .unwrap_or_else(|_| "rustfsadmin".to_string()),
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "teaql-blobs".to_string()),
        }
    }
}

fn sign_key(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

fn get_signature_key(secret_key: &str, date_stamp: &str, region: &str, service: &str) -> Vec<u8> {
    let k_secret = format!("AWS4{}", secret_key);
    let k_date = sign_key(k_secret.as_bytes(), date_stamp.as_bytes());
    let k_region = sign_key(&k_date, region.as_bytes());
    let k_service = sign_key(&k_region, service.as_bytes());
    sign_key(&k_service, b"aws4_request")
}

#[derive(Clone)]
pub struct S3BlobStore {
    config: S3Config,
    client: Client,
    store_name: String,
}

impl S3BlobStore {
    pub fn new(config: S3Config, store_name: impl Into<String>) -> Self {
        Self {
            config,
            client: Client::builder().build().expect("Failed to build reqwest client"),
            store_name: store_name.into(),
        }
    }

    pub fn from_env(store_name: impl Into<String>) -> Self {
        Self::new(S3Config::default(), store_name)
    }

    pub fn bucket(&self) -> &str {
        &self.config.bucket
    }

    fn get_object_key(&self, blob_id: &str) -> String {
        let prefix = if blob_id.len() >= 2 {
            &blob_id[0..2]
        } else {
            "00"
        };
        format!("{}/content/{}/{}", self.store_name, prefix, blob_id)
    }

    async fn send_s3_request(
        &self,
        method: Method,
        path: &str,
        query: Option<&str>,
        body: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<reqwest::Response> {
        let now = chrono::Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let endpoint_url = reqwest::Url::parse(&self.config.endpoint)?;
        let host = endpoint_url
            .host_str()
            .ok_or_else(|| anyhow!("Invalid S3 endpoint host"))?;
        let host_header = if let Some(port) = endpoint_url.port() {
            format!("{}:{}", host, port)
        } else {
            host.to_string()
        };

        let payload_hash = hex::encode(Sha256::digest(&body));

        let canonical_uri = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        let canonical_querystring = query.unwrap_or("");

        let canonical_headers = format!(
            "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
            host_header, payload_hash, amz_date
        );
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method.as_str(),
            canonical_uri,
            canonical_querystring,
            canonical_headers,
            signed_headers,
            payload_hash
        );

        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, self.config.region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            amz_date,
            credential_scope,
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );

        let signing_key = get_signature_key(
            &self.config.secret_key,
            &date_stamp,
            &self.config.region,
            "s3",
        );
        let signature = hex::encode(sign_key(&signing_key, string_to_sign.as_bytes()));

        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.config.access_key, credential_scope, signed_headers, signature
        );

        let url = if let Some(q) = query {
            format!("{}{}{}?{}", self.config.endpoint.trim_end_matches('/'), canonical_uri, "", q)
        } else {
            format!("{}{}", self.config.endpoint.trim_end_matches('/'), canonical_uri)
        };

        let mut req = self
            .client
            .request(method.clone(), &url)
            .header("Host", host_header)
            .header("x-amz-date", amz_date)
            .header("x-amz-content-sha256", payload_hash)
            .header("Authorization", authorization);

        if let Some(ct) = content_type {
            req = req.header("Content-Type", ct);
        }

        if method == Method::PUT || method == Method::POST {
            req = req.header("content-length", body.len().to_string());
            req = req.body(body);
        }

        let resp = req.send().await?;
        Ok(resp)
    }
}

#[async_trait]
impl BlobStore for S3BlobStore {
    async fn init(&self) -> Result<()> {
        let path = format!("/{}", self.config.bucket);
        let resp = self.send_s3_request(Method::HEAD, &path, None, vec![], None).await?;
        if resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::FORBIDDEN {
            let create_resp = self.send_s3_request(Method::PUT, &path, None, vec![], None).await?;
            if !create_resp.status().is_success() && create_resp.status() != StatusCode::CONFLICT {
                let err_body = create_resp.text().await.unwrap_or_default();
                return Err(anyhow!("Failed to create S3 bucket '{}': {}", self.config.bucket, err_body));
            }
        }
        Ok(())
    }

    async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo> {
        let blob_id = Uuid::new_v4().to_string();
        let blob_ref = format!("{}@{}", self.store_name, blob_id);
        let object_key = self.get_object_key(&blob_id);
        let path = format!("/{}/{}", self.config.bucket, object_key);

        let resp = self
            .send_s3_request(Method::PUT, &path, None, data.to_vec(), Some("application/octet-stream"))
            .await?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to upload blob to S3 '{}': {}", path, err_body));
        }

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

    async fn read_blob(&self, blob_ref: &str) -> Result<Bytes> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let object_key = self.get_object_key(blob_id);
        let path = format!("/{}/{}", self.config.bucket, object_key);

        let resp = self.send_s3_request(Method::GET, &path, None, vec![], None).await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Blob not found in S3: {}", blob_ref));
        }

        let bytes = resp.bytes().await?;
        Ok(bytes)
    }

    async fn delete_blob(&self, blob_ref: &str) -> Result<()> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let object_key = self.get_object_key(blob_id);
        let path = format!("/{}/{}", self.config.bucket, object_key);

        let resp = self.send_s3_request(Method::DELETE, &path, None, vec![], None).await?;
        if !resp.status().is_success() && resp.status() != StatusCode::NOT_FOUND {
            let err = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to delete S3 blob: {}", err));
        }
        Ok(())
    }

    async fn exists_blob(&self, blob_ref: &str) -> Result<bool> {
        let blob_id = blob_ref.split('@').nth(1).unwrap_or(blob_ref);
        let object_key = self.get_object_key(blob_id);
        let path = format!("/{}/{}", self.config.bucket, object_key);

        let resp = self.send_s3_request(Method::HEAD, &path, None, vec![], None).await?;
        Ok(resp.status().is_success())
    }

    fn store_name(&self) -> &str {
        &self.store_name
    }
}
