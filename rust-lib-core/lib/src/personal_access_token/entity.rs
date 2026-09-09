// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/personal_access_token
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
    entity = "PersonalAccessToken",
    table = "personal_access_token_data",
    data_service = "postgres",
    audit_mask_fields = "token_id,token_hash"
)]
pub struct PersonalAccessToken {
    #[teaql(id)]
    id: u64,

    // @source model.xml:236
    username: String,

    // @source model.xml:236
    token_id: String,

    // @source model.xml:236
    token_hash: String,

    // @source model.xml:236
    description: String,

    // @source model.xml:236
    scopes: String,

    // @source model.xml:236
    created_at: teaql_core::time::Timestamp,

    // @source model.xml:236
    expires_at_epoch_millis: i64,

    // @source model.xml:236
    revoked: bool,

    // @source model.xml:236
    revoked_at_epoch_millis: i64,
    #[teaql(version)]
    version: i64,
    // @source model.xml:236
    #[teaql(column = "tenant")]
    tenant_id: u64,

    // @source model.xml:236
    #[teaql(column = "security_user")]
    security_user_id: u64,
    // @source model.xml:236
    #[teaql(relation(target = "Tenant", local_key = "tenant_id", foreign_key = "id"))]
    tenant: Option<Box<crate::Tenant>>,

    // @source model.xml:236
    #[teaql(relation(
        target = "SecurityUser",
        local_key = "security_user_id",
        foreign_key = "id"
    ))]
    security_user: Option<Box<crate::SecurityUser>>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl PersonalAccessToken {
    pub const ENTITY_NAME: &'static str = "Personal Access Token";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            username: String::new(),
            token_id: String::new(),
            token_hash: String::new(),
            description: String::new(),
            scopes: String::new(),
            created_at: teaql_core::time::Timestamp::now(),
            expires_at_epoch_millis: 0_i64,
            revoked: false,
            revoked_at_epoch_millis: 0_i64,
            version: 0_i64,
            tenant_id: 0_u64,
            security_user_id: 0_u64,
            tenant: None,
            security_user: None,
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
        if let Some(entity) = &mut self.security_user {
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

    pub fn username(&self) -> String {
        self.changed_username()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.username.clone())
    }

    pub fn update_username(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.username = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.username.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "username", value);
        self
    }

    pub fn changed_username(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "username")
    }

    pub fn eval_username(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("username") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "username".to_string(),
                attempted_path: "username".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.username())
        }
    }

    pub fn token_id(&self) -> String {
        self.changed_token_id()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.token_id.clone())
    }

    pub fn update_token_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.token_id = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.token_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "token_id", value);
        self
    }

    pub fn changed_token_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "token_id")
    }

    pub fn eval_token_id(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("token_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "token_id".to_string(),
                attempted_path: "token_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.token_id())
        }
    }

    pub fn token_hash(&self) -> String {
        self.changed_token_hash()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.token_hash.clone())
    }

    pub fn update_token_hash(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.token_hash = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.token_hash.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "token_hash", value);
        self
    }

    pub fn changed_token_hash(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "token_hash")
    }

    pub fn eval_token_hash(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("token_hash") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "token_hash".to_string(),
                attempted_path: "token_hash".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.token_hash())
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

    pub fn scopes(&self) -> String {
        self.changed_scopes()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.scopes.clone())
    }

    pub fn update_scopes(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.scopes = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.scopes.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "scopes", value);
        self
    }

    pub fn changed_scopes(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "scopes")
    }

    pub fn eval_scopes(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("scopes") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "scopes".to_string(),
                attempted_path: "scopes".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.scopes())
        }
    }

    pub fn created_at(&self) -> teaql_core::time::Timestamp {
        self.changed_created_at()
            .and_then(|value| value.try_timestamp())
            .unwrap_or(self.created_at)
    }

    pub fn update_created_at(&mut self, value: teaql_core::time::Timestamp) -> &mut Self {
        self.created_at = value;
        let value = teaql_core::Value::from(value);
        self.__teaql_runtime_state()
            .set(self.entity_key(), "created_at", value);
        self
    }
    pub fn changed_created_at(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "created_at")
    }

    pub fn eval_created_at(&self) -> teaql_core::eval::EvalResult<teaql_core::time::Timestamp> {
        if !self.is_loaded("created_at") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "created_at".to_string(),
                attempted_path: "created_at".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.created_at())
        }
    }

    pub fn expires_at_epoch_millis(&self) -> i64 {
        self.changed_expires_at_epoch_millis()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.expires_at_epoch_millis)
    }

    pub fn update_expires_at_epoch_millis(
        &mut self,
        value: impl Into<teaql_core::Value>,
    ) -> &mut Self {
        let value = value.into();
        self.expires_at_epoch_millis = value
            .try_i64()
            .unwrap_or(self.expires_at_epoch_millis.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "expires_at_epoch_millis", value);
        self
    }

    pub fn changed_expires_at_epoch_millis(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "expires_at_epoch_millis")
    }

    pub fn eval_expires_at_epoch_millis(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("expires_at_epoch_millis") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "expires_at_epoch_millis".to_string(),
                attempted_path: "expires_at_epoch_millis".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.expires_at_epoch_millis())
        }
    }

    pub fn revoked(&self) -> bool {
        self.changed_revoked()
            .and_then(|value| value.try_bool())
            .unwrap_or(self.revoked)
    }

    pub fn update_revoked(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.revoked = value.try_bool().unwrap_or(self.revoked.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "revoked", value);
        self
    }

    pub fn changed_revoked(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "revoked")
    }

    pub fn eval_revoked(&self) -> teaql_core::eval::EvalResult<bool> {
        if !self.is_loaded("revoked") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "revoked".to_string(),
                attempted_path: "revoked".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.revoked())
        }
    }

    pub fn revoked_at_epoch_millis(&self) -> i64 {
        self.changed_revoked_at_epoch_millis()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.revoked_at_epoch_millis)
    }

    pub fn update_revoked_at_epoch_millis(
        &mut self,
        value: impl Into<teaql_core::Value>,
    ) -> &mut Self {
        let value = value.into();
        self.revoked_at_epoch_millis = value
            .try_i64()
            .unwrap_or(self.revoked_at_epoch_millis.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "revoked_at_epoch_millis", value);
        self
    }

    pub fn changed_revoked_at_epoch_millis(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "revoked_at_epoch_millis")
    }

    pub fn eval_revoked_at_epoch_millis(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("revoked_at_epoch_millis") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "revoked_at_epoch_millis".to_string(),
                attempted_path: "revoked_at_epoch_millis".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.revoked_at_epoch_millis())
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

    pub fn security_user_id(&self) -> u64 {
        self.changed_security_user_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.security_user_id)
    }

    pub fn update_security_user_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.security_user_id = value.try_u64().unwrap_or(self.security_user_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "security_user_id", value);
        self
    }

    pub fn changed_security_user_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "security_user_id")
    }

    pub fn eval_security_user_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("security_user_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_user_id".to_string(),
                attempted_path: "security_user_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.security_user_id())
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

    pub fn security_user(&self) -> Option<&crate::SecurityUser> {
        self.security_user.as_deref().or_else(|| {
            self.__teaql_runtime_state()
                .resolve_entity(self.security_user_id())
        })
    }

    pub fn eval_security_user(&self) -> teaql_core::eval::EvalResult<&crate::SecurityUser> {
        match self.security_user() {
            Some(v) => teaql_core::eval::EvalResult::Value(v),
            None if self.is_loaded("security_user") => teaql_core::eval::EvalResult::Null,
            None => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_user".to_string(),
                attempted_path: "security_user".to_string(),
            },
        }
    }
}
