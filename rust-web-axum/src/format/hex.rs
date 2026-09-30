use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};

const MAX_HEX_METADATA_SIZE: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexDependency {
    pub package: String,
    pub requirement: String,
    pub optional: bool,
    pub app: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredHexPackage {
    pub name: String,
    pub version: String,
    pub app: String,
    pub description: String,
    pub licenses: Vec<String>,
    pub build_tools: Vec<String>,
    pub dependencies: Vec<HexDependency>,
    pub inner_checksum: String,
    pub outer_checksum: String,
    pub published_at: i64,
}

fn binary_field(metadata: &str, field: &str) -> Option<String> {
    let pattern = format!(
        r#"(?s)\{{<<"{}">>,\s*<<"([^"]*)">>\}}\."#,
        regex::escape(field)
    );
    Regex::new(&pattern)
        .ok()?
        .captures(metadata)?
        .get(1)
        .map(|value| value.as_str().to_string())
}

fn binary_list_field(metadata: &str, field: &str) -> Vec<String> {
    let pattern = format!(r#"(?s)\{{<<"{}">>,\s*\[(.*?)\]\}}\."#, regex::escape(field));
    let Some(contents) = Regex::new(&pattern)
        .ok()
        .and_then(|regex| regex.captures(metadata))
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
    else {
        return Vec::new();
    };
    Regex::new(r#"<<"([^"]*)">>"#)
        .expect("static Hex binary regex")
        .captures_iter(&contents)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect()
}

fn dependencies(metadata: &str) -> Vec<HexDependency> {
    let Some(contents) = Regex::new(r#"(?s)\{<<"requirements">>,\s*\[(.*?)\]\}\."#)
        .expect("static Hex requirements regex")
        .captures(metadata)
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
    else {
        return Vec::new();
    };
    let entry =
        Regex::new(r#"(?s)\{<<"([^"]+)">>,\s*\[(.*?)\]\}"#).expect("static Hex dependency regex");
    let field = Regex::new(r#"\{<<"([^"]+)">>,\s*<<"([^"]*)">>\}"#)
        .expect("static Hex dependency field regex");
    entry
        .captures_iter(&contents)
        .filter_map(|captures| {
            let package = captures.get(1)?.as_str().to_string();
            let body = captures.get(2)?.as_str();
            let values = field
                .captures_iter(body)
                .filter_map(|field_capture| {
                    Some((
                        field_capture.get(1)?.as_str().to_string(),
                        field_capture.get(2)?.as_str().to_string(),
                    ))
                })
                .collect::<BTreeMap<_, _>>();
            let requirement = values.get("requirement")?.clone();
            Some(HexDependency {
                package,
                requirement,
                optional: body.contains("{<<\"optional\">>,true}"),
                app: values.get("app").cloned(),
                repository: values.get("repository").cloned(),
            })
        })
        .collect()
}

pub fn validate_hex_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 100
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        bail!("invalid Hex package name");
    }
    Ok(())
}

pub fn extract_hex_package(archive: &[u8]) -> Result<StoredHexPackage> {
    let mut files = BTreeMap::<String, Vec<u8>>::new();
    let mut tar = tar::Archive::new(Cursor::new(archive));
    for entry in tar
        .entries()
        .context("package is not a valid Hex tarball")?
    {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().into_owned();
        if !matches!(
            path.as_str(),
            "VERSION" | "metadata.config" | "contents.tar.gz" | "CHECKSUM"
        ) {
            continue;
        }
        if entry.size() > MAX_HEX_METADATA_SIZE && path != "contents.tar.gz" {
            bail!("Hex package metadata exceeds the 2 MiB limit");
        }
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents)?;
        if files.insert(path.clone(), contents).is_some() {
            bail!("Hex package contains duplicate {path}");
        }
    }
    let version_marker = files
        .get("VERSION")
        .context("Hex package is missing VERSION")?;
    if version_marker.as_slice() != b"3" {
        bail!("unsupported Hex package tarball version");
    }
    let metadata_bytes = files
        .get("metadata.config")
        .context("Hex package is missing metadata.config")?;
    let contents = files
        .get("contents.tar.gz")
        .context("Hex package is missing contents.tar.gz")?;
    let declared_checksum = String::from_utf8_lossy(
        files
            .get("CHECKSUM")
            .context("Hex package is missing CHECKSUM")?,
    )
    .trim()
    .to_ascii_uppercase();
    let mut inner_hasher = Sha256::new();
    inner_hasher.update(version_marker);
    inner_hasher.update(metadata_bytes);
    inner_hasher.update(contents);
    let inner = inner_hasher.finalize();
    let inner_hex = hex::encode_upper(inner);
    if declared_checksum != inner_hex {
        bail!("Hex package inner checksum mismatch");
    }
    let metadata = std::str::from_utf8(metadata_bytes).context("metadata.config is not UTF-8")?;
    let name = binary_field(metadata, "name").context("Hex metadata is missing name")?;
    let version = binary_field(metadata, "version").context("Hex metadata is missing version")?;
    validate_hex_name(&name)?;
    semver::Version::parse(&version).context("invalid Hex semantic version")?;
    Ok(StoredHexPackage {
        app: binary_field(metadata, "app").unwrap_or_else(|| name.clone()),
        description: binary_field(metadata, "description").unwrap_or_default(),
        licenses: binary_list_field(metadata, "licenses"),
        build_tools: binary_list_field(metadata, "build_tools"),
        dependencies: dependencies(metadata),
        name,
        version,
        inner_checksum: inner_hex.to_ascii_lowercase(),
        outer_checksum: hex::encode(Sha256::digest(archive)),
        published_at: chrono::Utc::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    fn package() -> Vec<u8> {
        let metadata = br#"{<<"app">>,<<"teaql_hex">>}.
{<<"build_tools">>,[<<"mix">>]}.
{<<"description">>,<<"TeaQL Hex probe">>}.
{<<"licenses">>,[<<"Apache-2.0">>]}.
{<<"name">>,<<"teaql_hex">>}.
{<<"requirements">>,[{<<"decimal">>,[{<<"app">>,<<"decimal">>},{<<"optional">>,false},{<<"requirement">>,<<"~> 2.0">>}]}]}.
{<<"version">>,<<"1.2.3">>}.
"#;
        let mut contents_encoder = GzEncoder::new(Vec::new(), Compression::default());
        contents_encoder.write_all(b"contents").unwrap();
        let contents = contents_encoder.finish().unwrap();
        let mut hasher = Sha256::new();
        hasher.update(b"3");
        hasher.update(metadata);
        hasher.update(&contents);
        let checksum = hex::encode_upper(hasher.finalize());
        let mut archive = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut archive);
            for (name, value) in [
                ("VERSION", b"3".as_slice()),
                ("CHECKSUM", checksum.as_bytes()),
                ("metadata.config", metadata.as_slice()),
                ("contents.tar.gz", contents.as_slice()),
            ] {
                let mut header = tar::Header::new_gnu();
                header.set_size(value.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                builder.append_data(&mut header, name, value).unwrap();
            }
            builder.finish().unwrap();
        }
        archive
    }

    #[test]
    fn parses_and_validates_hex_package() {
        let package = extract_hex_package(&package()).unwrap();
        assert_eq!(package.name, "teaql_hex");
        assert_eq!(package.version, "1.2.3");
        assert_eq!(package.dependencies[0].package, "decimal");
        assert_eq!(package.dependencies[0].requirement, "~> 2.0");
    }
}
