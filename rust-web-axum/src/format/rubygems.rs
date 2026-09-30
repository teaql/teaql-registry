use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RubyGemDependency {
    pub name: String,
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RubyGemMetadata {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub dependencies: Vec<RubyGemDependency>,
    pub required_ruby_version: String,
    pub required_rubygems_version: String,
    pub sha256: String,
    pub published_at: String,
}

impl RubyGemMetadata {
    pub fn version_platform(&self) -> String {
        if self.platform.is_empty() || self.platform == "ruby" {
            self.version.clone()
        } else {
            format!("{}-{}", self.version, self.platform)
        }
    }

    pub fn filename(&self) -> String {
        format!("{}-{}.gem", self.name, self.version_platform())
    }

    pub fn compact_info_line(&self) -> String {
        let dependencies = self
            .dependencies
            .iter()
            .map(|dependency| format!("{}:{}", dependency.name, dependency.requirement))
            .collect::<Vec<_>>()
            .join(",");
        let mut requirements = vec![format!("checksum:{}", self.sha256)];
        if !self.required_ruby_version.is_empty() {
            requirements.push(format!("ruby:{}", self.required_ruby_version));
        }
        if !self.required_rubygems_version.is_empty() {
            requirements.push(format!("rubygems:{}", self.required_rubygems_version));
        }
        requirements.push(format!("created_at:{}", self.published_at));
        format!(
            "{} {}|{}",
            self.version_platform(),
            dependencies,
            requirements.join(",")
        )
    }
}

fn yaml_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn top_level_scalar(lines: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    let index = lines
        .iter()
        .position(|line| !line.starts_with(char::is_whitespace) && line.starts_with(&prefix))?;
    let inline = lines[index].strip_prefix(&prefix)?.trim();
    if !inline.is_empty() && !inline.starts_with("!ruby/") {
        return Some(yaml_scalar(inline));
    }
    for line in lines.iter().skip(index + 1) {
        if !line.starts_with(char::is_whitespace) && !line.trim().is_empty() {
            break;
        }
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("version:") {
            return Some(yaml_scalar(value));
        }
    }
    None
}

fn requirement_from_lines(lines: &[&str]) -> String {
    let mut operator = None;
    let mut requirements = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("- - ") {
            operator = Some(yaml_scalar(value));
        } else if let Some(value) = trimmed.strip_prefix("version:") {
            if let Some(operator) = operator.take() {
                requirements.push(format!("{} {}", operator, yaml_scalar(value)));
            }
        }
    }
    requirements.join("&")
}

fn top_level_requirement(lines: &[&str], key: &str) -> String {
    let prefix = format!("{key}:");
    let Some(index) = lines
        .iter()
        .position(|line| !line.starts_with(char::is_whitespace) && line.starts_with(&prefix))
    else {
        return String::new();
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(index + 1)
        .find(|(_, line)| !line.starts_with(char::is_whitespace) && !line.trim().is_empty())
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    requirement_from_lines(&lines[index..end])
}

fn runtime_dependencies(lines: &[&str]) -> Vec<RubyGemDependency> {
    let Some(start) = lines.iter().position(|line| *line == "dependencies:") else {
        return Vec::new();
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| {
            !line.starts_with(char::is_whitespace)
                && !line.starts_with('-')
                && !line.trim().is_empty()
        })
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    let dependency_lines = &lines[start + 1..end];
    let starts = dependency_lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            line.trim_start()
                .starts_with("- !ruby/object:Gem::Dependency")
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let mut dependencies = Vec::new();
    for (position, start) in starts.iter().copied().enumerate() {
        let end = starts
            .get(position + 1)
            .copied()
            .unwrap_or(dependency_lines.len());
        let block = &dependency_lines[start..end];
        let name = block.iter().find_map(|line| {
            line.trim()
                .strip_prefix("name:")
                .map(yaml_scalar)
                .filter(|value| !value.is_empty())
        });
        let dependency_type = block
            .iter()
            .find_map(|line| line.trim().strip_prefix("type:").map(yaml_scalar))
            .unwrap_or_else(|| ":runtime".to_string());
        if dependency_type != ":runtime" && dependency_type != "runtime" {
            continue;
        }
        let requirement = requirement_from_lines(block);
        if let Some(name) = name {
            dependencies.push(RubyGemDependency {
                name,
                requirement: if requirement.is_empty() {
                    ">= 0".to_string()
                } else {
                    requirement
                },
            });
        }
    }
    dependencies
}

pub fn extract_rubygem_metadata(gem: &[u8]) -> Result<RubyGemMetadata> {
    let mut archive = tar::Archive::new(Cursor::new(gem));
    let mut metadata_gz = None;
    for entry in archive
        .entries()
        .context("file is not a valid RubyGem archive")?
    {
        let mut entry = entry.context("invalid RubyGem tar entry")?;
        if entry.path()?.as_ref() != std::path::Path::new("metadata.gz") {
            continue;
        }
        if entry.size() > 4 * 1024 * 1024 {
            bail!("RubyGem metadata exceeds 4 MiB");
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        metadata_gz = Some(bytes);
        break;
    }
    let metadata_gz = metadata_gz.context("RubyGem does not contain metadata.gz")?;
    let mut yaml = String::new();
    GzDecoder::new(Cursor::new(metadata_gz))
        .read_to_string(&mut yaml)
        .context("RubyGem metadata is not valid gzip UTF-8 YAML")?;
    let lines = yaml.lines().collect::<Vec<_>>();
    let name = top_level_scalar(&lines, "name").context("RubyGem metadata is missing name")?;
    let version =
        top_level_scalar(&lines, "version").context("RubyGem metadata is missing version")?;
    let platform = top_level_scalar(&lines, "platform").unwrap_or_else(|| "ruby".to_string());
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        bail!("invalid RubyGem name: {name}");
    }
    if version.is_empty()
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        bail!("invalid RubyGem version: {version}");
    }
    Ok(RubyGemMetadata {
        name,
        version,
        platform,
        dependencies: runtime_dependencies(&lines),
        required_ruby_version: top_level_requirement(&lines, "required_ruby_version"),
        required_rubygems_version: top_level_requirement(&lines, "required_rubygems_version"),
        sha256: String::new(),
        published_at: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    fn gem() -> Vec<u8> {
        let metadata = br#"--- !ruby/object:Gem::Specification
name: teaql_probe
version: !ruby/object:Gem::Version
  version: 1.2.3
platform: ruby
dependencies:
- !ruby/object:Gem::Dependency
  name: rack
  requirement: !ruby/object:Gem::Requirement
    requirements:
    - - ">="
      - !ruby/object:Gem::Version
        version: '2.0'
  type: :runtime
required_ruby_version: !ruby/object:Gem::Requirement
  requirements:
  - - ">="
    - !ruby/object:Gem::Version
      version: 3.0.0
"#;
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        gzip.write_all(metadata).unwrap();
        let metadata_gz = gzip.finish().unwrap();
        let mut output = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut output);
            let mut header = tar::Header::new_gnu();
            header.set_size(metadata_gz.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "metadata.gz", &metadata_gz[..])
                .unwrap();
            builder.finish().unwrap();
        }
        output
    }

    #[test]
    fn extracts_name_version_and_platform() {
        let metadata = extract_rubygem_metadata(&gem()).unwrap();
        assert_eq!(metadata.name, "teaql_probe");
        assert_eq!(metadata.version, "1.2.3");
        assert_eq!(metadata.filename(), "teaql_probe-1.2.3.gem");
        assert_eq!(
            metadata.dependencies,
            vec![RubyGemDependency {
                name: "rack".to_string(),
                requirement: ">= 2.0".to_string(),
            }]
        );
        assert_eq!(metadata.required_ruby_version, ">= 3.0.0");
    }
}
