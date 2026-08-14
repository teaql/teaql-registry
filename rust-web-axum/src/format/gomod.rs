use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoModuleVersionInfo {
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Time")]
    pub time: String,
}

pub fn parse_gomod_path(path: &str) -> Option<(String, String, String)> {
    // path: /<module>/@v/<target>
    // e.g. /github.com/gin-gonic/gin/@v/v1.9.1.info -> ("github.com/gin-gonic/gin", "v1.9.1", "info")
    // e.g. /github.com/gin-gonic/gin/@v/list -> ("github.com/gin-gonic/gin", "", "list")
    let clean = path.trim_matches('/');
    let (module, rest) = clean.split_once("/@v/")?;

    if rest == "list" {
        return Some((module.to_string(), String::new(), "list".to_string()));
    }

    if let Some((ver, ext)) = rest.rsplit_once('.') {
        return Some((module.to_string(), ver.to_string(), ext.to_string()));
    }

    None
}
