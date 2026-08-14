
// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/asset
use std::collections::BTreeMap;

use teaql_macros::TeaqlEntity;

/// [TEAQL AI WARNING]
/// TeaQL was explicitly designed to PREVENT AI hallucinations and random guessing.
/// DO NOT GUESS METHOD NAMES!
/// The methods listed below are the ONLY valid ways to interact with this entity.
/// If you encounter compilation errors (e.g., method not found), DO NOT guess another method name.
/// Read the method signatures in this file before proceeding.
#[derive(Clone, Debug, PartialEq, TeaqlEntity)]
#[teaql(entity = "Asset", table = "asset_data", data_service = "postgres")]
pub struct Asset {
#[teaql(id)]
    id: u64,

// @source model.xml:155
    path: String,

// @source model.xml:155
    kind: String,
#[teaql(version)]
    version: i64,
// @source model.xml:155
#[teaql(column = "content_repository")]
    content_repository_id: u64,

// @source model.xml:155
#[teaql(column = "component")]
    component_id: u64,

// @source model.xml:155
#[teaql(column = "asset_blob")]
    asset_blob_id: u64,
// @source model.xml:155
#[teaql(relation(target = "ContentRepository", local_key = "content_repository_id", foreign_key = "id"))]
    content_repository: Option<crate::ContentRepository>,

// @source model.xml:155
#[teaql(relation(target = "Component", local_key = "component_id", foreign_key = "id"))]
    component: Option<crate::Component>,

// @source model.xml:155
#[teaql(relation(target = "AssetBlob", local_key = "asset_blob_id", foreign_key = "id"))]
    asset_blob: Option<crate::AssetBlob>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    root: teaql_runtime::EntityRoot,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl Asset {
    pub const ENTITY_NAME: &'static str = "Asset";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRoot) -> Self {
        Self {
            id: 0_u64,
            path: String::new(),
            kind: String::new(),
            version: 0_i64,
            content_repository_id: 0_u64,
            component_id: 0_u64,
            asset_blob_id: 0_u64,
            content_repository: None,
            component: None,
            asset_blob: None,
            dynamic: BTreeMap::new(),
            root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn entity_key(&self) -> teaql_runtime::EntityKey {
        teaql_runtime::EntityKey::new("Asset", self.id)
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
        self.root = root.clone();
        if let Some(entity) = &mut self.content_repository {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.component {
            entity.attach_root_recursive(root.clone());
        }
        if let Some(entity) = &mut self.asset_blob {
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

    pub fn kind(&self) -> String {
        self.changed_kind().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.kind.clone())
    }

    pub fn update_kind(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.kind = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.kind.clone());
        self.root.set(self.entity_key(), "kind", value);
        self
    }

    pub fn changed_kind(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "kind")
    }

    pub fn eval_kind(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("kind") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "kind".to_string(), attempted_path: "kind".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.kind())
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
    pub fn content_repository_id(&self) -> u64 {
        self.changed_content_repository_id().and_then(|value| value.try_u64()).unwrap_or(self.content_repository_id)
    }

    pub fn update_content_repository_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.content_repository_id = value.try_u64().unwrap_or(self.content_repository_id.clone());
        self.root.set(self.entity_key(), "content_repository_id", value);
        self
    }

    pub fn changed_content_repository_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "content_repository_id")
    }

    pub fn eval_content_repository_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("content_repository_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "content_repository_id".to_string(), attempted_path: "content_repository_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.content_repository_id())
                }}

    pub fn component_id(&self) -> u64 {
        self.changed_component_id().and_then(|value| value.try_u64()).unwrap_or(self.component_id)
    }

    pub fn update_component_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.component_id = value.try_u64().unwrap_or(self.component_id.clone());
        self.root.set(self.entity_key(), "component_id", value);
        self
    }

    pub fn changed_component_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "component_id")
    }

    pub fn eval_component_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("component_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "component_id".to_string(), attempted_path: "component_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.component_id())
                }}

    pub fn asset_blob_id(&self) -> u64 {
        self.changed_asset_blob_id().and_then(|value| value.try_u64()).unwrap_or(self.asset_blob_id)
    }

    pub fn update_asset_blob_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.asset_blob_id = value.try_u64().unwrap_or(self.asset_blob_id.clone());
        self.root.set(self.entity_key(), "asset_blob_id", value);
        self
    }

    pub fn changed_asset_blob_id(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "asset_blob_id")
    }

    pub fn eval_asset_blob_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("asset_blob_id") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "asset_blob_id".to_string(), attempted_path: "asset_blob_id".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.asset_blob_id())
                }}
    pub fn content_repository(&self) -> Option<&crate::ContentRepository> {
        self.content_repository.as_ref()
    }

    pub fn eval_content_repository(&self) -> teaql_core::eval::EvalResult<&crate::ContentRepository> {
        if !self.is_loaded("content_repository") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "content_repository".to_string(), attempted_path: "content_repository".to_string() }
        } else {
            match &self.content_repository {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }

    pub fn component(&self) -> Option<&crate::Component> {
        self.component.as_ref()
    }

    pub fn eval_component(&self) -> teaql_core::eval::EvalResult<&crate::Component> {
        if !self.is_loaded("component") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "component".to_string(), attempted_path: "component".to_string() }
        } else {
            match &self.component {
                Some(v) => teaql_core::eval::EvalResult::Value(v),
                None => teaql_core::eval::EvalResult::Null,
            }
        }
    }

    pub fn asset_blob(&self) -> Option<&crate::AssetBlob> {
        self.asset_blob.as_ref()
    }

    pub fn eval_asset_blob(&self) -> teaql_core::eval::EvalResult<&crate::AssetBlob> {
        if !self.is_loaded("asset_blob") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "asset_blob".to_string(), attempted_path: "asset_blob".to_string() }
        } else {
            match &self.asset_blob {
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

