use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwiftReleaseSignature {
    pub signature_base64_encoded: String,
    pub signature_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwiftReleaseResource {
    pub name: String,
    #[serde(rename = "type")]
    pub content_type: String,
    pub checksum: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing: Option<SwiftReleaseSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwiftReleaseMetadata {
    pub id: String,
    pub version: String,
    pub resources: Vec<SwiftReleaseResource>,
    pub metadata: serde_json::Value,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwiftPackageReleases {
    pub releases: BTreeMap<String, SwiftPackageRelease>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwiftPackageRelease {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredSwiftRelease {
    pub metadata: serde_json::Value,
    pub published_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_archive_signature: Option<SwiftReleaseSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwiftManifest {
    pub filename: String,
    pub contents: Vec<u8>,
    pub tools_version: Option<String>,
}

pub fn validate_package_identity(scope: &str, name: &str) -> Result<()> {
    let valid = |value: &str| {
        !value.is_empty()
            && value.len() <= 100
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    };
    if !valid(scope) || !valid(name) {
        bail!(
            "invalid Swift package identity; scope and name must contain only ASCII letters, digits, '-' or '_'"
        );
    }
    Ok(())
}

pub fn validate_version(version: &str) -> Result<()> {
    semver::Version::parse(version)
        .with_context(|| format!("invalid semantic version: {version}"))?;
    Ok(())
}

pub fn extract_swift_manifests(source_archive: &[u8]) -> Result<Vec<SwiftManifest>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(source_archive))
        .context("source archive is not a valid ZIP file")?;
    let mut manifests = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let path = entry.name();
        let filename = match path.split_once('/') {
            Some((root, relative)) if !root.is_empty() && root != "." && root != ".." => {
                if relative.contains('/') {
                    continue;
                }
                relative
            }
            Some(_) => continue,
            None => path,
        };
        if filename.is_empty() {
            continue;
        }
        let filename = filename.to_string();
        if filename != "Package.swift"
            && !(filename.starts_with("Package@swift-") && filename.ends_with(".swift"))
        {
            continue;
        }
        if manifests
            .iter()
            .any(|item: &SwiftManifest| item.filename == filename)
        {
            bail!("package contains duplicate root manifest {filename}");
        }
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents)?;
        let tools_version = std::str::from_utf8(&contents)
            .ok()
            .and_then(parse_tools_version);
        manifests.push(SwiftManifest {
            filename,
            contents,
            tools_version,
        });
    }
    if !manifests
        .iter()
        .any(|item| item.filename == "Package.swift")
    {
        bail!("package doesn't contain a valid Package.swift manifest");
    }
    Ok(manifests)
}

fn parse_tools_version(contents: &str) -> Option<String> {
    contents.lines().take(5).find_map(|line| {
        let value = line.trim().strip_prefix("// swift-tools-version:")?.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

pub fn manifest_filename_for_swift_version(swift_version: &str) -> String {
    format!("Package@swift-{swift_version}.swift")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{write::SimpleFileOptions, ZipWriter};

    fn package_archive() -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("teaql.Probe/Package.swift", SimpleFileOptions::default())
            .unwrap();
        writer
            .write_all(b"// swift-tools-version: 5.9\nimport PackageDescription\n")
            .unwrap();
        writer
            .start_file(
                "teaql.Probe/Package@swift-5.8.swift",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer
            .write_all(b"// swift-tools-version:5.8\nimport PackageDescription\n")
            .unwrap();
        writer
            .start_file(
                "teaql.Probe/Examples/SchoolManagement/Package.swift",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer
            .write_all(b"// swift-tools-version: 6.0\n// nested package must be ignored\n")
            .unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn extracts_default_and_versioned_manifests() {
        let manifests = extract_swift_manifests(&package_archive()).unwrap();
        assert_eq!(manifests.len(), 2);
        assert_eq!(manifests[0].filename, "Package.swift");
        assert_eq!(manifests[0].tools_version.as_deref(), Some("5.9"));
        assert!(!String::from_utf8_lossy(&manifests[0].contents).contains("nested package"));
        assert_eq!(manifests[1].tools_version.as_deref(), Some("5.8"));
    }

    #[test]
    fn rejects_duplicate_root_manifest() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for root in ["teaql.Probe", "other"] {
            writer
                .start_file(
                    format!("{root}/Package.swift"),
                    SimpleFileOptions::default(),
                )
                .unwrap();
            writer.write_all(b"// swift-tools-version: 6.0\n").unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();
        assert!(extract_swift_manifests(&bytes)
            .unwrap_err()
            .to_string()
            .contains("duplicate root manifest"));
    }

    #[test]
    fn rejects_archive_without_manifest() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("README.md", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"missing manifest").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        assert!(extract_swift_manifests(&bytes).is_err());
    }
}
