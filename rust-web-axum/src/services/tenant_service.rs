use anyhow::{anyhow, Result};
use teaql_registry_core::{
    Q, ServiceRuntime, Tenant,
};
use teaql_core::{Entity, SmartList};
use crate::services::SaveAuditedExt;

use crate::security::hash_password;
use crate::services::{BlobStoreService, RepositoryService, SecurityService};

pub struct TenantService;

impl TenantService {
    pub async fn list_tenants(ctx: &ServiceRuntime) -> Result<SmartList<Tenant>> {
        let rows = Q::tenants_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: Load all tenants under platform")
            .purpose("why: Multi-tenant management")
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
            .comment("what: Load all tenants for platform")
            .purpose("why: Multi-tenant management by platform")
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
            .comment("what: Get tenant by id")
            .purpose("why: Multi-tenant resolution")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to get tenant: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn create_tenant(
        ctx: &ServiceRuntime,
        name: &str,
        code: &str,
    ) -> Result<Tenant> {
        Self::create_tenant_with_platform(ctx, 1_u64, name, code, "").await
    }

    pub async fn create_tenant_with_platform(
        ctx: &ServiceRuntime,
        platform_id: u64,
        name: &str,
        code: &str,
        description: &str,
    ) -> Result<Tenant> {
        let mut entity = Q::tenants()
            .purpose("why: Create new tenant instance under platform")
            .new_entity(ctx);

        entity.update_platform_id(platform_id);
        entity.update_name(name);
        entity.update_code(code);
        entity.update_description(description);
        entity.update_enabled(true);

        entity
            .clone()
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
    ) -> Result<()> {
        // 1. Create tenant default blob store
        let tenant_blob_path = format!("{}/tenant_{}/default", blob_root_dir.trim_end_matches('/'), tenant_id);
        let bs = BlobStoreService::create_with_tenant(
            ctx,
            tenant_id,
            "default",
            &tenant_blob_path,
            true,
        )
        .await?;

        // 2. Create tenant default repositories
        let format_repos = [
            ("maven-releases", "maven2-hosted", "HOSTED", "MAVEN2", "ALLOW_WRITE", ""),
            ("maven-snapshots", "maven2-hosted", "HOSTED", "MAVEN2", "ALLOW_WRITE", ""),
            ("maven-central", "maven2-proxy", "PROXY", "MAVEN2", "READ_ONLY", "https://repo1.maven.org/maven2"),
            ("maven-public", "maven2-group", "GROUP", "MAVEN2", "READ_ONLY", ""),
            ("raw-hosted", "raw-hosted", "HOSTED", "RAW", "ALLOW_WRITE", ""),
            ("docker-hosted", "docker-hosted", "HOSTED", "DOCKER", "ALLOW_WRITE", ""),
            ("npm-hosted", "npm-hosted", "HOSTED", "NPM", "ALLOW_WRITE", ""),
            ("pypi-hosted", "pypi-hosted", "HOSTED", "PYPI", "ALLOW_WRITE", ""),
            ("gomod-hosted", "gomod-hosted", "HOSTED", "GOMOD", "ALLOW_WRITE", ""),
            ("cargo-hosted", "cargo-hosted", "HOSTED", "CARGO", "ALLOW_WRITE", ""),
            ("nuget-hosted", "nuget-hosted", "HOSTED", "NUGET", "ALLOW_WRITE", ""),
        ];

        for (name, recipe, rtype, fmt, wpolicy, rurl) in format_repos {
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

        // 3. Create tenant admin user
        let pass_hash = hash_password("admin123");
        SecurityService::create_user_with_tenant(
            ctx,
            tenant_id,
            "admin",
            "Administrator",
            "User",
            &format!("admin@tenant{}.local", tenant_id),
            &pass_hash,
        )
        .await?;

        // 4. Create tenant admin role and privileges
        SecurityService::create_role_with_tenant(
            ctx,
            tenant_id,
            "nx-admin",
            "Tenant Administrator",
            "Administrator role for this tenant",
            true,
        )
        .await?;

        SecurityService::create_privilege_with_tenant(
            ctx,
            tenant_id,
            "nx-all",
            "All Privileges",
            "Full privileges on tenant repositories",
            "wildcard",
            "*.*.*",
            true,
        )
        .await?;

        Ok(())
    }
}
