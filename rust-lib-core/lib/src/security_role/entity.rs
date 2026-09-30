// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/security_role
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
#[teaql(reverse_relation(
    name = "security_user_role_list",
    target = "SecurityUserRole",
    local_key = "id",
    foreign_key = "security_role_id",
    many
))]
#[teaql(reverse_relation(
    name = "security_role_privilege_list",
    target = "SecurityRolePrivilege",
    local_key = "id",
    foreign_key = "security_role_id",
    many
))]
#[teaql(
    entity = "SecurityRole",
    table = "security_role_data",
    data_service = "postgres"
)]
pub struct SecurityRole {
    #[teaql(id)]
    id: u64,

    // @source model.xml:191
    role_id: String,

    // @source model.xml:191
    name: String,

    // @source model.xml:191
    description: String,

    // @source model.xml:191
    read_only: bool,
    #[teaql(version)]
    version: i64,
    // @source model.xml:191
    #[teaql(column = "tenant")]
    tenant_id: u64,
    // @source model.xml:191
    #[teaql(relation(target = "Tenant", local_key = "tenant_id", foreign_key = "id"))]
    tenant: Option<Box<crate::Tenant>>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl SecurityRole {
    pub const ENTITY_NAME: &'static str = "Security Role";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            role_id: String::new(),
            name: String::new(),
            description: String::new(),
            read_only: false,
            version: 0_i64,
            tenant_id: 0_u64,
            tenant: None,
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

    pub fn role_id(&self) -> String {
        self.changed_role_id()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.role_id.clone())
    }

    pub fn update_role_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.role_id = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.role_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "role_id", value);
        self
    }

    pub fn changed_role_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "role_id")
    }

    pub fn eval_role_id(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("role_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "role_id".to_string(),
                attempted_path: "role_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.role_id())
        }
    }

    pub fn name(&self) -> String {
        self.changed_name()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.name.clone())
    }

    pub fn update_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.name = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.name.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "name", value);
        self
    }

    pub fn changed_name(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state().get(&self.entity_key(), "name")
    }

    pub fn eval_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("name") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "name".to_string(),
                attempted_path: "name".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.name())
        }
    }

    pub fn description(&self) -> String {
        self.changed_description()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.description.clone())
    }

    pub fn update_description(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.description = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.description.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "description", value);
        self
    }

    pub fn changed_description(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "description")
    }

    pub fn eval_description(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("description") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "description".to_string(),
                attempted_path: "description".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.description())
        }
    }

    pub fn read_only(&self) -> bool {
        self.changed_read_only()
            .and_then(|value| value.try_bool())
            .unwrap_or(self.read_only)
    }

    pub fn update_read_only(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.read_only = value.try_bool().unwrap_or(self.read_only.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "read_only", value);
        self
    }

    pub fn changed_read_only(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "read_only")
    }

    pub fn eval_read_only(&self) -> teaql_core::eval::EvalResult<bool> {
        if !self.is_loaded("read_only") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "read_only".to_string(),
                attempted_path: "read_only".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.read_only())
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
    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn security_user_role_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::SecurityUserRole>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "security_user_role_list",
        )
    }

    pub fn eval_security_user_role_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::SecurityUserRole>> {
        let relation = self.security_user_role_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_user_role_list".to_string(),
                attempted_path: "security_user_role_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn security_role_privilege_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::SecurityRolePrivilege>>
    {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "security_role_privilege_list",
        )
    }

    pub fn eval_security_role_privilege_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::SecurityRolePrivilege>> {
        let relation = self.security_role_privilege_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_role_privilege_list".to_string(),
                attempted_path: "security_role_privilege_list".to_string(),
            },
        }
    }
}
