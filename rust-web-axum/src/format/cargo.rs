use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoIndexConfig {
    pub dl: String,
    pub api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoIndexRecord {
    pub name: String,
    pub vers: String,
    pub deps: Vec<CargoIndexDependency>,
    pub cksum: String,
    pub features: serde_json::Value,
    pub yanked: bool,
    pub v: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoIndexDependency {
    pub name: String,
    pub req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoPublishMetadata {
    name: String,
    vers: String,
    #[serde(default)]
    deps: Vec<CargoPublishDependency>,
    #[serde(default = "empty_features")]
    features: Value,
}

#[derive(Debug, Deserialize)]
struct CargoPublishDependency {
    name: String,
    version_req: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    optional: bool,
    #[serde(default = "default_true")]
    default_features: bool,
    target: Option<String>,
    kind: Option<String>,
    registry: Option<String>,
    explicit_name_in_toml: Option<String>,
}

fn empty_features() -> Value {
    serde_json::json!({})
}

fn default_true() -> bool {
    true
}

impl CargoIndexRecord {
    /// Cargo publish metadata and sparse-index dependency records differ.
    pub fn from_publish_metadata(metadata: &Value, checksum: String) -> anyhow::Result<Self> {
        let published: CargoPublishMetadata = serde_json::from_value(metadata.clone())?;
        anyhow::ensure!(
            !published.name.is_empty() && !published.vers.is_empty(),
            "Cargo publish name and version must not be empty"
        );
        anyhow::ensure!(
            published.features.is_object(),
            "Cargo publish features must be a JSON object"
        );
        let deps = published
            .deps
            .into_iter()
            .map(|dependency| {
                let package = dependency
                    .explicit_name_in_toml
                    .as_ref()
                    .map(|_| dependency.name.clone());
                CargoIndexDependency {
                    name: dependency.explicit_name_in_toml.unwrap_or(dependency.name),
                    req: dependency.version_req,
                    features: dependency.features,
                    optional: dependency.optional,
                    default_features: dependency.default_features,
                    target: dependency.target,
                    kind: dependency.kind.unwrap_or_else(|| "normal".to_owned()),
                    registry: dependency.registry,
                    package,
                }
            })
            .collect();
        Ok(Self {
            name: published.name,
            vers: published.vers,
            deps,
            cksum: checksum,
            features: published.features,
            yanked: false,
            v: 2,
        })
    }
}

pub fn get_cargo_index_path(name: &str) -> String {
    let len = name.len();
    match len {
        1 => format!("1/{}", name),
        2 => format!("2/{}", name),
        3 => format!("3/{}/{}", &name[0..1], name),
        _ => format!("{}/{}/{}", &name[0..2], &name[2..4], name),
    }
}

#[cfg(test)]
mod tests {
    use super::CargoIndexRecord;

    #[test]
    fn publish_dependencies_keep_registry_rename_features_and_kinds() {
        let metadata = serde_json::json!({
            "name": "teaql-core",
            "vers": "4.3.6",
            "deps": [{
                "name": "teaql-macros",
                "version_req": "^4.3.6",
                "features": ["derive"],
                "optional": true,
                "default_features": false,
                "target": "cfg(unix)",
                "kind": "dev",
                "registry": null,
                "explicit_name_in_toml": "macros"
            }],
            "features": {"derive": ["dep:teaql-macros"]}
        });
        let record = CargoIndexRecord::from_publish_metadata(&metadata, "abc123".into()).unwrap();
        assert_eq!(record.v, 2);
        assert_eq!(record.deps[0].name, "macros");
        assert_eq!(record.deps[0].package.as_deref(), Some("teaql-macros"));
        assert_eq!(record.deps[0].req, "^4.3.6");
        assert_eq!(record.deps[0].kind, "dev");
        assert_eq!(record.deps[0].target.as_deref(), Some("cfg(unix)"));
        assert_eq!(record.features["derive"][0], "dep:teaql-macros");
    }

    #[test]
    fn malformed_features_do_not_become_an_empty_valid_index() {
        let metadata = serde_json::json!({
            "name": "teaql-core", "vers": "4.3.6", "deps": [], "features": []
        });
        assert!(CargoIndexRecord::from_publish_metadata(&metadata, "abc123".into()).is_err());
    }
}
