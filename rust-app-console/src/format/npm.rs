use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmDist {
    pub shasum: String,
    pub tarball: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmVersionDetail {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub dist: NpmDist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmAttachment {
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    pub data: String, // base64 encoded
    pub length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmPackageDocument {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, NpmVersionDetail>,
    #[serde(rename = "_attachments", default)]
    pub attachments: HashMap<String, NpmAttachment>,
}
