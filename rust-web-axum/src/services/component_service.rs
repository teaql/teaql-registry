use crate::context::RegistryContextExt;
use crate::services::{AssetService, SaveAuditedExt};
use anyhow::{anyhow, Result};
use teaql_core::{Entity, SmartList};
use teaql_registry_core::{Component, ServiceRuntime, E, Q};

/// Stable application read model for a deliberately projected Component.
/// Keeping expression evaluation at this boundary prevents downstream format
/// handlers from silently reading TeaQL fields that a query stopped loading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedComponent {
    id: u64,
    namespace: String,
    name: String,
    version_name: String,
    normalized_version: String,
    kind: String,
}

impl LoadedComponent {
    fn from_selected(entity: Component) -> Self {
        Self {
            id: E::component(&entity).get_id().unwrap(),
            namespace: E::component(&entity).get_namespace().unwrap(),
            name: E::component(&entity).get_name().unwrap(),
            version_name: E::component(&entity).get_version_name().unwrap(),
            normalized_version: E::component(&entity).get_normalized_version().unwrap(),
            kind: E::component(&entity).get_kind().unwrap(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version_name(&self) -> &str {
        &self.version_name
    }

    pub fn normalized_version(&self) -> &str {
        &self.normalized_version
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }
}

pub struct ComponentService;

impl ComponentService {
    pub async fn get_by_id(ctx: &ServiceRuntime, component_id: u64) -> Result<Option<Component>> {
        let rows = Q::components()
            .select_self_fields()
            .with_id_is(component_id)
            .limit(1)
            .comment("what: query registry component metadata")
            .purpose("why: resolve package components and their assets")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to get component by id: {}", e))?;

        if let Some(comp) = rows.into_iter().next() {
            if comp.name().is_empty()
                || comp.name().starts_with("[DELETED")
                || comp.kind() == "deleted"
            {
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
    ) -> Result<SmartList<LoadedComponent>> {
        let rows = Q::components_minimal()
            .select_namespace()
            .select_name()
            .select_version_name()
            .select_normalized_version()
            .select_kind()
            .filter_by_content_repository(content_repo_id)
            .order_by_id_asc()
            .comment("what: query registry component metadata")
            .purpose("why: resolve package components and their assets")
            .execute_for_page(ctx, offset, limit)
            .await
            .map_err(|e| anyhow!("Failed to list components: {}", e))?;

        let filtered: Vec<LoadedComponent> = rows
            .into_iter()
            .map(LoadedComponent::from_selected)
            .filter(|c| {
                !c.name().is_empty() && !c.name().starts_with("[DELETED") && c.kind() != "deleted"
            })
            .collect();
        Ok(SmartList::new(filtered))
    }

    pub async fn list_by_repository(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
    ) -> Result<Vec<LoadedComponent>> {
        let smart_list = Self::list_by_content_repository(ctx, content_repo_id, 1000, 0).await?;
        Ok(smart_list.into_iter().collect())
    }

    /// Load one deterministic page for an exact package name. This keeps
    /// format-specific index builders on the same projected read boundary.
    pub async fn list_by_name(
        context: &ServiceRuntime,
        content_repo_id: u64,
        name: &str,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<LoadedComponent>> {
        let rows = Q::components_minimal()
            .select_namespace()
            .select_name()
            .select_version_name()
            .select_normalized_version()
            .select_kind()
            .filter_by_content_repository(content_repo_id)
            .with_name_is(name)
            .order_by_id_asc()
            .comment("what: query one package name with an explicit projection")
            .purpose("why: build a complete package index without full entity reads")
            .execute_for_page(context, offset, limit)
            .await
            .map_err(|error| anyhow!("Failed to list named components: {error}"))?;

        Ok(rows
            .into_iter()
            .map(LoadedComponent::from_selected)
            .filter(|component| {
                !component.name().is_empty()
                    && !component.name().starts_with("[DELETED")
                    && component.kind() != "deleted"
            })
            .collect())
    }

    /// In pure in-memory mode, evict older versions of the same artifact so that only the latest single version is kept.
    pub async fn evict_older_versions(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        namespace: &str,
        name: &str,
        current_version: &str,
    ) -> Result<usize> {
        let all_comps = Self::list_by_repository(ctx, content_repo_id).await?;
        let blobstore = ctx.blobstore();
        let mut evicted_count = 0;

        for old_comp in all_comps {
            if old_comp.namespace() == namespace
                && old_comp.name() == name
                && old_comp.version_name() != current_version
            {
                let assets = AssetService::list_by_component_with_blobs(ctx, old_comp.id()).await?;
                for asset in assets {
                    if let Some(blob) = asset.blob() {
                        let _ = blobstore.delete_blob(blob.blob_ref()).await;
                        let _ = AssetService::delete_asset_blob(ctx, blob.id()).await;
                    }
                    let _ = AssetService::delete(ctx, asset.id()).await;
                }

                let _ = Self::delete(ctx, old_comp.id()).await;
                evicted_count += 1;
            }
        }

        Ok(evicted_count)
    }

    pub async fn find_or_create(
        ctx: &ServiceRuntime,
        content_repo_id: u64,
        namespace: &str,
        name: &str,
        version_name: &str,
        kind: &str,
    ) -> Result<Component> {
        if ctx.is_memory_mode() {
            let _ = Self::evict_older_versions(ctx, content_repo_id, namespace, name, version_name)
                .await;
        }

        let rows = Q::components()
            .select_self_fields()
            .filter_by_content_repository(content_repo_id)
            .with_namespace_is(namespace)
            .with_name_is(name)
            .with_version_name_is(version_name)
            .limit(1)
            .comment("what: query registry component metadata")
            .purpose("why: resolve package components and their assets")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to check existing component: {}", e))?;

        if let Some(comp) = rows.into_iter().find(|c| {
            !c.name().is_empty() && !c.name().starts_with("[DELETED") && c.kind() != "deleted"
        }) {
            return Ok(comp);
        }

        Self::create(
            ctx,
            content_repo_id,
            namespace,
            name,
            version_name,
            version_name,
            kind,
        )
        .await
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
            .comment("what: create registry component metadata")
            .purpose("why: persist a package component in its repository")
            .new_entity(ctx);
        comp.update_content_repository_id(content_repo_id);
        comp.update_namespace(namespace);
        comp.update_name(name);
        comp.update_version_name(version_name);
        comp.update_normalized_version(normalized_version);
        comp.update_kind(kind);

        let comp = comp
            .audit_as("Creating component record")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save component: {}", e))?;

        Ok(comp)
    }

    pub async fn delete(ctx: &ServiceRuntime, component_id: u64) -> Result<()> {
        let rows = Q::components()
            .select_self_fields()
            .with_id_is(component_id)
            .limit(1)
            .comment("what: query registry component metadata")
            .purpose("why: resolve package components and their assets")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find component for delete: {}", e))?;

        if let Some(mut comp) = rows.into_iter().next() {
            comp.update_kind("deleted");
            comp.update_name(format!("[DELETED_{}]", comp.id()));
            let _ = comp.audit_as("Deleting component").save_with(ctx).await?;
        }
        Ok(())
    }
}
