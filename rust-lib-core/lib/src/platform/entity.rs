
// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/platform
use std::collections::BTreeMap;

use teaql_core::SmartList;
use teaql_macros::TeaqlEntity;

/// [TEAQL AI WARNING]
/// TeaQL was explicitly designed to PREVENT AI hallucinations and random guessing.
/// DO NOT GUESS METHOD NAMES!
/// The methods listed below are the ONLY valid ways to interact with this entity.
/// If you encounter compilation errors (e.g., method not found), DO NOT guess another method name.
/// Read the method signatures in this file before proceeding.
#[derive(Clone, Debug, PartialEq, TeaqlEntity)]
#[teaql(entity = "Platform", table = "platform_data", data_service = "postgres")]
pub struct Platform {
#[teaql(id)]
    id: u64,

// @source model.xml:12
    name: String,

// @source model.xml:12
    platform_version: String,
#[teaql(version)]
    version: i64,
#[teaql(relation(target = "Tenant", local_key = "id", foreign_key = "platform_id", many))]
    tenant_list: SmartList<crate::Tenant>,
#[teaql(relation(target = "RepositoryType", local_key = "id", foreign_key = "platform_id", many))]
    repository_type_list: SmartList<crate::RepositoryType>,
#[teaql(relation(target = "RepositoryFormat", local_key = "id", foreign_key = "platform_id", many))]
    repository_format_list: SmartList<crate::RepositoryFormat>,
#[teaql(relation(target = "WritePolicy", local_key = "id", foreign_key = "platform_id", many))]
    write_policy_list: SmartList<crate::WritePolicy>,
#[teaql(relation(target = "BlobStoreType", local_key = "id", foreign_key = "platform_id", many))]
    blob_store_type_list: SmartList<crate::BlobStoreType>,
#[teaql(relation(target = "UserStatus", local_key = "id", foreign_key = "platform_id", many))]
    user_status_list: SmartList<crate::UserStatus>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    root: teaql_runtime::EntityRoot,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl Platform {
    pub const ENTITY_NAME: &'static str = "Nexus Platform";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRoot) -> Self {
        Self {
            id: 0_u64,
            name: String::new(),
            platform_version: String::new(),
            version: 0_i64,
            tenant_list: Default::default(),
            repository_type_list: Default::default(),
            repository_format_list: Default::default(),
            write_policy_list: Default::default(),
            blob_store_type_list: Default::default(),
            user_status_list: Default::default(),
            dynamic: BTreeMap::new(),
            root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn entity_key(&self) -> teaql_runtime::EntityKey {
        teaql_runtime::EntityKey::new("Platform", self.id)
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
        self.root = root.clone();
        for entity in &mut self.tenant_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.repository_type_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.repository_format_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.write_policy_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.blob_store_type_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.user_status_list {
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

    pub fn platform_version(&self) -> String {
        self.changed_platform_version().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.platform_version.clone())
    }

    pub fn update_platform_version(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.platform_version = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.platform_version.clone());
        self.root.set(self.entity_key(), "platform_version", value);
        self
    }

    pub fn changed_platform_version(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "platform_version")
    }

    pub fn eval_platform_version(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("platform_version") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "platform_version".to_string(), attempted_path: "platform_version".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.platform_version())
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
    pub fn tenant_list(&self) -> &SmartList<crate::Tenant> {
        &self.tenant_list
    }

    pub fn tenant_list_mut(&mut self) -> &mut SmartList<crate::Tenant> {
        &mut self.tenant_list
    }

    pub fn eval_tenant_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::Tenant>> {
        if !self.is_loaded("tenant_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "tenant_list".to_string(), attempted_path: "tenant_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.tenant_list)
        }
    }

    pub fn repository_type_list(&self) -> &SmartList<crate::RepositoryType> {
        &self.repository_type_list
    }

    pub fn repository_type_list_mut(&mut self) -> &mut SmartList<crate::RepositoryType> {
        &mut self.repository_type_list
    }

    pub fn eval_repository_type_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::RepositoryType>> {
        if !self.is_loaded("repository_type_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_type_list".to_string(), attempted_path: "repository_type_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.repository_type_list)
        }
    }

    pub fn repository_format_list(&self) -> &SmartList<crate::RepositoryFormat> {
        &self.repository_format_list
    }

    pub fn repository_format_list_mut(&mut self) -> &mut SmartList<crate::RepositoryFormat> {
        &mut self.repository_format_list
    }

    pub fn eval_repository_format_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::RepositoryFormat>> {
        if !self.is_loaded("repository_format_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_format_list".to_string(), attempted_path: "repository_format_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.repository_format_list)
        }
    }

    pub fn write_policy_list(&self) -> &SmartList<crate::WritePolicy> {
        &self.write_policy_list
    }

    pub fn write_policy_list_mut(&mut self) -> &mut SmartList<crate::WritePolicy> {
        &mut self.write_policy_list
    }

    pub fn eval_write_policy_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::WritePolicy>> {
        if !self.is_loaded("write_policy_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "write_policy_list".to_string(), attempted_path: "write_policy_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.write_policy_list)
        }
    }

    pub fn blob_store_type_list(&self) -> &SmartList<crate::BlobStoreType> {
        &self.blob_store_type_list
    }

    pub fn blob_store_type_list_mut(&mut self) -> &mut SmartList<crate::BlobStoreType> {
        &mut self.blob_store_type_list
    }

    pub fn eval_blob_store_type_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::BlobStoreType>> {
        if !self.is_loaded("blob_store_type_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_type_list".to_string(), attempted_path: "blob_store_type_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.blob_store_type_list)
        }
    }

    pub fn user_status_list(&self) -> &SmartList<crate::UserStatus> {
        &self.user_status_list
    }

    pub fn user_status_list_mut(&mut self) -> &mut SmartList<crate::UserStatus> {
        &mut self.user_status_list
    }

    pub fn eval_user_status_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::UserStatus>> {
        if !self.is_loaded("user_status_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "user_status_list".to_string(), attempted_path: "user_status_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.user_status_list)
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

