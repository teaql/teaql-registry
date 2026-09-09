use crate::services::SaveAuditedExt;
use anyhow::{anyhow, Result};
use teaql_core::{Entity, SmartList};
use teaql_registry_core::{ServiceRuntime, Tenant, Q};

use crate::services::{BlobStoreService, RepositoryService, SecurityService};

pub struct TenantService;

impl TenantService {
    pub async fn list_tenants(ctx: &ServiceRuntime) -> Result<SmartList<Tenant>> {
        let rows = Q::tenants_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: query registry tenant metadata")
            .purpose("why: resolve or administer an isolated tenant")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list tenants: {}", e))?;
        Ok(rows)
    }

    pub async fn list_tenants_by_platform(
        ctx: &ServiceRuntime,
        platform_id: u64,
    ) -> Result<SmartList<Tenant>> {
        let rows = Q::tenants_minimal()
            .select_self_fields()
            .filter_by_platform(platform_id)
            .limit(100)
            .comment("what: query registry tenant metadata")
            .purpose("why: resolve or administer an isolated tenant")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list tenants: {}", e))?;
        Ok(rows)
    }

    pub async fn get_tenant(ctx: &ServiceRuntime, tenant_id: u64) -> Result<Option<Tenant>> {
        let rows = Q::tenants_minimal()
            .select_self_fields()
            .with_id_is(Tenant::with_id(tenant_id))
            .limit(1)
            .comment("what: query registry tenant metadata")
            .purpose("why: resolve or administer an isolated tenant")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to get tenant: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn find_tenant_by_code(ctx: &ServiceRuntime, code: &str) -> Result<Option<Tenant>> {
        let rows = Q::tenants_minimal()
            .select_self_fields()
            .with_code_is(code)
            .limit(1)
            .comment("what: locate a tenant by its stable code")
            .purpose("why: bind an authenticated request to its tenant")
            .execute_for_list(ctx)
            .await
            .map_err(|error| anyhow!("Failed to find tenant: {error}"))?;
        Ok(rows.into_iter().next())
    }

    pub async fn create_tenant(ctx: &ServiceRuntime, name: &str, code: &str) -> Result<Tenant> {
        Self::create_tenant_with_platform(ctx, 1_u64, name, code, "").await
    }

    pub async fn create_tenant_with_platform(
        ctx: &ServiceRuntime,
        platform_id: u64,
        name: &str,
        code: &str,
        description: &str,
    ) -> Result<Tenant> {
        if let Some(existing) = Self::find_tenant_by_code(ctx, code).await? {
            return Ok(existing);
        }
        let mut entity = Q::tenants()
            .comment("what: create registry tenant metadata")
            .purpose("why: provision an isolated registry tenant")
            .new_entity(ctx);

        entity.update_platform_id(platform_id);
        entity.update_name(name);
        entity.update_code(code);
        entity.update_description(description);
        entity.update_enabled(true);

        let entity = entity
            .audit_as("Creating tenant")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save tenant: {}", e))?;

        Ok(entity)
    }

    pub async fn provision_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        blob_root_dir: &str,
        users: &[(&str, &str, &str, &str, &str)],
    ) -> Result<()> {
        // 1. Create or recover the tenant default blob store.
        let tenant_blob_path = format!(
            "{}/tenant_{}/default",
            blob_root_dir.trim_end_matches('/'),
            tenant_id
        );
        let bs = match BlobStoreService::find_by_name_and_tenant(ctx, tenant_id, "default").await? {
            Some(existing) => existing,
            None => {
                BlobStoreService::create_with_tenant(
                    ctx,
                    tenant_id,
                    "default",
                    &tenant_blob_path,
                    true,
                )
                .await?
            }
        };

        // 2. Create tenant default repositories
        let format_repos = [
            (
                "maven-releases",
                "maven2-hosted",
                "HOSTED",
                "MAVEN2",
                "ALLOW_WRITE",
                "",
            ),
            (
                "maven-snapshots",
                "maven2-hosted",
                "HOSTED",
                "MAVEN2",
                "ALLOW_WRITE",
                "",
            ),
            (
                "maven-central",
                "maven2-proxy",
                "PROXY",
                "MAVEN2",
                "READ_ONLY",
                "https://repo1.maven.org/maven2",
            ),
            (
                "maven-public",
                "maven2-group",
                "GROUP",
                "MAVEN2",
                "READ_ONLY",
                "",
            ),
            (
                "raw-hosted",
                "raw-hosted",
                "HOSTED",
                "RAW",
                "ALLOW_WRITE",
                "",
            ),
            (
                "docker-hosted",
                "docker-hosted",
                "HOSTED",
                "DOCKER",
                "ALLOW_WRITE",
                "",
            ),
            (
                "npm-hosted",
                "npm-hosted",
                "HOSTED",
                "NPM",
                "ALLOW_WRITE",
                "",
            ),
            (
                "pypi-hosted",
                "pypi-hosted",
                "HOSTED",
                "PYPI",
                "ALLOW_WRITE",
                "",
            ),
            (
                "gomod-hosted",
                "gomod-hosted",
                "HOSTED",
                "GOMOD",
                "ALLOW_WRITE",
                "",
            ),
            (
                "cargo-hosted",
                "cargo-hosted",
                "HOSTED",
                "CARGO",
                "ALLOW_WRITE",
                "",
            ),
            (
                "nuget-hosted",
                "nuget-hosted",
                "HOSTED",
                "NUGET",
                "ALLOW_WRITE",
                "",
            ),
            (
                "swift-hosted",
                "swift-hosted",
                "HOSTED",
                "SWIFT",
                "ALLOW_ONCE",
                "",
            ),
        ];

        for (name, recipe, rtype, fmt, wpolicy, rurl) in format_repos {
            if RepositoryService::find_by_name_and_tenant(ctx, tenant_id, name)
                .await?
                .is_none()
            {
                RepositoryService::create_with_tenant(
                    ctx,
                    tenant_id,
                    name,
                    recipe,
                    rtype,
                    fmt,
                    wpolicy,
                    bs.id(),
                    true,
                    rurl,
                )
                .await?;
            }
        }

        // 3. Create or recover roles and their durable privilege assignments.
        let admin_role =
            match SecurityService::find_role_by_tenant_and_key(ctx, tenant_id, "nx-admin").await? {
                Some(existing) => existing,
                None => {
                    SecurityService::create_role_with_tenant(
                        ctx,
                        tenant_id,
                        "nx-admin",
                        "Tenant Administrator",
                        "Administrator role for this tenant",
                        true,
                    )
                    .await?
                }
            };
        let developer_role = match SecurityService::find_role_by_tenant_and_key(
            ctx,
            tenant_id,
            "registry-developer",
        )
        .await?
        {
            Some(existing) => existing,
            None => {
                SecurityService::create_role_with_tenant(
                    ctx,
                    tenant_id,
                    "registry-developer",
                    "Registry Developer",
                    "Read and publish packages in this tenant",
                    true,
                )
                .await?
            }
        };
        let admin_privilege =
            match SecurityService::find_privilege_by_tenant_and_key(ctx, tenant_id, "nx-all")
                .await?
            {
                Some(existing) => {
                    SecurityService::reconcile_privilege_pattern(
                        ctx,
                        existing,
                        "registry",
                        "registry:admin",
                    )
                    .await?
                }
                None => {
                    SecurityService::create_privilege_with_tenant(
                        ctx,
                        tenant_id,
                        "nx-all",
                        "All Privileges",
                        "Administer this tenant registry",
                        "registry",
                        "registry:admin",
                        true,
                    )
                    .await?
                }
            };
        let read_privilege = match SecurityService::find_privilege_by_tenant_and_key(
            ctx,
            tenant_id,
            "registry-read",
        )
        .await?
        {
            Some(existing) => existing,
            None => {
                SecurityService::create_privilege_with_tenant(
                    ctx,
                    tenant_id,
                    "registry-read",
                    "Read Packages",
                    "Read packages and repository metadata",
                    "registry",
                    "repository:read",
                    true,
                )
                .await?
            }
        };
        let write_privilege = match SecurityService::find_privilege_by_tenant_and_key(
            ctx,
            tenant_id,
            "registry-write",
        )
        .await?
        {
            Some(existing) => existing,
            None => {
                SecurityService::create_privilege_with_tenant(
                    ctx,
                    tenant_id,
                    "registry-write",
                    "Publish Packages",
                    "Publish packages to writable repositories",
                    "registry",
                    "repository:write",
                    true,
                )
                .await?
            }
        };
        SecurityService::assign_privilege(ctx, tenant_id, admin_role.id(), admin_privilege.id())
            .await?;
        SecurityService::assign_privilege(ctx, tenant_id, admin_role.id(), read_privilege.id())
            .await?;
        SecurityService::assign_privilege(ctx, tenant_id, admin_role.id(), write_privilege.id())
            .await?;
        SecurityService::assign_privilege(ctx, tenant_id, developer_role.id(), read_privilege.id())
            .await?;
        SecurityService::assign_privilege(
            ctx,
            tenant_id,
            developer_role.id(),
            write_privilege.id(),
        )
        .await?;

        // 4. Create or recover users, then attach explicit roles.
        for (username, first_name, last_name, email, password_hash) in users {
            let user =
                match SecurityService::find_user_by_tenant_and_username(ctx, tenant_id, username)
                    .await?
                {
                    Some(existing) => existing,
                    None => {
                        SecurityService::create_user_with_tenant(
                            ctx,
                            tenant_id,
                            username,
                            first_name,
                            last_name,
                            email,
                            password_hash,
                        )
                        .await?
                    }
                };
            let role = if *username == "admin" {
                &admin_role
            } else {
                &developer_role
            };
            SecurityService::assign_role(ctx, tenant_id, user.id(), role.id()).await?;
        }

        Ok(())
    }
}
