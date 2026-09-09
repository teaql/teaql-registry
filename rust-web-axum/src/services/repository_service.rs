use crate::services::SaveAuditedExt;
use anyhow::{anyhow, Result};
use teaql_core::{Entity, SmartList};
use teaql_registry_core::{ContentRepository, RepositoryConfiguration, ServiceRuntime, Q};

use crate::context::NexusContextExt;

pub struct RepositoryService;

impl RepositoryService {
    pub async fn list(ctx: &ServiceRuntime) -> Result<SmartList<RepositoryConfiguration>> {
        let rows = Q::repository_configurations_minimal()
            .select_self_fields()
            .limit(1000)
            .comment("what: query tenant repository configuration")
            .purpose("why: resolve an isolated package repository")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list repositories: {}", e))?;
        Ok(rows)
    }

    pub async fn list_by_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
    ) -> Result<SmartList<RepositoryConfiguration>> {
        let rows = Q::repository_configurations_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .limit(1000)
            .comment("what: query tenant repository configuration")
            .purpose("why: resolve an isolated package repository")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list repositories: {}", e))?;
        Ok(rows)
    }

    pub async fn find_by_name(
        ctx: &ServiceRuntime,
        name: &str,
    ) -> Result<Option<RepositoryConfiguration>> {
        let rows = Q::repository_configurations_minimal()
            .select_self_fields()
            .with_name_is(name)
            .limit(1)
            .comment("what: query tenant repository configuration")
            .purpose("why: resolve an isolated package repository")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find repository: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn find_by_name_and_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        name: &str,
    ) -> Result<Option<RepositoryConfiguration>> {
        let rows = Q::repository_configurations_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .with_name_is(name)
            .limit(1)
            .comment("what: query tenant repository configuration")
            .purpose("why: resolve an isolated package repository")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find repository: {}", e))?;
        Ok(rows.into_iter().next())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        ctx: &ServiceRuntime,
        name: &str,
        recipe_name: &str,
        repo_type: &str,
        format: &str,
        write_policy: &str,
        blob_store_id: u64,
        online: bool,
        remote_url: &str,
    ) -> Result<RepositoryConfiguration> {
        let tenant_id = ctx.tenant_id();
        Self::create_with_tenant(
            ctx,
            tenant_id,
            name,
            recipe_name,
            repo_type,
            format,
            write_policy,
            blob_store_id,
            online,
            remote_url,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        name: &str,
        recipe_name: &str,
        repo_type: &str,
        format: &str,
        write_policy: &str,
        blob_store_id: u64,
        online: bool,
        remote_url: &str,
    ) -> Result<RepositoryConfiguration> {
        let mut entity = Q::repository_configurations()
            .comment("what: create tenant repository configuration")
            .purpose("why: provision an isolated package repository")
            .new_entity(ctx);

        entity.update_tenant_id(tenant_id);
        entity.update_name(name);
        entity.update_recipe_name(recipe_name);
        entity.update_blob_store_id(blob_store_id);
        entity.update_online(online);
        entity.update_remote_url(remote_url);

        match repo_type.to_uppercase().as_str() {
            "HOSTED" => {
                entity.update_repository_type_to_hosted();
            }
            "PROXY" => {
                entity.update_repository_type_to_proxy();
            }
            "GROUP" => {
                entity.update_repository_type_to_group();
            }
            _ => {
                entity.update_repository_type_to_hosted();
            }
        }

        match format.to_uppercase().as_str() {
            "MAVEN2" | "MAVEN" => {
                entity.update_repository_format_to_maven2();
            }
            "RAW" => {
                entity.update_repository_format_to_raw();
            }
            "DOCKER" => {
                entity.update_repository_format_to_docker();
            }
            "NPM" => {
                entity.update_repository_format_to_npm();
            }
            "PYPI" => {
                entity.update_repository_format_to_pypi();
            }
            "GOMOD" => {
                entity.update_repository_format_to_gomod();
            }
            "CARGO" => {
                entity.update_repository_format_to_cargo();
            }
            "NUGET" => {
                entity.update_repository_format_to_nuget();
            }
            "SWIFT" | "SPM" => {
                entity.update_repository_format_to_swift();
            }
            _ => {
                entity.update_repository_format_to_raw();
            }
        }

        match write_policy.to_uppercase().as_str() {
            "ALLOW_WRITE" => {
                entity.update_write_policy_to_allow_write();
            }
            "ALLOW_ONCE" => {
                entity.update_write_policy_to_allow_once();
            }
            "READ_ONLY" => {
                entity.update_write_policy_to_read_only();
            }
            _ => {
                entity.update_write_policy_to_allow_write();
            }
        }

        let entity = entity
            .audit_as("Creating repository configuration")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save repository configuration: {}", e))?;

        // Also ensure ContentRepository exists
        Self::ensure_content_repository(ctx, entity.id(), format).await?;

        Ok(entity)
    }

    pub async fn ensure_content_repository(
        ctx: &ServiceRuntime,
        repo_id: u64,
        format_name: &str,
    ) -> Result<ContentRepository> {
        let records = Q::content_repositories_minimal()
            .select_self_fields()
            .with_repository_id_is(repo_id)
            .limit(1)
            .comment("what: locate content repository mapping")
            .purpose("why: reuse the tenant-scoped repository identity")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to query content repository: {}", e))?;

        if let Some(entity) = records.into_iter().next() {
            return Ok(entity);
        }

        let mut cr_entity = Q::content_repositories()
            .comment("what: create tenant repository configuration")
            .purpose("why: provision an isolated package repository")
            .new_entity(ctx);

        cr_entity.update_tenant_id(ctx.tenant_id());
        cr_entity.update_repository_id(repo_id);
        cr_entity.update_format_name(format_name);

        let cr_entity = cr_entity
            .audit_as("Creating content repository mapping")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save content repository: {}", e))?;

        Ok(cr_entity)
    }

    pub async fn get_content_repository(
        ctx: &ServiceRuntime,
        repo_id: u64,
    ) -> Result<Option<ContentRepository>> {
        let records = Q::content_repositories_minimal()
            .select_self_fields()
            .with_repository_id_is(repo_id)
            .limit(1)
            .comment("what: load content repository mapping")
            .purpose("why: resolve registry package metadata")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to load content repository: {}", e))?;

        Ok(records.into_iter().next())
    }
}
