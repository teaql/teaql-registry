
// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/security_user
use std::collections::BTreeMap;

use teaql_macros::TeaqlEntity;

/// [TEAQL AI WARNING]
/// TeaQL was explicitly designed to PREVENT AI hallucinations and random guessing.
/// DO NOT GUESS METHOD NAMES!
/// The methods listed below are the ONLY valid ways to interact with this entity.
/// If you encounter compilation errors (e.g., method not found), DO NOT guess another method name.
/// Read the method signatures in this file before proceeding.
#[derive(Clone, Debug, PartialEq, TeaqlEntity)]
#[teaql(entity = "SecurityUser", table = "security_user_data", data_service = "postgres", audit_mask_fields = "password_hash,email")]
pub struct SecurityUser {
#[teaql(id)]
    id: u64,

// @source model.xml:179
    username: String,

// @source model.xml:179
    first_name: String,

// @source model.xml:179
    last_name: String,

// @source model.xml:179
    password_hash: String,

// @source model.xml:179
    email: String,
#[teaql(version)]
    version: i64,
// @source model.xml:179
#[teaql(column = "tenant")]
    tenant_id: u64,

// @source model.xml:179
#[teaql(column = "user_status")]
    user_status_id: u64,
// @source model.xml:179
#[teaql(relation(target = "Tenant", local_key = "tenant_id", foreign_key = "id"))]
    tenant: Option<crate::Tenant>,

// @source model.xml:179
#[teaql(relation(target = "UserStatus", local_key = "user_status_id", foreign_key = "id"))]
    user_status: Option<crate::UserStatus>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    root: teaql_runtime::EntityRoot,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl SecurityUser {
    pub const ENTITY_NAME: &'static str = "Security User";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRoot) -> Self {
        Self {
            id: 0_u64,
            username: String::new(),
            first_name: String::new(),
            last_name: String::new(),
            password_hash: String::new(),
            email: String::new(),
            version: 0_i64,
            tenant_id: 0_u64,
            user_status_id: 0_u64,
            tenant: None,
            user_status: None,
            dynamic: BTreeMap::new(),
            root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn entity_key(&self) -> teaql_runtime::EntityKey {
        teaql_runtime::EntityKey::new("SecurityUser", self.id)
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
        self.root = root.clone();
        if let Some(entity) = &mut self.tenant {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.user_status {
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

    pub fn username(&self) -> String {
        self.changed_username().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.username.clone())
    }

    pub fn update_username(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.username = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.username.clone());
        self.root.set(self.entity_key(), "username", value);
        self
    }

    pub fn changed_username(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "username")
    }

    pub fn eval_username(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("username") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "username".to_string(), attempted_path: "username".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.username())
                }}

    pub fn first_name(&self) -> String {
        self.changed_first_name().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.first_name.clone())
    }

    pub fn update_first_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.first_name = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.first_name.clone());
        self.root.set(self.entity_key(), "first_name", value);
        self
    }

    pub fn changed_first_name(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "first_name")
    }

    pub fn eval_first_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("first_name") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "first_name".to_string(), attempted_path: "first_name".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.first_name())
                }}

    pub fn last_name(&self) -> String {
        self.changed_last_name().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.last_name.clone())
    }

    pub fn update_last_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.last_name = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.last_name.clone());
        self.root.set(self.entity_key(), "last_name", value);
        self
    }

    pub fn changed_last_name(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "last_name")
    }

    pub fn eval_last_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("last_name") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "last_name".to_string(), attempted_path: "last_name".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.last_name())
                }}

    pub fn password_hash(&self) -> String {
        self.changed_password_hash().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.password_hash.clone())
    }

    pub fn update_password_hash(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.password_hash = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.password_hash.clone());
        self.root.set(self.entity_key(), "password_hash", value);
        self
    }

    pub fn changed_password_hash(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "password_hash")
    }

    pub fn eval_password_hash(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("password_hash") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "password_hash".to_string(), attempted_path: "password_hash".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.password_hash())
                }}

    pub fn email(&self) -> String {
        self.changed_email().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.email.clone())
    }

    pub fn update_email(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.email = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.email.clone());
        self.root.set(self.entity_key(), "email", value);
        self
    }

    pub fn changed_email(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "email")
    }

    pub fn eval_email(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("email") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "email".to_string(), attempted_path: "email".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.email())
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
    pub fn tenant_id(&self) -> u64 {
        self.changed_tenant_id().and_then(|value| value.try_u64()).unwrap_or(self.tenant_id)
    }

    pub fn update_tenant_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.tenant_id = value.try_u64().unwrap_or(self.tenant_id.clone());
        self.root.set(self.entity_key(), "tenant_id", value);
        self
    }

    pub fn changed_tenant_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "tenant_id")
    }

    pub fn eval_tenant_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("tenant_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "tenant_id".to_string(), attempted_path: "tenant_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.tenant_id())
                }}

    pub fn user_status_id(&self) -> u64 {
        self.changed_user_status_id().and_then(|value| value.try_u64()).unwrap_or(self.user_status_id)
    }

    pub(crate) fn update_user_status_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.user_status_id = value.try_u64().unwrap_or(self.user_status_id.clone());
        self.root.set(self.entity_key(), "user_status_id", value);
        self
    }

    pub fn changed_user_status_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "user_status_id")
    }

    pub fn eval_user_status_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("user_status_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "user_status_id".to_string(), attempted_path: "user_status_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.user_status_id())
                }}
    pub fn update_user_status_to_active(&mut self) -> &mut Self {
        self.update_user_status_id(1001_u64)
    }

    pub fn user_status_is_active(&self) -> bool {
        self.user_status_id() == 1001_u64
    }
    pub fn update_user_status_to_disabled(&mut self) -> &mut Self {
        self.update_user_status_id(1002_u64)
    }

    pub fn user_status_is_disabled(&self) -> bool {
        self.user_status_id() == 1002_u64
    }
    pub fn update_user_status_to_locked(&mut self) -> &mut Self {
        self.update_user_status_id(1003_u64)
    }

    pub fn user_status_is_locked(&self) -> bool {
        self.user_status_id() == 1003_u64
    }
    pub fn tenant(&self) -> Option<&crate::Tenant> {
        self.tenant.as_ref()
    }

    pub fn eval_tenant(&self) -> teaql_core::eval::EvalResult<&crate::Tenant> {
        if !self.is_loaded("tenant") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "tenant".to_string(), attempted_path: "tenant".to_string() }
        } else {
            match &self.tenant {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }

    pub fn user_status(&self) -> Option<&crate::UserStatus> {
        self.user_status.as_ref()
    }

    pub fn eval_user_status(&self) -> teaql_core::eval::EvalResult<&crate::UserStatus> {
        if !self.is_loaded("user_status") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "user_status".to_string(), attempted_path: "user_status".to_string() }
        } else {
            match &self.user_status {
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

