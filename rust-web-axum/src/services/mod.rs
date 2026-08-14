pub mod asset_service;
pub mod blob_store_service;
pub mod component_service;
pub mod repository_service;
pub mod security_service;
pub mod tenant_service;

pub use asset_service::AssetService;
pub use blob_store_service::BlobStoreService;
pub use component_service::ComponentService;
pub use repository_service::RepositoryService;
pub use security_service::SecurityService;
pub use tenant_service::TenantService;

pub trait SaveAuditedExt<T: teaql_core::Entity + teaql_runtime::LedgerEntity> {
    fn save_with<'a, C: teaql_registry_core::TeaqlRuntime + Sync + ?Sized + 'a>(
        self,
        ctx: &'a C,
    ) -> impl std::future::Future<Output = Result<teaql_runtime::GraphNode, anyhow::Error>> + Send + 'a;
}

impl<T: teaql_core::Entity + teaql_runtime::LedgerEntity + Send + 'static> SaveAuditedExt<T> for teaql_core::Audited<T> {
    fn save_with<'a, C: teaql_registry_core::TeaqlRuntime + Sync + ?Sized + 'a>(
        self,
        ctx: &'a C,
    ) -> impl std::future::Future<Output = Result<teaql_runtime::GraphNode, anyhow::Error>> + Send + 'a {
        async move {
            teaql_runtime::save_audited_ledger_entity(self, ctx.user_context())
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))
        }
    }
}
