//! Disposable, localhost-only server for scripts/verify_proxy_clients.py.
use std::{str::FromStr, sync::Arc};
use teaql_registry::{
    build_app,
    services::{BlobStoreService, RepositoryService, TenantService},
    AppState, BlobStore, FileBlobStore,
};
use teaql_registry_core::{service_runtime_from_pool, ServiceRuntimeConfig};

fn main() -> anyhow::Result<()> {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(4)
                .thread_stack_size(32 * 1024 * 1024)
                .build()?
                .block_on(run())
        })?
        .join()
        .expect("validation server thread")
}
async fn run() -> anyhow::Result<()> {
    let config = ServiceRuntimeConfig::from_env()?;
    let mut pg = tokio_postgres::Config::from_str(&config.database_url)?;
    pg.user(&config.database_user)
        .password(&config.database_password);
    let manager = deadpool_postgres::Manager::new(pg, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(manager)
        .max_size(16)
        .build()?;
    let context = service_runtime_from_pool(pool.clone()).await?;
    context.ensure_schema().await?;
    if TenantService::find_tenant_by_code(&context, "default")
        .await?
        .is_none()
    {
        TenantService::create_tenant_with_platform(
            &context,
            1,
            "Default",
            "default",
            "Native proxy verification",
        )
        .await?;
    }
    let root = std::env::var("PROXY_TEST_BLOB_ROOT")?;
    let store: Arc<dyn BlobStore> = Arc::new(FileBlobStore::new(&root, "proxy-validation"));
    store.init().await?;
    let blob = match BlobStoreService::list(&context)
        .await?
        .into_iter()
        .find(|s| s.name() == "proxy-validation")
    {
        Some(blob) => blob,
        None => BlobStoreService::create(&context, "proxy-validation", &root, true).await?,
    };
    let gateway = std::env::var("PROXY_TEST_UPSTREAM_ROOT")?;
    let protocols = [
        ("cargo", "CARGO"),
        ("pypi", "PYPI"),
        ("gomod", "GOMOD"),
        ("nuget", "NUGET"),
        ("docker", "DOCKER"),
        ("swift", "SWIFT"),
        ("dart", "DART"),
        ("rubygems", "RUBYGEMS"),
        ("composer", "COMPOSER"),
        ("conan", "CONAN"),
        ("hex", "HEX"),
        ("raw", "RAW"),
        ("npm", "NPM"),
        ("maven2", "MAVEN2"),
    ];
    for (protocol, format) in protocols {
        let name = format!("proxy-{protocol}");
        if RepositoryService::find_by_name(&context, &name)
            .await?
            .is_none()
        {
            let remote = if protocol == "nuget" {
                format!("{gateway}/{protocol}/v3/index.json")
            } else {
                format!("{gateway}/{protocol}")
            };
            RepositoryService::create(
                &context,
                &name,
                &format!("{protocol}-proxy"),
                "PROXY",
                format,
                "READ_ONLY",
                blob.id(),
                true,
                &remote,
            )
            .await?;
        }
    }
    for (protocol, format) in [("swift", "SWIFT"), ("conan", "CONAN"), ("raw", "RAW")] {
        let name = format!("fixture-{protocol}");
        if RepositoryService::find_by_name(&context, &name)
            .await?
            .is_none()
        {
            RepositoryService::create(
                &context,
                &name,
                &format!("{protocol}-hosted"),
                "HOSTED",
                format,
                "ALLOW_WRITE",
                blob.id(),
                true,
                "",
            )
            .await?;
        }
    }
    let state = AppState::with_runtime_pool_unsecured_for_tests(Arc::new(context), pool, store);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:28441").await?;
    println!("PROXY_VALIDATION_READY http://127.0.0.1:28441");
    axum::serve(listener, build_app(state)).await?;
    Ok(())
}
