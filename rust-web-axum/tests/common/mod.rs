use teaql_registry_core::ServiceRuntimeConfig;

/// Use the same environment contract as the production binary and CI. The
/// fallback keeps local tests convenient when the standard test database is
/// available.
pub fn runtime_config() -> ServiceRuntimeConfig {
    ServiceRuntimeConfig::from_env().unwrap_or_else(|_| ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    })
}
