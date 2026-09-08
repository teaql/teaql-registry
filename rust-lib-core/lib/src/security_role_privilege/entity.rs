// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/security_role_privilege
use std::collections::BTreeMap;

use teaql_macros::{teaql_entity, TeaqlEntity};

/// [TEAQL AI WARNING]
/// TeaQL was explicitly designed to PREVENT AI hallucinations and random guessing.
/// DO NOT GUESS METHOD NAMES!
/// The methods listed below are the ONLY valid ways to interact with this entity.
/// If you encounter compilation errors (e.g., method not found), DO NOT guess another method name.
/// Read the method signatures in this file before proceeding.
#[teaql_entity]
#[derive(Clone, Debug, PartialEq, TeaqlEntity)]
#[teaql(
    entity = "SecurityRolePrivilege",
    table = "security_role_privilege_data",
    data_service = "postgres"
)]
pub struct SecurityRolePrivilege {
    #[teaql(id)]
    id: u64,
    #[teaql(version)]
    version: i64,
    // @source model.xml:218
    #[teaql(column = "tenant")]
    tenant_id: u64,

    // @source model.xml:218
    #[teaql(column = "security_role")]
    security_role_id: u64,

    // @source model.xml:218
    #[teaql(column = "security_privilege")]
    security_privilege_id: u64,
    // @source model.xml:218
    #[teaql(relation(target = "Tenant", local_key = "tenant_id", foreign_key = "id"))]
    tenant: Option<Box<crate::Tenant>>,

    // @source model.xml:218
    #[teaql(relation(
        target = "SecurityRole",
        local_key = "security_role_id",
        foreign_key = "id"
    ))]
    security_role: Option<Box<crate::SecurityRole>>,

    // @source model.xml:218
    #[teaql(relation(
        target = "SecurityPrivilege",
        local_key = "security_privilege_id",
        foreign_key = "id"
    ))]
    security_privilege: Option<Box<crate::SecurityPrivilege>>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl SecurityRolePrivilege {
    pub const ENTITY_NAME: &'static str = "Security Role Privilege Assignment";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            version: 0_i64,
            tenant_id: 0_u64,
            security_role_id: 0_u64,
            security_privilege_id: 0_u64,
            tenant: None,
            security_role: None,
            security_privilege: None,
            dynamic: BTreeMap::new(),
            __teaql_runtime_state: root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn attach_runtime_state_recursive(&mut self, root: teaql_runtime::EntityRuntimeState) {
        root.adopt_mutations_from(self.__teaql_runtime_state());
        self.__teaql_replace_runtime_state(root.clone());
        if let Some(entity) = &mut self.tenant {
            entity.attach_runtime_state_recursive(root.clone());
        }
        if let Some(entity) = &mut self.security_role {
            entity.attach_runtime_state_recursive(root.clone());
        }
        if let Some(entity) = &mut self.security_privilege {
            entity.attach_runtime_state_recursive(root.clone());
        }
    }

    pub fn is_loaded(&self, field_or_relation: &str) -> bool {
        self.__load_state.is_loaded(field_or_relation)
    }

    pub fn set_load_state(&mut self, state: teaql_core::eval::LoadState) {
        self.__load_state = state;
    }

    pub fn id(&self) -> u64 {
        self.changed_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.id)
    }

    pub fn update_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.id = value.try_u64().unwrap_or(self.id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "id", value);
        self
    }

    pub fn changed_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state().get(&self.entity_key(), "id")
    }

    pub fn eval_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "id".to_string(),
                attempted_path: "id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.id())
        }
    }

    pub fn version(&self) -> i64 {
        self.changed_version()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.version)
    }

    pub fn update_version(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.version = value.try_i64().unwrap_or(self.version.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "version", value);
        self
    }

    pub fn changed_version(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "version")
    }

    pub fn eval_version(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("version") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "version".to_string(),
                attempted_path: "version".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.version())
        }
    }
    pub fn tenant_id(&self) -> u64 {
        self.changed_tenant_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.tenant_id)
    }

    pub fn update_tenant_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.tenant_id = value.try_u64().unwrap_or(self.tenant_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "tenant_id", value);
        self
    }

    pub fn changed_tenant_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "tenant_id")
    }

    pub fn eval_tenant_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("tenant_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "tenant_id".to_string(),
                attempted_path: "tenant_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.tenant_id())
        }
    }

    pub fn security_role_id(&self) -> u64 {
        self.changed_security_role_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.security_role_id)
    }

    pub fn update_security_role_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.security_role_id = value.try_u64().unwrap_or(self.security_role_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "security_role_id", value);
        self
    }

    pub fn changed_security_role_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "security_role_id")
    }

    pub fn eval_security_role_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("security_role_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_role_id".to_string(),
                attempted_path: "security_role_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.security_role_id())
        }
    }

    pub fn security_privilege_id(&self) -> u64 {
        self.changed_security_privilege_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.security_privilege_id)
    }

    pub fn update_security_privilege_id(
        &mut self,
        value: impl Into<teaql_core::Value>,
    ) -> &mut Self {
        let value = value.into();
        self.security_privilege_id = value
            .try_u64()
            .unwrap_or(self.security_privilege_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "security_privilege_id", value);
        self
    }

    pub fn changed_security_privilege_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "security_privilege_id")
    }

    pub fn eval_security_privilege_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("security_privilege_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_privilege_id".to_string(),
                attempted_path: "security_privilege_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.security_privilege_id())
        }
    }
    pub fn tenant(&self) -> Option<&crate::Tenant> {
        self.tenant.as_deref().or_else(|| {
            self.__teaql_runtime_state()
                .resolve_entity(self.tenant_id())
        })
    }

    pub fn eval_tenant(&self) -> teaql_core::eval::EvalResult<&crate::Tenant> {
        match self.tenant() {
            Some(v) => teaql_core::eval::EvalResult::Value(v),
            None if self.is_loaded("tenant") => teaql_core::eval::EvalResult::Null,
            None => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "tenant".to_string(),
                attempted_path: "tenant".to_string(),
            },
        }
    }

    pub fn security_role(&self) -> Option<&crate::SecurityRole> {
        self.security_role.as_deref().or_else(|| {
            self.__teaql_runtime_state()
                .resolve_entity(self.security_role_id())
        })
    }

    pub fn eval_security_role(&self) -> teaql_core::eval::EvalResult<&crate::SecurityRole> {
        match self.security_role() {
            Some(v) => teaql_core::eval::EvalResult::Value(v),
            None if self.is_loaded("security_role") => teaql_core::eval::EvalResult::Null,
            None => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_role".to_string(),
                attempted_path: "security_role".to_string(),
            },
        }
    }

    pub fn security_privilege(&self) -> Option<&crate::SecurityPrivilege> {
        self.security_privilege.as_deref().or_else(|| {
            self.__teaql_runtime_state()
                .resolve_entity(self.security_privilege_id())
        })
    }

    pub fn eval_security_privilege(
        &self,
    ) -> teaql_core::eval::EvalResult<&crate::SecurityPrivilege> {
        match self.security_privilege() {
            Some(v) => teaql_core::eval::EvalResult::Value(v),
            None if self.is_loaded("security_privilege") => teaql_core::eval::EvalResult::Null,
            None => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_privilege".to_string(),
                attempted_path: "security_privilege".to_string(),
            },
        }
    }
}
