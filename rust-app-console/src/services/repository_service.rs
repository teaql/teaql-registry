use anyhow::{anyhow, Result};
use nexus_repository_service_core::{
    ContentRepository, Q, RepositoryConfiguration, ServiceRuntime,
};
use teaql_core::{Entity, SmartList};
use crate::services::SaveAuditedExt;

use crate::context::NexusContextExt;

pub struct RepositoryService;

impl RepositoryService {
    pub async fn list(ctx: &ServiceRuntime) -> Result<SmartList<RepositoryConfiguration>> {
        let rows = Q::repository_configurations_minimal()
            .select_self_fields()
            .limit(1000)
            .comment("what: Load all repository configurations for tenant")
            .purpose("why: REST repositories list API")
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
            .comment("what: Load all repository configurations for tenant")
            .purpose("why: REST repositories list API")
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
            .comment("what: Load repository configuration by name")
            .purpose("why: Route request to specific repository")
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
            .comment("what: Load repository configuration by name and tenant")
            .purpose("why: Route request to specific repository")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find repository: {}", e))?;
        Ok(rows.into_iter().next())
    }

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
            .purpose("why: Create new RepositoryConfiguration")
            .new_entity(ctx);

        entity.update_tenant_id(tenant_id);
        entity.update_name(name);
        entity.update_recipe_name(recipe_name);
        entity.update_blob_store_id(blob_store_id);
        entity.update_online(online);
        entity.update_remote_url(remote_url);

        match repo_type.to_uppercase().as_str() {
            "HOSTED" => { entity.update_repository_type_to_hosted(); }
            "PROXY" => { entity.update_repository_type_to_proxy(); }
            "GROUP" => { entity.update_repository_type_to_group(); }
            _ => { entity.update_repository_type_to_hosted(); }
        }

        match format.to_uppercase().as_str() {
            "MAVEN2" | "MAVEN" => { entity.update_repository_format_to_maven2(); }
            "RAW" => { entity.update_repository_format_to_raw(); }
            "DOCKER" => { entity.update_repository_format_to_docker(); }
            "NPM" => { entity.update_repository_format_to_npm(); }
            "PYPI" => { entity.update_repository_format_to_pypi(); }
            "GOMOD" => { entity.update_repository_format_to_gomod(); }
            "CARGO" => { entity.update_repository_format_to_cargo(); }
            "NUGET" => { entity.update_repository_format_to_nuget(); }
            _ => { entity.update_repository_format_to_raw(); }
        }

        match write_policy.to_uppercase().as_str() {
            "ALLOW_WRITE" => { entity.update_write_policy_to_allow_write(); }
            "ALLOW_ONCE" => { entity.update_write_policy_to_allow_once(); }
            "READ_ONLY" => { entity.update_write_policy_to_read_only(); }
            _ => { entity.update_write_policy_to_allow_write(); }
        }

        entity
            .clone()
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
            .select_repository_id()
            .select_format_name()
            .with_repository_id_is(repo_id)
            .limit(1)
            .comment("what: Check existing content repository")
            .purpose("why: Prevent duplicate content repository records")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to query content repository: {}", e))?;

        if let Some(rec) = records.into_iter().next() {
            let cr = ContentRepository::from_record(rec).map_err(|e| anyhow!("Failed to parse content repository: {}", e))?;
            return Ok(cr);
        }

        let mut cr_entity = Q::content_repositories()
            .purpose("why: Create content repository record")
            .new_entity(ctx);

        cr_entity.update_repository_id(repo_id);
        cr_entity.update_format_name(format_name);

        cr_entity
            .clone()
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
            .select_repository_id()
            .select_format_name()
            .with_repository_id_is(repo_id)
            .limit(1)
            .comment("what: Load content repository by repo id")
            .purpose("why: Find content repository container")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to load content repository: {}", e))?;

        if let Some(rec) = records.into_iter().next() {
            let cr = ContentRepository::from_record(rec).map_err(|e| anyhow!("Failed to parse content repository: {}", e))?;
            Ok(Some(cr))
        } else {
            Ok(None)
        }
    }
}
