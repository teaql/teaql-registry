use std::sync::Arc;
use teaql_registry_core::ServiceRuntime;
use teaql_core::{Expr, SelectQuery};
use teaql_runtime::{RequestPolicy, RuntimeError, UserContext};

use crate::blobstore::{BlobStore, BlobStoreManager, MemoryBlobStore};
use crate::engine::RepositoryRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantInfo {
    pub tenant_id: u64,
    pub tenant_name: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TeaQLRegistryTenantRequestPolicy;

// Keep alias for backwards compatibility
pub type NexusTenantRequestPolicy = TeaQLRegistryTenantRequestPolicy;

impl RequestPolicy for TeaQLRegistryTenantRequestPolicy {
    fn enforce_select(
        &self,
        ctx: &UserContext,
        query: &mut SelectQuery,
    ) -> Result<(), RuntimeError> {
        let tenant_scoped_entities = [
            "RepositoryConfiguration",
            "BlobStoreConfiguration",
            "SecurityUser",
            "SecurityRole",
            "SecurityPrivilege",
        ];

        if tenant_scoped_entities.contains(&query.entity.as_str()) {
            let tenant_id = ctx
                .get_resource::<TenantInfo>()
                .map(|t| t.tenant_id)
                .unwrap_or(1_u64);
            let tenant_filter = Expr::eq("tenant_id", tenant_id);
            query.filter = Some(match query.filter.take() {
                Some(existing) => existing.and_expr(tenant_filter),
                None => tenant_filter,
            });
        }

        Ok(())
    }
}

pub trait RegistryContextExt {
    fn set_tenant(&mut self, tenant_id: u64, tenant_name: &str);
    fn tenant_id(&self) -> u64;
    fn tenant_name(&self) -> &str;
    fn init_tenant_policy(&mut self);

    fn set_blobstore(&mut self, blobstore: Arc<dyn BlobStore>);
    fn blobstore(&self) -> Arc<dyn BlobStore>;
    fn set_blobstore_manager(&mut self, manager: BlobStoreManager);
    fn blobstore_manager(&self) -> Option<Arc<BlobStoreManager>>;

    fn set_repository_registry(&mut self, registry: RepositoryRegistry);
    fn repository_registry(&self) -> Option<Arc<RepositoryRegistry>>;

    fn init_registry_context(&mut self, blobstore: Arc<dyn BlobStore>);
}

// Keep NexusContextExt alias for backwards compatibility
pub use RegistryContextExt as NexusContextExt;

impl RegistryContextExt for ServiceRuntime {
    fn set_tenant(&mut self, tenant_id: u64, tenant_name: &str) {
        self.insert_resource(TenantInfo {
            tenant_id,
            tenant_name: tenant_name.to_string(),
        });
        self.set_request_policy(TeaQLRegistryTenantRequestPolicy);
    }

    fn tenant_id(&self) -> u64 {
        self.get_resource::<TenantInfo>()
            .map(|t| t.tenant_id)
            .unwrap_or(1_u64)
    }

    fn tenant_name(&self) -> &str {
        self.get_resource::<TenantInfo>()
            .map(|t| t.tenant_name.as_str())
            .unwrap_or("Default Platform")
    }

    fn init_tenant_policy(&mut self) {
        self.set_request_policy(TeaQLRegistryTenantRequestPolicy);
    }

    fn set_blobstore(&mut self, blobstore: Arc<dyn BlobStore>) {
        self.insert_resource(BlobStoreManager::new(blobstore));
    }

    fn blobstore(&self) -> Arc<dyn BlobStore> {
        if let Some(mgr) = self.get_resource::<BlobStoreManager>() {
            mgr.default_store()
        } else {
            // Fallback in-memory store if not initialized
            Arc::new(MemoryBlobStore::new("default"))
        }
    }

    fn set_blobstore_manager(&mut self, manager: BlobStoreManager) {
        self.insert_resource(manager);
    }

    fn blobstore_manager(&self) -> Option<Arc<BlobStoreManager>> {
        self.get_resource::<BlobStoreManager>().map(|m| Arc::new(m.clone()))
    }

    fn set_repository_registry(&mut self, registry: RepositoryRegistry) {
        self.insert_resource(registry);
    }

    fn repository_registry(&self) -> Option<Arc<RepositoryRegistry>> {
        self.get_resource::<RepositoryRegistry>().map(|r| Arc::new(r.clone()))
    }

    fn init_registry_context(&mut self, blobstore: Arc<dyn BlobStore>) {
        self.init_tenant_policy();
        self.set_blobstore(blobstore);
        self.set_repository_registry(RepositoryRegistry::new());
    }
}
