
// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/blob_store_configuration
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
#[teaql(entity = "BlobStoreConfiguration", table = "blob_store_configuration_data", data_service = "postgres")]
pub struct BlobStoreConfiguration {
#[teaql(id)]
    id: u64,

// @source model.xml:112
    name: String,

// @source model.xml:112
    path: String,

// @source model.xml:112
    total_size: i64,

// @source model.xml:112
    blob_count: i64,
#[teaql(version)]
    version: i64,
// @source model.xml:112
#[teaql(column = "tenant")]
    tenant_id: u64,

// @source model.xml:112
#[teaql(column = "blob_store_type")]
    blob_store_type_id: u64,
// @source model.xml:112
#[teaql(relation(target = "Tenant", local_key = "tenant_id", foreign_key = "id"))]
    tenant: Option<crate::Tenant>,

// @source model.xml:112
#[teaql(relation(target = "BlobStoreType", local_key = "blob_store_type_id", foreign_key = "id"))]
    blob_store_type: Option<crate::BlobStoreType>,
#[teaql(relation(target = "RepositoryConfiguration", local_key = "id", foreign_key = "blob_store_id", many))]
    repository_configuration_list: SmartList<crate::RepositoryConfiguration>,
#[teaql(relation(target = "AssetBlob", local_key = "id", foreign_key = "blob_store_id", many))]
    asset_blob_list: SmartList<crate::AssetBlob>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    root: teaql_runtime::EntityRoot,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl BlobStoreConfiguration {
    pub const ENTITY_NAME: &'static str = "Blob Store Configuration";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRoot) -> Self {
        Self {
            id: 0_u64,
            name: String::new(),
            path: String::new(),
            total_size: 0_i64,
            blob_count: 0_i64,
            version: 0_i64,
            tenant_id: 0_u64,
            blob_store_type_id: 0_u64,
            tenant: None,
            blob_store_type: None,
            repository_configuration_list: Default::default(),
            asset_blob_list: Default::default(),
            dynamic: BTreeMap::new(),
            root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn entity_key(&self) -> teaql_runtime::EntityKey {
        teaql_runtime::EntityKey::new("BlobStoreConfiguration", self.id)
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
        self.root = root.clone();
        if let Some(entity) = &mut self.tenant {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.blob_store_type {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.repository_configuration_list {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.asset_blob_list {
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

    pub fn path(&self) -> String {
        self.changed_path().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.path.clone())
    }

    pub fn update_path(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.path = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.path.clone());
        self.root.set(self.entity_key(), "path", value);
        self
    }

    pub fn changed_path(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "path")
    }

    pub fn eval_path(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("path") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "path".to_string(), attempted_path: "path".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.path())
                }}

    pub fn total_size(&self) -> i64 {
        self.changed_total_size().and_then(|value| value.try_i64()).unwrap_or(self.total_size)
    }

    pub fn update_total_size(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.total_size = value.try_i64().unwrap_or(self.total_size.clone());
        self.root.set(self.entity_key(), "total_size", value);
        self
    }

    pub fn changed_total_size(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "total_size")
    }

    pub fn eval_total_size(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("total_size") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "total_size".to_string(), attempted_path: "total_size".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.total_size())
                }}

    pub fn blob_count(&self) -> i64 {
        self.changed_blob_count().and_then(|value| value.try_i64()).map(|value| value as i64).unwrap_or(self.blob_count)
    }

    pub fn update_blob_count(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.blob_count = value.try_i64().map(|value| value as i64).unwrap_or(self.blob_count.clone());
        self.root.set(self.entity_key(), "blob_count", value);
        self
    }

    pub fn changed_blob_count(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "blob_count")
    }

    pub fn eval_blob_count(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("blob_count") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_count".to_string(), attempted_path: "blob_count".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.blob_count())
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

    pub fn blob_store_type_id(&self) -> u64 {
        self.changed_blob_store_type_id().and_then(|value| value.try_u64()).unwrap_or(self.blob_store_type_id)
    }

    pub(crate) fn update_blob_store_type_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.blob_store_type_id = value.try_u64().unwrap_or(self.blob_store_type_id.clone());
        self.root.set(self.entity_key(), "blob_store_type_id", value);
        self
    }

    pub fn changed_blob_store_type_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "blob_store_type_id")
    }

    pub fn eval_blob_store_type_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("blob_store_type_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_type_id".to_string(), attempted_path: "blob_store_type_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.blob_store_type_id())
                }}
    pub fn update_blob_store_type_to_file(&mut self) -> &mut Self {
        self.update_blob_store_type_id(1001_u64)
    }

    pub fn blob_store_type_is_file(&self) -> bool {
        self.blob_store_type_id() == 1001_u64
    }
    pub fn update_blob_store_type_to_s3(&mut self) -> &mut Self {
        self.update_blob_store_type_id(1002_u64)
    }

    pub fn blob_store_type_is_s3(&self) -> bool {
        self.blob_store_type_id() == 1002_u64
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

    pub fn blob_store_type(&self) -> Option<&crate::BlobStoreType> {
        self.blob_store_type.as_ref()
    }

    pub fn eval_blob_store_type(&self) -> teaql_core::eval::EvalResult<&crate::BlobStoreType> {
        if !self.is_loaded("blob_store_type") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_type".to_string(), attempted_path: "blob_store_type".to_string() }
        } else {
            match &self.blob_store_type {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }
    pub fn repository_configuration_list(&self) -> &SmartList<crate::RepositoryConfiguration> {
        &self.repository_configuration_list
    }

    pub fn repository_configuration_list_mut(&mut self) -> &mut SmartList<crate::RepositoryConfiguration> {
        &mut self.repository_configuration_list
    }

    pub fn eval_repository_configuration_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::RepositoryConfiguration>> {
        if !self.is_loaded("repository_configuration_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_configuration_list".to_string(), attempted_path: "repository_configuration_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.repository_configuration_list)
        }
    }

    pub fn asset_blob_list(&self) -> &SmartList<crate::AssetBlob> {
        &self.asset_blob_list
    }

    pub fn asset_blob_list_mut(&mut self) -> &mut SmartList<crate::AssetBlob> {
        &mut self.asset_blob_list
    }

    pub fn eval_asset_blob_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::AssetBlob>> {
        if !self.is_loaded("asset_blob_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "asset_blob_list".to_string(), attempted_path: "asset_blob_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.asset_blob_list)
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

