use nexus_repository_service_core::ServiceRuntime;
use teaql_core::{Expr, SelectQuery};
use teaql_runtime::{RequestPolicy, RuntimeError, UserContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantInfo {
    pub tenant_id: u64,
    pub tenant_name: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NexusTenantRequestPolicy;

impl RequestPolicy for NexusTenantRequestPolicy {
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

pub trait NexusContextExt {
    fn set_tenant(&mut self, tenant_id: u64, tenant_name: &str);
    fn tenant_id(&self) -> u64;
    fn tenant_name(&self) -> &str;
    fn init_nexus_policy(&mut self);
}

impl NexusContextExt for ServiceRuntime {
    fn set_tenant(&mut self, tenant_id: u64, tenant_name: &str) {
        self.insert_resource(TenantInfo {
            tenant_id,
            tenant_name: tenant_name.to_string(),
        });
        self.set_request_policy(NexusTenantRequestPolicy);
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

    fn init_nexus_policy(&mut self) {
        self.set_request_policy(NexusTenantRequestPolicy);
    }
}
