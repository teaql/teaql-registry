use serde::{Deserialize, Serialize};

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
