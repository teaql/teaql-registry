use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConanRevision {
    pub revision: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConanRevisions {
    pub revisions: Vec<ConanRevision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConanFileSnapshot {
    pub files: BTreeMap<String, serde_json::Value>,
}

pub fn validate_conan_coordinate(value: &str, field: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 256
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_whitespace)
    {
        bail!("invalid Conan {field}");
    }
    Ok(())
}

pub fn validate_conan_file_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains('\\')
    {
        bail!("invalid Conan file path");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_coordinates_and_paths() {
        assert!(validate_conan_coordinate("hello", "name").is_ok());
        assert!(validate_conan_coordinate("../hello", "name").is_err());
        assert!(validate_conan_file_path("metadata/sign/signature").is_ok());
        assert!(validate_conan_file_path("../secret").is_err());
    }
}
