#![recursion_limit = "256"]

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use teaql_registry::{
    api::{build_app, AppState},
    blobstore::{BlobStore, MemoryBlobStore, S3BlobStore},
    context::RegistryContextExt,
    schema_indexes::ensure_application_indexes,
    security::password::{hash_password, validate_password_strength, verify_password},
    services::{SecurityService, TenantService},
};
use teaql_registry_core::{service_runtime_from_pool, ServiceRuntime, ServiceRuntimeConfig};

const DEFAULT_CREDENTIALS_DIR: &str = "/var/lib/teaql-registry/credentials";

fn credentials_dir() -> String {
    std::env::var("CREDENTIALS_DIR").unwrap_or_else(|_| DEFAULT_CREDENTIALS_DIR.to_string())
}

async fn ensure_private_directory(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(path).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

async fn seed_initial_data(runtime: &ServiceRuntime) -> Result<(), Box<dyn std::error::Error>> {
    let default_tenant = TenantService::create_tenant(runtime, "Default Tenant", "default").await?;
    if default_tenant.id() != 1 {
        return Err(format!(
            "default tenant must own reserved tenant id 1, found {}",
            default_tenant.id()
        )
        .into());
    }

    let credentials_dir = credentials_dir();
    ensure_private_directory(&credentials_dir).await?;
    let credential_file = format!("{credentials_dir}/global-admin.txt");
    let configured_password = std::env::var("ADMIN_PASSWORD")
        .ok()
        .filter(|password| !password.trim().is_empty());
    let retained_password = tokio::fs::read_to_string(&credential_file)
        .await
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find_map(|line| line.trim().strip_prefix("admin / ").map(str::to_string))
        });
    let existing_admin =
        SecurityService::find_user_by_tenant_and_username(runtime, default_tenant.id(), "admin")
            .await?;
    let generated_password =
        existing_admin.is_none() && configured_password.is_none() && retained_password.is_none();
    let selected_password = configured_password.clone().or(retained_password);
    let (admin, known_password) = match existing_admin {
        Some(user) => match selected_password {
            Some(password) => {
                validate_password_strength(&password, &["admin", "teaql", "registry"])
                    .map_err(|error| format!("administrator password rejected: {error}"))?;
                let user = if verify_password(&password, &user.password_hash()) {
                    user
                } else {
                    SecurityService::replace_password_hash(runtime, user, &hash_password(&password))
                        .await?
                };
                (user, Some(password))
            }
            None => {
                info!(
                    "Existing administrator retained; set ADMIN_PASSWORD to perform an explicit credential reset."
                );
                (user, None)
            }
        },
        None => {
            let password = selected_password.unwrap_or_else(|| generate_random_password(20));
            validate_password_strength(&password, &["admin", "teaql", "registry"])
                .map_err(|error| format!("administrator password rejected: {error}"))?;
            let user = SecurityService::create_user_with_tenant(
                runtime,
                default_tenant.id(),
                "admin",
                "Administrator",
                "User",
                "admin@example.com",
                &hash_password(&password),
            )
            .await?;
            (user, Some(password))
        }
    };

    let blob_root =
        std::env::var("BLOB_ROOT_DIR").unwrap_or_else(|_| "s3://teaql-blobs".to_string());
    TenantService::provision_tenant(runtime, default_tenant.id(), &blob_root, &[]).await?;
    let admin_role =
        SecurityService::find_role_by_tenant_and_key(runtime, default_tenant.id(), "nx-admin")
            .await?
            .ok_or("default administrator role was not provisioned")?;
    let platform_admin = match SecurityService::find_privilege_by_tenant_and_key(
        runtime,
        default_tenant.id(),
        "platform-admin",
    )
    .await?
    {
        Some(existing) => existing,
        None => {
            SecurityService::create_privilege_with_tenant(
                runtime,
                default_tenant.id(),
                "platform-admin",
                "Platform Administration",
                "Create and inspect registry tenants",
                "platform",
                "platform:admin",
                true,
            )
            .await?
        }
    };
    SecurityService::assign_privilege(
        runtime,
        default_tenant.id(),
        admin_role.id(),
        platform_admin.id(),
    )
    .await?;
    SecurityService::assign_role(runtime, default_tenant.id(), admin.id(), admin_role.id()).await?;

    if configured_password.is_none() {
        if let Some(password) = known_password.as_deref() {
            write_credential_file(&credential_file, "admin", password).await?;
        }
        if generated_password {
            tracing::warn!(
                "Generated administrator credential is available at {}",
                credential_file
            );
        }
    } else {
        info!("Administrator credential reconciled from ADMIN_PASSWORD.");
    }
    Ok(())
}

async fn write_credential_file(
    path: &str,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = format!(
        "# TeaQL Registry bootstrap credential\n# Generated at {}\n{} / {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        username,
        password
    );
    tokio::fs::write(path, content).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

async fn reconcile_seeded_user(
    runtime: &ServiceRuntime,
    tenant_id: u64,
    username: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    requested_password: Option<String>,
) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    let existing =
        SecurityService::find_user_by_tenant_and_username(runtime, tenant_id, username).await?;
    if let (Some(user), None) = (&existing, &requested_password) {
        // Losing a bootstrap secret must never silently rotate a live account.
        return Ok((user.password_hash().to_string(), None));
    }

    let password = requested_password.unwrap_or_else(|| generate_random_password(20));
    validate_password_strength(&password, &[username, email, "teaql", "registry"])
        .map_err(|error| format!("bootstrap password for {username} rejected: {error}"))?;
    let password_hash = hash_password(&password);
    match existing {
        Some(user) if verify_password(&password, &user.password_hash()) => {}
        Some(user) => {
            SecurityService::replace_password_hash(runtime, user, &password_hash).await?;
        }
        None => {
            SecurityService::create_user_with_tenant(
                runtime,
                tenant_id,
                username,
                first_name,
                last_name,
                email,
                &password_hash,
            )
            .await?;
        }
    }
    Ok((password_hash, Some(password)))
}

async fn seed_tenants(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    use teaql_registry::services::TenantService;

    let creds_dir = credentials_dir();

    let blob_root =
        std::env::var("BLOB_ROOT_DIR").unwrap_or_else(|_| "s3://teaql-blobs".to_string());

    let tenant_defs = [
        ("Development", "development"),
        ("Test", "test"),
        ("UAT", "uat"),
    ];

    // Create credentials directory
    ensure_private_directory(&creds_dir).await?;

    let mut all_creds = String::new();
    all_creds.push_str("# TeaQL Registry - Initial Credentials\n");
    all_creds.push_str(&format!(
        "# Generated at {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    all_creds.push_str("# ⚠ Change these passwords after first login.\n\n");

    // Include global admin credentials if the file was written by seed_initial_data
    let global_admin_file = format!("{}/global-admin.txt", creds_dir);
    if let Ok(content) = tokio::fs::read_to_string(&global_admin_file).await {
        all_creds.push_str("## Global Admin (tenant_id=1)\n");
        for line in content.lines() {
            if !line.starts_with('#') && !line.is_empty() {
                all_creds.push_str(&format!("  {}\n", line));
            }
        }
        all_creds.push('\n');
    }

    for (name, code) in tenant_defs {
        let tenant = TenantService::create_tenant(&state.runtime, name, code).await?;
        let tenant_id = tenant.id();
        let runtime = state.request_runtime(tenant_id, name).await?;

        let tenant_file = format!("{}/{}.txt", creds_dir, code);
        let retained = tokio::fs::read_to_string(&tenant_file).await.ok();
        let admin_requested = std::env::var(format!(
            "TEAQL_REGISTRY_{}_ADMIN_PASSWORD",
            code.to_ascii_uppercase()
        ))
        .ok()
        .filter(|password| !password.trim().is_empty())
        .or_else(|| {
            retained
                .as_deref()
                .and_then(|content| credential_password(content, "admin"))
        });
        let developer_requested = std::env::var(format!(
            "TEAQL_REGISTRY_{}_DEVELOPER_PASSWORD",
            code.to_ascii_uppercase()
        ))
        .ok()
        .filter(|password| !password.trim().is_empty())
        .or_else(|| {
            retained
                .as_deref()
                .and_then(|content| credential_password(content, "developer"))
        });

        let admin_email = format!("admin@{}.local", code);
        let dev_email = format!("developer@{}.local", code);
        let (admin_hash, admin_pass) = reconcile_seeded_user(
            &runtime,
            tenant_id,
            "admin",
            "Administrator",
            "User",
            &admin_email,
            admin_requested,
        )
        .await?;
        let (dev_hash, dev_pass) = reconcile_seeded_user(
            &runtime,
            tenant_id,
            "developer",
            "Developer",
            "User",
            &dev_email,
            developer_requested,
        )
        .await?;

        let users = vec![
            (
                "admin",
                "Administrator",
                "User",
                admin_email.as_str(),
                admin_hash.as_str(),
            ),
            (
                "developer",
                "Developer",
                "User",
                dev_email.as_str(),
                dev_hash.as_str(),
            ),
        ];

        TenantService::provision_tenant(&runtime, tenant_id, &blob_root, &users).await?;

        // Persist only secrets that are actually known to match the database.
        let mut tenant_creds = format!("tenant: {}\n", code);
        if let Some(password) = &admin_pass {
            tenant_creds.push_str(&format!("  admin / {}\n", password));
        }
        if let Some(password) = &dev_pass {
            tenant_creds.push_str(&format!("  developer / {}\n", password));
        }
        if admin_pass.is_some() || dev_pass.is_some() {
            tokio::fs::write(&tenant_file, &tenant_creds).await?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&tenant_file, std::fs::Permissions::from_mode(0o600))?;
            }
        }

        all_creds.push_str(&format!("## Tenant: {} ({})\n", name, code));
        if let Some(password) = admin_pass {
            all_creds.push_str(&format!("  admin      / {}\n", password));
        } else {
            all_creds.push_str("  admin      / <existing credential retained>\n");
        }
        if let Some(password) = dev_pass {
            all_creds.push_str(&format!("  developer  / {}\n\n", password));
        } else {
            all_creds.push_str("  developer  / <existing credential retained>\n\n");
        }

        info!("Seeded tenant '{}' with 2 users.", code);
    }

    // Write combined credentials file
    let all_file = format!("{}/all-credentials.txt", creds_dir);
    tokio::fs::write(&all_file, &all_creds).await?;

    // Set file permissions to owner-only (0600) on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&all_file, perms)?;
    }

    tracing::warn!(
        "⚠ Initial tenant credentials written to: {}\n  \
         Read {}/all-credentials.txt for all passwords.\n  \
         Keep this directory private and persistent; deleting it removes the retained bootstrap secrets.",
        creds_dir,
        creds_dir
    );

    Ok(())
}

fn credential_password(content: &str, username: &str) -> Option<String> {
    let prefix = format!("{username} / ");
    content.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .map(|password| password.trim().to_string())
    })
}

/// Generate a random password of the specified length containing uppercase,
/// lowercase, digits, and special characters.
fn generate_random_password(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%&*";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

const REGISTRY_WORKER_STACK_BYTES: usize = 32 * 1024 * 1024;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The generated TeaQL runtime builds large schema/mutation futures. A
    // Cargo publication previously overflowed Tokio's default worker stack.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(REGISTRY_WORKER_STACK_BYTES)
        .build()?;
    runtime.block_on(run_registry())
}

async fn run_registry() -> Result<(), Box<dyn std::error::Error>> {
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
    let runtime_config = ServiceRuntimeConfig::from_env()?;
    let postgres_config = runtime_config
        .database_url
        .parse::<tokio_postgres::Config>()?;
    let pool_manager = deadpool_postgres::Manager::new(postgres_config, tokio_postgres::NoTls);
    let runtime_pool = deadpool_postgres::Pool::builder(pool_manager).build()?;
    let runtime = service_runtime_from_pool(runtime_pool.clone()).await?;
    runtime.ensure_schema().await?;
    ensure_application_indexes(&runtime_pool).await?;
    info!("TeaQL PostgreSQL schema verified and synchronized.");

    // 2. Initialize Blob Store (Pure In-Memory Mode or Persistent S3/RustFS)
    let args: Vec<String> = std::env::args().collect();
    let memory_mode = args
        .iter()
        .any(|arg| arg == "--memory-mode" || arg == "--in-memory" || arg == "-m")
        || std::env::var("MEMORY_MODE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
        || std::env::var("STORAGE_MODE")
            .map(|v| v.to_lowercase() == "memory")
            .unwrap_or(false);

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
    runtime.set_tenant(1, "Default Tenant");

    // 3. Seed baseline initial configuration and sample demo artifacts
    seed_initial_data(&runtime).await?;
    let app_state =
        AppState::with_runtime_pool(Arc::new(runtime), runtime_pool, blobstore, memory_mode);
    seed_tenants(&app_state).await?;
    let _ = teaql_registry::services::seed_demo_artifacts(
        &app_state.runtime,
        app_state.blobstore.as_ref(),
    )
    .await;

    // 4. Assemble Axum Router
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
