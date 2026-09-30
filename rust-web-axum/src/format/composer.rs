use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Cursor, Read};

const MAX_COMPOSER_JSON_SIZE: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredComposerPackage {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub package: Value,
}

pub fn validate_composer_name(name: &str) -> Result<(&str, &str)> {
    let Some((vendor, package)) = name.split_once('/') else {
        bail!("Composer package name must be vendor/package");
    };
    let valid_segment = |segment: &str| {
        !segment.is_empty()
            && segment.len() <= 128
            && segment.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && segment
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_.-".contains(c))
    };
    if !valid_segment(vendor) || !valid_segment(package) {
        bail!("invalid Composer package name");
    }
    Ok((vendor, package))
}

pub fn validate_composer_version(version: &str) -> Result<()> {
    if version.is_empty()
        || version.len() > 128
        || version.contains('/')
        || version.contains("..")
        || version.chars().any(char::is_whitespace)
    {
        bail!("invalid Composer package version");
    }
    Ok(())
}

pub fn extract_composer_json(archive: &[u8]) -> Result<Value> {
    let cursor = Cursor::new(archive);
    let mut zip = zip::ZipArchive::new(cursor).context("package is not a valid ZIP archive")?;
    let mut selected = None;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let path = entry
            .enclosed_name()
            .context("archive contains an unsafe path")?;
        let is_manifest = path.file_name().is_some_and(|name| name == "composer.json");
        if !is_manifest || entry.is_dir() {
            continue;
        }
        if entry.size() > MAX_COMPOSER_JSON_SIZE {
            bail!("composer.json exceeds the 1 MiB limit");
        }
        let depth = path.components().count();
        if selected
            .as_ref()
            .is_some_and(|(selected_depth, _): &(usize, Value)| *selected_depth <= depth)
        {
            continue;
        }
        let mut contents = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut contents)?;
        let json: Value =
            serde_json::from_slice(&contents).context("composer.json is not valid JSON")?;
        if !json.is_object() {
            bail!("composer.json must contain a JSON object");
        }
        selected = Some((depth, json));
    }
    selected
        .map(|(_, json)| json)
        .context("ZIP archive does not contain composer.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_manifest_from_wrapped_zip() {
        let mut bytes = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(Cursor::new(&mut bytes));
            writer
                .start_file(
                    "project/composer.json",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            writer
                .write_all(br#"{"name":"teaql/example","description":"probe"}"#)
                .unwrap();
            writer.finish().unwrap();
        }
        let manifest = extract_composer_json(&bytes).unwrap();
        assert_eq!(manifest["name"], "teaql/example");
    }

    #[test]
    fn validates_names_and_versions() {
        assert_eq!(
            validate_composer_name("teaql/example").unwrap(),
            ("teaql", "example")
        );
        assert!(validate_composer_name("TeaQL/example").is_err());
        assert!(validate_composer_version("1.2.3-RC1").is_ok());
        assert!(validate_composer_version("../bad").is_err());
    }
}
