use anyhow::{anyhow, Result};
use nexus_repository_service_core::{
    BlobStoreConfiguration, Q, ServiceRuntime,
};
use teaql_core::{Entity, SmartList};
use crate::services::SaveAuditedExt;

use crate::context::NexusContextExt;

pub struct BlobStoreService;

impl BlobStoreService {
    pub async fn list(ctx: &ServiceRuntime) -> Result<SmartList<BlobStoreConfiguration>> {
        let rows = Q::blob_store_configurations_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: Load all blob store configurations for tenant")
            .purpose("why: REST blobstores list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list blob stores: {}", e))?;
        Ok(rows)
    }

    pub async fn list_by_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
    ) -> Result<SmartList<BlobStoreConfiguration>> {
        let rows = Q::blob_store_configurations_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .limit(100)
            .comment("what: Load all blob store configurations for tenant")
            .purpose("why: REST blobstores list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list blob stores: {}", e))?;
        Ok(rows)
    }

    pub async fn find_by_name(
        ctx: &ServiceRuntime,
        name: &str,
    ) -> Result<Option<BlobStoreConfiguration>> {
        let rows = Q::blob_store_configurations_minimal()
            .select_self_fields()
            .with_name_is(name)
            .limit(1)
            .comment("what: Load blob store configuration by name")
            .purpose("why: Find blob store for repository operations")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find blob store: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn find_by_name_and_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        name: &str,
    ) -> Result<Option<BlobStoreConfiguration>> {
        let rows = Q::blob_store_configurations_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .with_name_is(name)
            .limit(1)
            .comment("what: Load blob store configuration by name and tenant")
            .purpose("why: Find blob store for repository operations")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find blob store: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn create(
        ctx: &ServiceRuntime,
        name: &str,
        path: &str,
        is_file: bool,
    ) -> Result<BlobStoreConfiguration> {
        let tenant_id = ctx.tenant_id();
        Self::create_with_tenant(ctx, tenant_id, name, path, is_file).await
    }

    pub async fn create_with_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        name: &str,
        path: &str,
        is_file: bool,
    ) -> Result<BlobStoreConfiguration> {
        let mut entity = Q::blob_store_configurations()
            .purpose("why: Create new BlobStoreConfiguration")
            .new_entity(ctx);

        entity.update_tenant_id(tenant_id);
        entity.update_name(name);
        entity.update_path(path);
        entity.update_total_size(0);
        entity.update_blob_count(0);
        if is_file {
            entity.update_blob_store_type_to_file();
        } else {
            entity.update_blob_store_type_to_s3();
        }

        entity
            .clone()
            .audit_as("Creating blob store configuration")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save blob store configuration: {}", e))?;

        Ok(entity)
    }
}
