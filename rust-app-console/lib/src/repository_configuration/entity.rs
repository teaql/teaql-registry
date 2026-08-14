
// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/repository_configuration
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
#[teaql(entity = "RepositoryConfiguration", table = "repository_configuration_data", data_service = "postgres")]
pub struct RepositoryConfiguration {
#[teaql(id)]
    id: u64,

// @source model.xml:115
    name: String,

// @source model.xml:115
    recipe_name: String,

// @source model.xml:115
    online: bool,

// @source model.xml:115
    remote_url: String,
#[teaql(version)]
    version: i64,
// @source model.xml:115
#[teaql(column = "platform")]
    platform_id: u64,

// @source model.xml:115
#[teaql(column = "repository_type")]
    repository_type_id: u64,

// @source model.xml:115
#[teaql(column = "repository_format")]
    repository_format_id: u64,

// @source model.xml:115
#[teaql(column = "write_policy")]
    write_policy_id: u64,

// @source model.xml:115
#[teaql(column = "blob_store")]
    blob_store_id: u64,
// @source model.xml:115
#[teaql(relation(target = "Platform", local_key = "platform_id", foreign_key = "id"))]
    platform: Option<crate::Platform>,

// @source model.xml:115
#[teaql(relation(target = "RepositoryType", local_key = "repository_type_id", foreign_key = "id"))]
    repository_type: Option<crate::RepositoryType>,

// @source model.xml:115
#[teaql(relation(target = "RepositoryFormat", local_key = "repository_format_id", foreign_key = "id"))]
    repository_format: Option<crate::RepositoryFormat>,

// @source model.xml:115
#[teaql(relation(target = "WritePolicy", local_key = "write_policy_id", foreign_key = "id"))]
    write_policy: Option<crate::WritePolicy>,

// @source model.xml:115
#[teaql(relation(target = "BlobStoreConfiguration", local_key = "blob_store_id", foreign_key = "id"))]
    blob_store: Option<crate::BlobStoreConfiguration>,
#[teaql(relation(target = "ContentRepository", local_key = "id", foreign_key = "repository_id", many))]
    content_repository_list: SmartList<crate::ContentRepository>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    root: teaql_runtime::EntityRoot,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl RepositoryConfiguration {
    pub const ENTITY_NAME: &'static str = "Repository Configuration";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRoot) -> Self {
        Self {
            id: 0_u64,
            name: String::new(),
            recipe_name: String::new(),
            online: false,
            remote_url: String::new(),
            version: 0_i64,
            platform_id: 0_u64,
            repository_type_id: 0_u64,
            repository_format_id: 0_u64,
            write_policy_id: 0_u64,
            blob_store_id: 0_u64,
            platform: None,
            repository_type: None,
            repository_format: None,
            write_policy: None,
            blob_store: None,
            content_repository_list: Default::default(),
            dynamic: BTreeMap::new(),
            root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn entity_key(&self) -> teaql_runtime::EntityKey {
        teaql_runtime::EntityKey::new("RepositoryConfiguration", self.id)
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
        self.root = root.clone();
        if let Some(entity) = &mut self.platform {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.repository_type {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.repository_format {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.write_policy {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.blob_store {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.content_repository_list {
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

    pub fn recipe_name(&self) -> String {
        self.changed_recipe_name().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.recipe_name.clone())
    }

    pub fn update_recipe_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.recipe_name = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.recipe_name.clone());
        self.root.set(self.entity_key(), "recipe_name", value);
        self
    }

    pub fn changed_recipe_name(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "recipe_name")
    }

    pub fn eval_recipe_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("recipe_name") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "recipe_name".to_string(), attempted_path: "recipe_name".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.recipe_name())
                }}

    pub fn online(&self) -> bool {
        self.changed_online().and_then(|value| value.try_bool()).unwrap_or(self.online)
    }

    pub fn update_online(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.online = value.try_bool().unwrap_or(self.online.clone());
        self.root.set(self.entity_key(), "online", value);
        self
    }

    pub fn changed_online(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "online")
    }

    pub fn eval_online(&self) -> teaql_core::eval::EvalResult<bool> {
        if !self.is_loaded("online") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "online".to_string(), attempted_path: "online".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.online())
                }}

    pub fn remote_url(&self) -> String {
        self.changed_remote_url().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.remote_url.clone())
    }

    pub fn update_remote_url(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.remote_url = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.remote_url.clone());
        self.root.set(self.entity_key(), "remote_url", value);
        self
    }

    pub fn changed_remote_url(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "remote_url")
    }

    pub fn eval_remote_url(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("remote_url") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "remote_url".to_string(), attempted_path: "remote_url".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.remote_url())
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

    pub fn repository_type_id(&self) -> u64 {
        self.changed_repository_type_id().and_then(|value| value.try_u64()).unwrap_or(self.repository_type_id)
    }

    pub(crate) fn update_repository_type_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.repository_type_id = value.try_u64().unwrap_or(self.repository_type_id.clone());
        self.root.set(self.entity_key(), "repository_type_id", value);
        self
    }

    pub fn changed_repository_type_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "repository_type_id")
    }

    pub fn eval_repository_type_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("repository_type_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_type_id".to_string(), attempted_path: "repository_type_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.repository_type_id())
                }}

    pub fn repository_format_id(&self) -> u64 {
        self.changed_repository_format_id().and_then(|value| value.try_u64()).unwrap_or(self.repository_format_id)
    }

    pub(crate) fn update_repository_format_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.repository_format_id = value.try_u64().unwrap_or(self.repository_format_id.clone());
        self.root.set(self.entity_key(), "repository_format_id", value);
        self
    }

    pub fn changed_repository_format_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "repository_format_id")
    }

    pub fn eval_repository_format_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("repository_format_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_format_id".to_string(), attempted_path: "repository_format_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.repository_format_id())
                }}

    pub fn write_policy_id(&self) -> u64 {
        self.changed_write_policy_id().and_then(|value| value.try_u64()).unwrap_or(self.write_policy_id)
    }

    pub(crate) fn update_write_policy_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.write_policy_id = value.try_u64().unwrap_or(self.write_policy_id.clone());
        self.root.set(self.entity_key(), "write_policy_id", value);
        self
    }

    pub fn changed_write_policy_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "write_policy_id")
    }

    pub fn eval_write_policy_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("write_policy_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "write_policy_id".to_string(), attempted_path: "write_policy_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.write_policy_id())
                }}

    pub fn blob_store_id(&self) -> u64 {
        self.changed_blob_store_id().and_then(|value| value.try_u64()).unwrap_or(self.blob_store_id)
    }

    pub fn update_blob_store_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.blob_store_id = value.try_u64().unwrap_or(self.blob_store_id.clone());
        self.root.set(self.entity_key(), "blob_store_id", value);
        self
    }

    pub fn changed_blob_store_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "blob_store_id")
    }

    pub fn eval_blob_store_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("blob_store_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_id".to_string(), attempted_path: "blob_store_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.blob_store_id())
                }}
    pub fn update_repository_type_to_hosted(&mut self) -> &mut Self {
        self.update_repository_type_id(1001_u64)
    }

    pub fn repository_type_is_hosted(&self) -> bool {
        self.repository_type_id() == 1001_u64
    }
    pub fn update_repository_type_to_proxy(&mut self) -> &mut Self {
        self.update_repository_type_id(1002_u64)
    }

    pub fn repository_type_is_proxy(&self) -> bool {
        self.repository_type_id() == 1002_u64
    }
    pub fn update_repository_type_to_group(&mut self) -> &mut Self {
        self.update_repository_type_id(1003_u64)
    }

    pub fn repository_type_is_group(&self) -> bool {
        self.repository_type_id() == 1003_u64
    }

    pub fn update_repository_format_to_maven2(&mut self) -> &mut Self {
        self.update_repository_format_id(1001_u64)
    }

    pub fn repository_format_is_maven2(&self) -> bool {
        self.repository_format_id() == 1001_u64
    }
    pub fn update_repository_format_to_raw(&mut self) -> &mut Self {
        self.update_repository_format_id(1002_u64)
    }

    pub fn repository_format_is_raw(&self) -> bool {
        self.repository_format_id() == 1002_u64
    }

    pub fn update_write_policy_to_allow_write(&mut self) -> &mut Self {
        self.update_write_policy_id(1001_u64)
    }

    pub fn write_policy_is_allow_write(&self) -> bool {
        self.write_policy_id() == 1001_u64
    }
    pub fn update_write_policy_to_allow_once(&mut self) -> &mut Self {
        self.update_write_policy_id(1002_u64)
    }

    pub fn write_policy_is_allow_once(&self) -> bool {
        self.write_policy_id() == 1002_u64
    }
    pub fn update_write_policy_to_read_only(&mut self) -> &mut Self {
        self.update_write_policy_id(1003_u64)
    }

    pub fn write_policy_is_read_only(&self) -> bool {
        self.write_policy_id() == 1003_u64
    }
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

    pub fn repository_type(&self) -> Option<&crate::RepositoryType> {
        self.repository_type.as_ref()
    }

    pub fn eval_repository_type(&self) -> teaql_core::eval::EvalResult<&crate::RepositoryType> {
        if !self.is_loaded("repository_type") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_type".to_string(), attempted_path: "repository_type".to_string() }
        } else {
            match &self.repository_type {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }

    pub fn repository_format(&self) -> Option<&crate::RepositoryFormat> {
        self.repository_format.as_ref()
    }

    pub fn eval_repository_format(&self) -> teaql_core::eval::EvalResult<&crate::RepositoryFormat> {
        if !self.is_loaded("repository_format") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_format".to_string(), attempted_path: "repository_format".to_string() }
        } else {
            match &self.repository_format {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }

    pub fn write_policy(&self) -> Option<&crate::WritePolicy> {
        self.write_policy.as_ref()
    }

    pub fn eval_write_policy(&self) -> teaql_core::eval::EvalResult<&crate::WritePolicy> {
        if !self.is_loaded("write_policy") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "write_policy".to_string(), attempted_path: "write_policy".to_string() }
        } else {
            match &self.write_policy {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }

    pub fn blob_store(&self) -> Option<&crate::BlobStoreConfiguration> {
        self.blob_store.as_ref()
    }

    pub fn eval_blob_store(&self) -> teaql_core::eval::EvalResult<&crate::BlobStoreConfiguration> {
        if !self.is_loaded("blob_store") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store".to_string(), attempted_path: "blob_store".to_string() }
        } else {
            match &self.blob_store {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }
    pub fn content_repository_list(&self) -> &SmartList<crate::ContentRepository> {
        &self.content_repository_list
    }

    pub fn content_repository_list_mut(&mut self) -> &mut SmartList<crate::ContentRepository> {
        &mut self.content_repository_list
    }

    pub fn eval_content_repository_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::ContentRepository>> {
        if !self.is_loaded("content_repository_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "content_repository_list".to_string(), attempted_path: "content_repository_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.content_repository_list)
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

