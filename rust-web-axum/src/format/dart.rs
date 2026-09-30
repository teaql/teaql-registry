use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DartPackageVersion {
    pub version: String,
    pub archive_url: String,
    pub archive_sha256: String,
    pub pubspec: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DartPackageVersions {
    pub name: String,
    pub latest: DartPackageVersion,
    pub versions: Vec<DartPackageVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredDartPackage {
    pub name: String,
    pub version: String,
    pub pubspec: serde_json::Value,
    pub archive_sha256: String,
    pub published_at: String,
}

pub fn validate_dart_package_name(name: &str) -> Result<()> {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        bail!("Dart package name is empty");
    };
    if !(first.is_ascii_lowercase() || first == b'_')
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        bail!("invalid Dart package name: {name}");
    }
    Ok(())
}

pub fn extract_dart_pubspec(archive: &[u8]) -> Result<(String, String, serde_json::Value)> {
    let decoder = GzDecoder::new(Cursor::new(archive));
    let mut tar = tar::Archive::new(decoder);
    let mut pubspec = None;
    for entry in tar
        .entries()
        .context("package is not a valid .tar.gz archive")?
    {
        let mut entry = entry.context("invalid tar entry")?;
        let path = entry.path().context("invalid archive path")?;
        let is_pubspec = path.file_name().is_some_and(|name| name == "pubspec.yaml")
            && path.components().count() <= 2;
        if !is_pubspec {
            continue;
        }
        if entry.size() > 1024 * 1024 {
            bail!("pubspec.yaml exceeds 1 MiB");
        }
        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .context("pubspec.yaml is not UTF-8")?;
        pubspec = Some(contents);
        break;
    }
    let pubspec = pubspec.context("package archive does not contain a root pubspec.yaml")?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&pubspec).context("invalid pubspec.yaml")?;
    let json = serde_json::to_value(yaml).context("pubspec cannot be represented as JSON")?;
    let name = json
        .get("name")
        .and_then(serde_json::Value::as_str)
        .context("pubspec.yaml is missing name")?
        .to_string();
    let version = json
        .get("version")
        .and_then(serde_json::Value::as_str)
        .context("pubspec.yaml is missing version")?
        .to_string();
    validate_dart_package_name(&name)?;
    semver::Version::parse(&version)
        .with_context(|| format!("invalid Dart package version: {version}"))?;
    Ok((name, version, json))
}

pub fn dart_archive_sha256(archive: &[u8]) -> String {
    hex::encode(Sha256::digest(archive))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{write::GzEncoder, Compression};

    fn package() -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = tar::Builder::new(&mut encoder);
            let body = b"name: teaql_probe\nversion: 1.2.3\ndescription: test package\n";
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "pubspec.yaml", &body[..])
                .unwrap();
            builder.finish().unwrap();
        }
        encoder.finish().unwrap()
    }

    #[test]
    fn extracts_pubspec_from_package() {
        let (name, version, pubspec) = extract_dart_pubspec(&package()).unwrap();
        assert_eq!(name, "teaql_probe");
        assert_eq!(version, "1.2.3");
        assert_eq!(pubspec["description"], "test package");
    }

    #[test]
    fn validates_package_names() {
        assert!(validate_dart_package_name("teaql_probe2").is_ok());
        assert!(validate_dart_package_name("TeaQL-Probe").is_err());
    }
}
