
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
    version: String,
    #[teaql(boxed_relations)]
    pub _relations: Box<PlatformReverseRelations>,
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
            version: String::new(),
            _relations: Box::new(PlatformReverseRelations::new()),
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
        self._relations.attach_root_recursive(root.clone());
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

    pub fn version(&self) -> String {
        self.changed_version().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.version.clone())
    }

    pub fn update_version(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.version = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.version.clone());
        self.root.set(self.entity_key(), "version", value);
        self
    }

    pub fn changed_version(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "version")
    }

    pub fn eval_version(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("version") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "version".to_string(), attempted_path: "version".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.version())
                }}
    pub fn repository_type_list(&self) -> &SmartList<crate::RepositoryType> {
        &self._relations.repository_type_list
    }

    pub fn repository_type_list_mut(&mut self) -> &mut SmartList<crate::RepositoryType> {
        &mut self._relations.repository_type_list
    }

    pub fn eval_repository_type_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::RepositoryType>> {
        if !self.is_loaded("repository_type_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_type_list".to_string(), attempted_path: "repository_type_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.repository_type_list)
        }
    }

    pub fn repository_format_list(&self) -> &SmartList<crate::RepositoryFormat> {
        &self._relations.repository_format_list
    }

    pub fn repository_format_list_mut(&mut self) -> &mut SmartList<crate::RepositoryFormat> {
        &mut self._relations.repository_format_list
    }

    pub fn eval_repository_format_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::RepositoryFormat>> {
        if !self.is_loaded("repository_format_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_format_list".to_string(), attempted_path: "repository_format_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.repository_format_list)
        }
    }

    pub fn write_policy_list(&self) -> &SmartList<crate::WritePolicy> {
        &self._relations.write_policy_list
    }

    pub fn write_policy_list_mut(&mut self) -> &mut SmartList<crate::WritePolicy> {
        &mut self._relations.write_policy_list
    }

    pub fn eval_write_policy_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::WritePolicy>> {
        if !self.is_loaded("write_policy_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "write_policy_list".to_string(), attempted_path: "write_policy_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.write_policy_list)
        }
    }

    pub fn blob_store_type_list(&self) -> &SmartList<crate::BlobStoreType> {
        &self._relations.blob_store_type_list
    }

    pub fn blob_store_type_list_mut(&mut self) -> &mut SmartList<crate::BlobStoreType> {
        &mut self._relations.blob_store_type_list
    }

    pub fn eval_blob_store_type_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::BlobStoreType>> {
        if !self.is_loaded("blob_store_type_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_type_list".to_string(), attempted_path: "blob_store_type_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.blob_store_type_list)
        }
    }

    pub fn user_status_list(&self) -> &SmartList<crate::UserStatus> {
        &self._relations.user_status_list
    }

    pub fn user_status_list_mut(&mut self) -> &mut SmartList<crate::UserStatus> {
        &mut self._relations.user_status_list
    }

    pub fn eval_user_status_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::UserStatus>> {
        if !self.is_loaded("user_status_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "user_status_list".to_string(), attempted_path: "user_status_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.user_status_list)
        }
    }

    pub fn blob_store_configuration_list(&self) -> &SmartList<crate::BlobStoreConfiguration> {
        &self._relations.blob_store_configuration_list
    }

    pub fn blob_store_configuration_list_mut(&mut self) -> &mut SmartList<crate::BlobStoreConfiguration> {
        &mut self._relations.blob_store_configuration_list
    }

    pub fn eval_blob_store_configuration_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::BlobStoreConfiguration>> {
        if !self.is_loaded("blob_store_configuration_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_configuration_list".to_string(), attempted_path: "blob_store_configuration_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.blob_store_configuration_list)
        }
    }

    pub fn repository_configuration_list(&self) -> &SmartList<crate::RepositoryConfiguration> {
        &self._relations.repository_configuration_list
    }

    pub fn repository_configuration_list_mut(&mut self) -> &mut SmartList<crate::RepositoryConfiguration> {
        &mut self._relations.repository_configuration_list
    }

    pub fn eval_repository_configuration_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::RepositoryConfiguration>> {
        if !self.is_loaded("repository_configuration_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_configuration_list".to_string(), attempted_path: "repository_configuration_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.repository_configuration_list)
        }
    }

    pub fn security_user_list(&self) -> &SmartList<crate::SecurityUser> {
        &self._relations.security_user_list
    }

    pub fn security_user_list_mut(&mut self) -> &mut SmartList<crate::SecurityUser> {
        &mut self._relations.security_user_list
    }

    pub fn eval_security_user_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::SecurityUser>> {
        if !self.is_loaded("security_user_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "security_user_list".to_string(), attempted_path: "security_user_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.security_user_list)
        }
    }

    pub fn security_role_list(&self) -> &SmartList<crate::SecurityRole> {
        &self._relations.security_role_list
    }

    pub fn security_role_list_mut(&mut self) -> &mut SmartList<crate::SecurityRole> {
        &mut self._relations.security_role_list
    }

    pub fn eval_security_role_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::SecurityRole>> {
        if !self.is_loaded("security_role_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "security_role_list".to_string(), attempted_path: "security_role_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.security_role_list)
        }
    }

    pub fn security_privilege_list(&self) -> &SmartList<crate::SecurityPrivilege> {
        &self._relations.security_privilege_list
    }

    pub fn security_privilege_list_mut(&mut self) -> &mut SmartList<crate::SecurityPrivilege> {
        &mut self._relations.security_privilege_list
    }

    pub fn eval_security_privilege_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::SecurityPrivilege>> {
        if !self.is_loaded("security_privilege_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "security_privilege_list".to_string(), attempted_path: "security_privilege_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self._relations.security_privilege_list)
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

#[derive(Clone, Debug, PartialEq, teaql_macros::TeaqlReverseRelations)]
pub struct PlatformReverseRelations {
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
#[teaql(relation(target = "BlobStoreConfiguration", local_key = "id", foreign_key = "platform_id", many))]
    blob_store_configuration_list: SmartList<crate::BlobStoreConfiguration>,
#[teaql(relation(target = "RepositoryConfiguration", local_key = "id", foreign_key = "platform_id", many))]
    repository_configuration_list: SmartList<crate::RepositoryConfiguration>,
#[teaql(relation(target = "SecurityUser", local_key = "id", foreign_key = "platform_id", many))]
    security_user_list: SmartList<crate::SecurityUser>,
#[teaql(relation(target = "SecurityRole", local_key = "id", foreign_key = "platform_id", many))]
    security_role_list: SmartList<crate::SecurityRole>,
#[teaql(relation(target = "SecurityPrivilege", local_key = "id", foreign_key = "platform_id", many))]
    security_privilege_list: SmartList<crate::SecurityPrivilege>,
}

impl PlatformReverseRelations {
    pub fn new() -> Self {
        Self {
            repository_type_list: Default::default(),
            repository_format_list: Default::default(),
            write_policy_list: Default::default(),
            blob_store_type_list: Default::default(),
            user_status_list: Default::default(),
            blob_store_configuration_list: Default::default(),
            repository_configuration_list: Default::default(),
            security_user_list: Default::default(),
            security_role_list: Default::default(),
            security_privilege_list: Default::default(),
        }
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
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
        for entity in &mut self.blob_store_configuration_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.repository_configuration_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.security_user_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.security_role_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.security_privilege_list {
            entity.attach_root_recursive(root.clone());
        }
    }
}
