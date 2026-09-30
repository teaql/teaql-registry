use crate::services::SaveAuditedExt;
use anyhow::{anyhow, Result};
use teaql_core::{Entity, SmartList};
use teaql_registry_core::{BlobStoreConfiguration, ServiceRuntime, Q};

use crate::context::NexusContextExt;

pub struct BlobStoreService;

impl BlobStoreService {
    pub async fn list(ctx: &ServiceRuntime) -> Result<SmartList<BlobStoreConfiguration>> {
        let rows = Q::blob_store_configurations_minimal()
            .select_self_fields()
            .limit(1000)
            .comment("what: query tenant blob store configuration")
            .purpose("why: resolve durable package content storage")
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
            .limit(1000)
            .comment("what: query tenant blob store configuration")
            .purpose("why: resolve durable package content storage")
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
            .comment("what: query tenant blob store configuration")
            .purpose("why: resolve durable package content storage")
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
            .comment("what: query tenant blob store configuration")
            .purpose("why: resolve durable package content storage")
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
            .comment("what: create tenant blob store configuration")
            .purpose("why: provision durable package content storage")
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

        let entity = entity
            .audit_as("Creating blob store configuration")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save blob store configuration: {}", e))?;

        Ok(entity)
    }
}
