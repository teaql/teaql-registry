
// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/security_privilege
use std::collections::BTreeMap;

use teaql_macros::TeaqlEntity;

/// [TEAQL AI WARNING]
/// TeaQL was explicitly designed to PREVENT AI hallucinations and random guessing.
/// DO NOT GUESS METHOD NAMES!
/// The methods listed below are the ONLY valid ways to interact with this entity.
/// If you encounter compilation errors (e.g., method not found), DO NOT guess another method name.
/// Read the method signatures in this file before proceeding.
#[derive(Clone, Debug, PartialEq, TeaqlEntity)]
#[teaql(entity = "SecurityPrivilege", table = "security_privilege_data", data_service = "postgres")]
pub struct SecurityPrivilege {
#[teaql(id)]
    id: u64,

// @source model.xml:190
    privilege_id: String,

// @source model.xml:190
    name: String,

// @source model.xml:190
    description: String,

// @source model.xml:190
    privilege_type: String,

// @source model.xml:190
    permission_pattern: String,

// @source model.xml:190
    read_only: bool,
#[teaql(version)]
    version: i64,
// @source model.xml:190
#[teaql(column = "platform")]
    platform_id: u64,
// @source model.xml:190
#[teaql(relation(target = "Platform", local_key = "platform_id", foreign_key = "id"))]
    platform: Option<crate::Platform>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    root: teaql_runtime::EntityRoot,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl SecurityPrivilege {
    pub const ENTITY_NAME: &'static str = "Security Privilege";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRoot) -> Self {
        Self {
            id: 0_u64,
            privilege_id: String::new(),
            name: String::new(),
            description: String::new(),
            privilege_type: String::new(),
            permission_pattern: String::new(),
            read_only: false,
            version: 0_i64,
            platform_id: 0_u64,
            platform: None,
            dynamic: BTreeMap::new(),
            root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn entity_key(&self) -> teaql_runtime::EntityKey {
        teaql_runtime::EntityKey::new("SecurityPrivilege", self.id)
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
        self.root = root.clone();
        if let Some(entity) = &mut self.platform {
            entity.attach_root_recursive(root.clone());
        }
    }

    pub fn is_loaded(&self, field_or_relation: &str) -> bool {
        self.__load_state.is_loaded(field_or_relation)
    }

    pub fn set_load_state(&mut self, state: teaql_core::eval::LoadState) {
        self.__load_state = state;
    }

    pub fn id(&self) -> u64 {
        self.changed_id().and_then(|value| value.try_u64()).unwrap_or(self.id)
    }

    pub fn update_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.id = value.try_u64().unwrap_or(self.id.clone());
        self.root.set(self.entity_key(), "id", value);
        self
    }

    pub fn changed_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "id")
    }

    pub fn eval_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "id".to_string(), attempted_path: "id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.id())
                }}

    pub fn privilege_id(&self) -> String {
        self.changed_privilege_id().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.privilege_id.clone())
    }

    pub fn update_privilege_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.privilege_id = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.privilege_id.clone());
        self.root.set(self.entity_key(), "privilege_id", value);
        self
    }

    pub fn changed_privilege_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "privilege_id")
    }

    pub fn eval_privilege_id(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("privilege_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "privilege_id".to_string(), attempted_path: "privilege_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.privilege_id())
                }}

    pub fn name(&self) -> String {
        self.changed_name().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.name.clone())
    }

    pub fn update_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.name = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.name.clone());
        self.root.set(self.entity_key(), "name", value);
        self
    }

    pub fn changed_name(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "name")
    }

    pub fn eval_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("name") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "name".to_string(), attempted_path: "name".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.name())
                }}

    pub fn description(&self) -> String {
        self.changed_description().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.description.clone())
    }

    pub fn update_description(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.description = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.description.clone());
        self.root.set(self.entity_key(), "description", value);
        self
    }

    pub fn changed_description(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "description")
    }

    pub fn eval_description(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("description") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "description".to_string(), attempted_path: "description".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.description())
                }}

    pub fn privilege_type(&self) -> String {
        self.changed_privilege_type().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.privilege_type.clone())
    }

    pub fn update_privilege_type(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.privilege_type = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.privilege_type.clone());
        self.root.set(self.entity_key(), "privilege_type", value);
        self
    }

    pub fn changed_privilege_type(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "privilege_type")
    }

    pub fn eval_privilege_type(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("privilege_type") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "privilege_type".to_string(), attempted_path: "privilege_type".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.privilege_type())
                }}

    pub fn permission_pattern(&self) -> String {
        self.changed_permission_pattern().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.permission_pattern.clone())
    }

    pub fn update_permission_pattern(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.permission_pattern = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.permission_pattern.clone());
        self.root.set(self.entity_key(), "permission_pattern", value);
        self
    }

    pub fn changed_permission_pattern(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "permission_pattern")
    }

    pub fn eval_permission_pattern(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("permission_pattern") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "permission_pattern".to_string(), attempted_path: "permission_pattern".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.permission_pattern())
                }}

    pub fn read_only(&self) -> bool {
        self.changed_read_only().and_then(|value| value.try_bool()).unwrap_or(self.read_only)
    }

    pub fn update_read_only(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.read_only = value.try_bool().unwrap_or(self.read_only.clone());
        self.root.set(self.entity_key(), "read_only", value);
        self
    }

    pub fn changed_read_only(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "read_only")
    }

    pub fn eval_read_only(&self) -> teaql_core::eval::EvalResult<bool> {
        if !self.is_loaded("read_only") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "read_only".to_string(), attempted_path: "read_only".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.read_only())
                }}

    pub fn version(&self) -> i64 {
        self.changed_version().and_then(|value| value.try_i64()).unwrap_or(self.version)
    }

    pub fn update_version(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.version = value.try_i64().unwrap_or(self.version.clone());
        self.root.set(self.entity_key(), "version", value);
        self
    }

    pub fn changed_version(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "version")
    }

    pub fn eval_version(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("version") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "version".to_string(), attempted_path: "version".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.version())
                }}
    pub fn platform_id(&self) -> u64 {
        self.changed_platform_id().and_then(|value| value.try_u64()).unwrap_or(self.platform_id)
    }

    pub fn update_platform_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.platform_id = value.try_u64().unwrap_or(self.platform_id.clone());
        self.root.set(self.entity_key(), "platform_id", value);
        self
    }

    pub fn changed_platform_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "platform_id")
    }

    pub fn eval_platform_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("platform_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "platform_id".to_string(), attempted_path: "platform_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.platform_id())
                }}
    pub fn platform(&self) -> Option<&crate::Platform> {
        self.platform.as_ref()
    }

    pub fn eval_platform(&self) -> teaql_core::eval::EvalResult<&crate::Platform> {
        if !self.is_loaded("platform") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "platform".to_string(), attempted_path: "platform".to_string() }
        } else {
            match &self.platform {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }

    pub fn mark_as_delete(&mut self) -> &mut Self {
        self.root.mark_as_delete(self.entity_key());
        self
    }

    pub fn set_comment(&mut self, comment: impl Into<String>) -> &mut Self {
        self.root.set_comment(comment);
        self
    }
}

