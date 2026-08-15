use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use teaql_registry_core::{service_runtime_from_env, ServiceRuntime};
use teaql_registry::{
    api::{build_app, AppState},
    blobstore::{BlobStore, MemoryBlobStore, S3BlobStore},
    context::RegistryContextExt,
    security::password::hash_password,
    services::{BlobStoreService, RepositoryService, SecurityService},
};

async fn seed_initial_data(runtime: &ServiceRuntime) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Default Blob Store
    let blob_stores = BlobStoreService::list(runtime).await?;
    let default_blob_store = if let Some(bs) = blob_stores.into_iter().find(|s| s.name() == "default") {
        bs
    } else {
        BlobStoreService::create(runtime, "default", "s3://teaql-blobs/default", true).await?
    };

    // 2. Default Repositories
    let repos = RepositoryService::list(runtime).await?;
    if !repos.iter().any(|r| r.name() == "maven-releases") {
        RepositoryService::create(
            runtime,
            "maven-releases",
            "maven2-hosted",
            "HOSTED",
            "MAVEN2",
            "ALLOW_ONCE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: maven-releases");
    }

    if !repos.iter().any(|r| r.name() == "maven-snapshots") {
        RepositoryService::create(
            runtime,
            "maven-snapshots",
            "maven2-hosted",
            "HOSTED",
            "MAVEN2",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: maven-snapshots");
    }

    if !repos.iter().any(|r| r.name() == "maven-central") {
        RepositoryService::create(
            runtime,
            "maven-central",
            "maven2-proxy",
            "PROXY",
            "MAVEN2",
            "READ_ONLY",
            default_blob_store.id(),
            true,
            "https://repo1.maven.org/maven2",
        )
        .await?;
        info!("Seeded default repository: maven-central");
    }

    if !repos.iter().any(|r| r.name() == "maven-public") {
        RepositoryService::create(
            runtime,
            "maven-public",
            "maven2-group",
            "GROUP",
            "MAVEN2",
            "READ_ONLY",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: maven-public");
    }

    if !repos.iter().any(|r| r.name() == "raw-hosted") {
        RepositoryService::create(
            runtime,
            "raw-hosted",
            "raw-hosted",
            "HOSTED",
            "RAW",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: raw-hosted");
    }

    if !repos.iter().any(|r| r.name() == "docker-hosted") {
        RepositoryService::create(
            runtime,
            "docker-hosted",
            "docker-hosted",
            "HOSTED",
            "DOCKER",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: docker-hosted");
    }

    if !repos.iter().any(|r| r.name() == "npm-hosted") {
        RepositoryService::create(
            runtime,
            "npm-hosted",
            "npm-hosted",
            "HOSTED",
            "NPM",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: npm-hosted");
    }

    if !repos.iter().any(|r| r.name() == "pypi-hosted") {
        RepositoryService::create(
            runtime,
            "pypi-hosted",
            "pypi-hosted",
            "HOSTED",
            "PYPI",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: pypi-hosted");
    }

    if !repos.iter().any(|r| r.name() == "gomod-hosted") {
        RepositoryService::create(
            runtime,
            "gomod-hosted",
            "gomod-hosted",
            "HOSTED",
            "GOMOD",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: gomod-hosted");
    }

    if !repos.iter().any(|r| r.name() == "cargo-hosted") {
        RepositoryService::create(
            runtime,
            "cargo-hosted",
            "cargo-hosted",
            "HOSTED",
            "CARGO",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: cargo-hosted");
    }

    if !repos.iter().any(|r| r.name() == "nuget-hosted") {
        RepositoryService::create(
            runtime,
            "nuget-hosted",
            "nuget-hosted",
            "HOSTED",
            "NUGET",
            "ALLOW_WRITE",
            default_blob_store.id(),
            true,
            "",
        )
        .await?;
        info!("Seeded default repository: nuget-hosted");
    }

    // 3. Default Admin User
    let users = SecurityService::list_users(runtime).await?;
    if !users.iter().any(|u| u.username() == "admin") {
        let password_hash = hash_password("admin123");
        SecurityService::create_user(
            runtime,
            "admin",
            "Administrator",
            "User",
            "admin@example.com",
            &password_hash,
        )
        .await?;
        info!("Seeded default user: admin (password: admin123)");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!(
        "[{}] Starting TeaQL Registry (Rust + TeaQL)...",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
    );

    // 1. Initialize TeaQL Service Runtime connecting directly to PostgreSQL
    let runtime = service_runtime_from_env().await?;
    runtime.ensure_schema().await?;
    info!("TeaQL PostgreSQL schema verified and synchronized.");

    // 2. Initialize Blob Store (Pure In-Memory Mode or Persistent S3/RustFS)
    let args: Vec<String> = std::env::args().collect();
    let memory_mode = args.iter().any(|arg| arg == "--memory-mode" || arg == "--in-memory" || arg == "-m")
        || std::env::var("MEMORY_MODE").map(|v| v == "true" || v == "1").unwrap_or(false)
        || std::env::var("STORAGE_MODE").map(|v| v.to_lowercase() == "memory").unwrap_or(false);

    let blobstore: Arc<dyn BlobStore> = if memory_mode {
        info!(">> PURE IN-MEMORY HIGH-PERFORMANCE MODE ACTIVE: Using volatile memory blob storage with single latest version retention <<");
        Arc::new(MemoryBlobStore::new("default"))
    } else {
        info!(">> PERSISTENT STORAGE MODE ACTIVE: Using S3 / RustFS blob storage <<");
        Arc::new(S3BlobStore::from_env("default"))
    };
    blobstore.init().await?;

    let mut runtime = runtime;
    runtime.init_registry_context(blobstore.clone());
    runtime.set_memory_mode(memory_mode);

    // 3. Seed baseline initial configuration and sample demo artifacts
    seed_initial_data(&runtime).await?;
    let _ = teaql_registry::services::seed_demo_artifacts(&runtime, blobstore.as_ref()).await;

    // 4. Assemble Axum Router
    let app_state = AppState {
        runtime: Arc::new(runtime),
        blobstore,
    };
    let app = build_app(app_state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8081);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("TeaQL Registry listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}