use anyhow::{anyhow, Result};
use teaql_registry_core::{
    Component, Q, ServiceRuntime,
};
use teaql_core::{Entity, SmartList};
use crate::services::SaveAuditedExt;

pub struct ComponentService;

impl ComponentService {
    pub async fn list_by_content_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        limit: u64,
        offset: u64,
    ) -> Result<SmartList<Component>> {
        let records = Q::components_minimal()
            .select_self_fields()
            .filter_by_content_repository(content_repo_id)
            .offset(offset, limit)
            .comment("what: Load components for content repository")
            .purpose("why: REST components query API")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list components: {}", e))?;

        let mut items = Vec::new();
        for rec in records {
            let comp = Component::from_record(rec).map_err(|e| anyhow!("Failed to parse component: {}", e))?;
            items.push(comp);
        }
        Ok(SmartList::new(items))
    }

    pub async fn find_or_create(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        namespace: &str,
        name: &str,
        version_name: &str,
        kind: &str,
    ) -> Result<Component> {
        let records = Q::components_minimal()
            .select_self_fields()
            .filter_by_content_repository(content_repo_id)
            .with_namespace_is(namespace)
            .with_name_is(name)
            .with_version_name_is(version_name)
            .limit(1)
            .comment("what: Check existing component")
            .purpose("why: Avoid duplicate component creations")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to check existing component: {}", e))?;

        if let Some(rec) = records.into_iter().next() {
            let comp = Component::from_record(rec).map_err(|e| anyhow!("Failed to parse component: {}", e))?;
            return Ok(comp);
        }

        let mut comp = Q::components()
            .purpose("why: Create new component entity")
            .new_entity(ctx);

        comp.update_content_repository_id(content_repo_id);
        comp.update_namespace(namespace);
        comp.update_name(name);
        comp.update_version_name(version_name);
        comp.update_normalized_version(version_name);
        comp.update_kind(kind);

        comp.clone()
            .audit_as("Creating component coordinate")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save component: {}", e))?;

        Ok(comp)
    }

    pub async fn get_by_id(
        ctx: &ServiceRuntime,
        id: u64,
    ) -> Result<Option<Component>> {
        let records = Q::components_minimal()
            .select_self_fields()
            .with_id_is(id)
            .limit(1)
            .comment("what: Load component by ID")
            .purpose("why: REST component details API")
            .execute_for_records(ctx)
            .await
            .map_err(|e| anyhow!("Failed to load component: {}", e))?;

        if let Some(rec) = records.into_iter().next() {
            let comp = Component::from_record(rec).map_err(|e| anyhow!("Failed to parse component: {}", e))?;
            Ok(Some(comp))
        } else {
            Ok(None)
        }
    }
}
