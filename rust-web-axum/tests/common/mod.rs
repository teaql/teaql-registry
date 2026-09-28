use std::future::Future;
use teaql_registry_core::ServiceRuntimeConfig;

#[allow(dead_code)] // Each integration-test crate uses this only when it exercises generated TeaQL paths.
pub fn run_with_large_stack<Factory, Fut>(factory: Factory)
where
    Factory: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + 'static,
{
    let test_thread = std::thread::Builder::new()
        .name("registry-integration".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(32 * 1024 * 1024)
                .build()
                .expect("Registry integration runtime");
            runtime.block_on(factory());
        })
        .expect("Registry integration thread");
    test_thread.join().expect("Registry integration panic");
}

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
