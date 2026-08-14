use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuGetResource {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuGetServiceIndex {
    pub version: String,
    pub resources: Vec<NuGetResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuGetPackageVersions {
    pub versions: Vec<String>,
}

pub fn create_nuget_service_index(base_url: &str) -> NuGetServiceIndex {
    NuGetServiceIndex {
        version: "3.0.0".to_string(),
        resources: vec![
            NuGetResource {
                id: format!("{}/v3/package", base_url.trim_end_matches('/')),
                resource_type: "PackagePublish/2.0.0".to_string(),
                comment: Some("Initial push endpoint".to_string()),
            },
            NuGetResource {
                id: format!("{}/v3/flatcontainer/", base_url.trim_end_matches('/')),
                resource_type: "PackageBaseAddress/3.0.0".to_string(),
                comment: Some("Base URL of Azure storage where NuGet packages are stored".to_string()),
            },
            NuGetResource {
                id: format!("{}/v3/query", base_url.trim_end_matches('/')),
                resource_type: "SearchQueryService".to_string(),
                comment: Some("Search endpoint".to_string()),
            },
            NuGetResource {
                id: format!("{}/v3/registration/", base_url.trim_end_matches('/')),
                resource_type: "RegistrationsBaseUrl/3.6.0".to_string(),
                comment: Some("Base URL of package registration".to_string()),
            },
        ],
    }
}
