use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DOCKER_MANIFEST_V2_MEDIA_TYPE: &str = "application/vnd.docker.distribution.manifest.v2+json";
pub const OCI_MANIFEST_V1_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
pub const DOCKER_CONFIG_JSON_MEDIA_TYPE: &str = "application/vnd.docker.container.image.v1+json";
pub const OCI_CONFIG_JSON_MEDIA_TYPE: &str = "application/vnd.oci.image.config.v1+json";
pub const DOCKER_LAYER_GZIP_MEDIA_TYPE: &str = "application/vnd.docker.image.rootfs.diff.tar.gzip";
pub const OCI_LAYER_GZIP_MEDIA_TYPE: &str = "application/vnd.oci.image.layer.v1.tar+gzip";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerDescriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub size: i64,
    pub digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerManifestV2 {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType", default = "default_manifest_media_type")]
    pub media_type: String,
    pub config: DockerDescriptor,
    pub layers: Vec<DockerDescriptor>,
}

fn default_manifest_media_type() -> String {
    DOCKER_MANIFEST_V2_MEDIA_TYPE.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerTagList {
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerErrorResponse {
    pub errors: Vec<DockerErrorDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

pub fn compute_sha256_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

pub fn is_valid_digest(digest: &str) -> bool {
    if let Some(hex_part) = digest.strip_prefix("sha256:") {
        hex_part.len() == 64 && hex_part.chars().all(|c| c.is_ascii_hexdigit())
    } else {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockerPath {
    BasePing,
    BlobsUploadInit { name: String },
    BlobsUploadChunk { name: String, uuid: String },
    Blob { name: String, digest: String },
    Manifest { name: String, reference: String },
    TagsList { name: String },
}

pub fn parse_docker_path(path: &str) -> Option<DockerPath> {
    let clean = path.trim_matches('/');
    let clean = if clean == "v2" {
        ""
    } else {
        clean.strip_prefix("v2/").unwrap_or(clean)
    };
    let clean = clean.trim_matches('/');

    if clean.is_empty() {
        return Some(DockerPath::BasePing);
    }

    let parts: Vec<&str> = clean.split('/').collect();

    // Check tags list: <name>/tags/list
    if parts.len() >= 3 && parts[parts.len() - 2] == "tags" && parts[parts.len() - 1] == "list" {
        let name = parts[0..parts.len() - 2].join("/");
        return Some(DockerPath::TagsList { name });
    }

    // Check blobs uploads init: <name>/blobs/uploads
    if parts.len() >= 3 && parts[parts.len() - 2] == "blobs" && parts[parts.len() - 1] == "uploads" {
        let name = parts[0..parts.len() - 2].join("/");
        return Some(DockerPath::BlobsUploadInit { name });
    }

    // Check blobs upload chunk/finish: <name>/blobs/uploads/<uuid>
    if parts.len() >= 4 && parts[parts.len() - 3] == "blobs" && parts[parts.len() - 2] == "uploads" {
        let name = parts[0..parts.len() - 3].join("/");
        let uuid = parts[parts.len() - 1].to_string();
        return Some(DockerPath::BlobsUploadChunk { name, uuid });
    }

    // Check blob: <name>/blobs/<digest>
    if parts.len() >= 3 && parts[parts.len() - 2] == "blobs" {
        let name = parts[0..parts.len() - 2].join("/");
        let digest = parts[parts.len() - 1].to_string();
        return Some(DockerPath::Blob { name, digest });
    }

    // Check manifest: <name>/manifests/<reference>
    if parts.len() >= 3 && parts[parts.len() - 2] == "manifests" {
        let name = parts[0..parts.len() - 2].join("/");
        let reference = parts[parts.len() - 1].to_string();
        return Some(DockerPath::Manifest { name, reference });
    }

    None
}
