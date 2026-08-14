use anyhow::{anyhow, Result};
use teaql_registry_core::{
    Component, Q, ServiceRuntime,
};
use teaql_core::{Entity, SmartList};
use crate::services::SaveAuditedExt;

pub struct ComponentService;

impl ComponentService {
    pub async fn get_by_id(ctx: &ServiceRuntime, component_id: u64) -> Result<Option<Component>> {
        let rows = Q::components()
            .with_id_is(component_id)
            .limit(1)
            .comment("what: Get component by id")
            .purpose("why: Get component by id")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to get component by id: {}", e))?;

        if let Some(comp) = rows.into_iter().next() {
            if comp.name().is_empty() {
                Ok(None)
            } else {
                Ok(Some(comp))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn list_by_content_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        limit: u64,
        offset: u64,
    ) -> Result<SmartList<Component>> {
        let rows = Q::components()
            .filter_by_content_repository(content_repo_id)
            .offset(offset, limit)
            .comment("what: Load components for content repository")
            .purpose("why: REST components query API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list components: {}", e))?;

        let filtered: Vec<Component> = rows.into_iter().filter(|c| !c.name().is_empty()).collect();
        Ok(SmartList::new(filtered))
    }

    pub async fn list_by_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
    ) -> Result<Vec<Component>> {
        let smart_list = Self::list_by_content_repository(ctx, content_repo_id, 1000, 0).await?;
        Ok(smart_list.into_iter().collect())
    }

    pub async fn find_or_create(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        namespace: &str,
        name: &str,
        version_name: &str,
        kind: &str,
    ) -> Result<Component> {
        let rows = Q::components()
            .filter_by_content_repository(content_repo_id)
            .with_namespace_is(namespace)
            .with_name_is(name)
            .with_version_name_is(version_name)
            .limit(1)
            .comment("what: Check existing component")
            .purpose("why: Avoid duplicate component creations")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to check existing component: {}", e))?;

        if let Some(comp) = rows.into_iter().find(|c| !c.name().is_empty()) {
            return Ok(comp);
        }

        Self::create(ctx, content_repo_id, namespace, name, version_name, version_name, kind).await
    }

    pub async fn create(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        namespace: &str,
        name: &str,
        version_name: &str,
        normalized_version: &str,
        kind: &str,
    ) -> Result<Component> {
        let mut comp = Q::components()
            .purpose("why: Create new component record")
            .new_entity(ctx);

        comp.update_content_repository_id(content_repo_id);
        comp.update_namespace(namespace);
        comp.update_name(name);
        comp.update_version_name(version_name);
        comp.update_normalized_version(normalized_version);
        comp.update_kind(kind);

        comp.clone()
            .audit_as("Creating component record")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save component: {}", e))?;

        Ok(comp)
    }

    pub async fn delete(ctx: &ServiceRuntime, component_id: u64) -> Result<()> {
        let rows = Q::components()
            .with_id_is(component_id)
            .limit(1)
            .comment("what: Find component to delete")
            .purpose("why: Delete component")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find component for delete: {}", e))?;

        if let Some(mut comp) = rows.into_iter().next() {
            comp.update_name("");
            let _ = comp.audit_as("Deleting component").save_with(ctx).await?;
        }
        Ok(())
    }
}
