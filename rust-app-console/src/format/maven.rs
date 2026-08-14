use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenCoordinates {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub file_name: String,
    pub extension: String,
    pub is_snapshot: bool,
    pub is_metadata: bool,
}

pub fn parse_maven_path(path: &str) -> Option<MavenCoordinates> {
    let clean_path = path.trim_start_matches('/');
    let parts: Vec<&str> = clean_path.split('/').collect();

    if parts.len() < 3 {
        return None;
    }

    let file_name = parts[parts.len() - 1].to_string();

    if file_name == "maven-metadata.xml" || file_name.starts_with("maven-metadata.xml.") {
        let artifact_id = parts[parts.len() - 2].to_string();
        let group_id = parts[0..parts.len() - 2].join(".");
        return Some(MavenCoordinates {
            group_id,
            artifact_id,
            version: "".to_string(),
            file_name,
            extension: "xml".to_string(),
            is_snapshot: false,
            is_metadata: true,
        });
    }

    if parts.len() < 4 {
        return None;
    }

    let version = parts[parts.len() - 2].to_string();
    let artifact_id = parts[parts.len() - 3].to_string();
    let group_id = parts[0..parts.len() - 3].join(".");
    let is_snapshot = version.to_uppercase().ends_with("-SNAPSHOT");

    let ext = file_name
        .rsplit_once('.')
        .map(|(_, e)| e.to_string())
        .unwrap_or_default();

    Some(MavenCoordinates {
        group_id,
        artifact_id,
        version,
        file_name,
        extension: ext,
        is_snapshot,
        is_metadata: false,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenMetadata {
    pub group_id: String,
    pub artifact_id: String,
    pub version: Option<String>,
    pub latest: Option<String>,
    pub release: Option<String>,
    pub versions: Vec<String>,
    pub last_updated: String,
}

pub fn generate_maven_metadata_xml(
    group_id: &str,
    artifact_id: &str,
    versions: &[String],
) -> String {
    let now = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let latest = versions.last().cloned().unwrap_or_default();
    let release = versions
        .iter()
        .filter(|v| !v.to_uppercase().ends_with("-SNAPSHOT"))
        .last()
        .cloned()
        .unwrap_or_else(|| latest.clone());

    let mut versions_xml = String::new();
    for v in versions {
        versions_xml.push_str(&format!("      <version>{}</version>\n", v));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata>
  <groupId>{}</groupId>
  <artifactId>{}</artifactId>
  <versioning>
    <latest>{}</latest>
    <release>{}</release>
    <versions>
{}    </versions>
    <lastUpdated>{}</lastUpdated>
  </versioning>
</metadata>
"#,
        group_id, artifact_id, latest, release, versions_xml, now
    )
}
