
// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/component
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
#[teaql(entity = "Component", table = "component_data", data_service = "postgres")]
pub struct Component {
#[teaql(id)]
    id: u64,

// @source model.xml:133
    namespace: String,

// @source model.xml:133
    name: String,

// @source model.xml:133
    version: String,

// @source model.xml:133
    normalized_version: String,

// @source model.xml:133
    kind: String,
// @source model.xml:133
#[teaql(column = "content_repository")]
    content_repository_id: u64,
// @source model.xml:133
#[teaql(relation(target = "ContentRepository", local_key = "content_repository_id", foreign_key = "id"))]
    content_repository: Option<crate::ContentRepository>,
#[teaql(relation(target = "Asset", local_key = "id", foreign_key = "component_id", many))]
    asset_list: SmartList<crate::Asset>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    root: teaql_runtime::EntityRoot,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl Component {
    pub const ENTITY_NAME: &'static str = "Component";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRoot) -> Self {
        Self {
            id: 0_u64,
            namespace: String::new(),
            name: String::new(),
            version: String::new(),
            normalized_version: String::new(),
            kind: String::new(),
            content_repository_id: 0_u64,
            content_repository: None,
            asset_list: Default::default(),
            dynamic: BTreeMap::new(),
            root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn entity_key(&self) -> teaql_runtime::EntityKey {
        teaql_runtime::EntityKey::new("Component", self.id)
    }

    pub fn attach_root_recursive(&mut self, root: teaql_runtime::EntityRoot) {
        self.root = root.clone();
        if let Some(entity) = &mut self.content_repository {
            entity.attach_root_recursive(root.clone());
        }
        for entity in &mut self.asset_list {
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

    pub fn namespace(&self) -> String {
        self.changed_namespace().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.namespace.clone())
    }

    pub fn update_namespace(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.namespace = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.namespace.clone());
        self.root.set(self.entity_key(), "namespace", value);
        self
    }

    pub fn changed_namespace(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "namespace")
    }

    pub fn eval_namespace(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("namespace") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "namespace".to_string(), attempted_path: "namespace".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.namespace())
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

    pub fn normalized_version(&self) -> String {
        self.changed_normalized_version().and_then(|value| value.try_text().map(|value| value.to_owned())).unwrap_or_else(|| self.normalized_version.clone())
    }

    pub fn update_normalized_version(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.normalized_version = value.try_text().map(|value| value.trim().to_owned()).unwrap_or_else(|| self.normalized_version.clone());
        self.root.set(self.entity_key(), "normalized_version", value);
        self
    }

    pub fn changed_normalized_version(&self) -> Option<teaql_core::Value> {
        self.root.get(&self.entity_key(), "normalized_version")
    }

    pub fn eval_normalized_version(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("normalized_version") {
                    teaql_core::eval::EvalResult::NotLoaded { failed_node: "normalized_version".to_string(), attempted_path: "normalized_version".to_string() }
                } else {
                    teaql_core::eval::EvalResult::Value(self.normalized_version())
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
    pub fn asset_list(&self) -> &SmartList<crate::Asset> {
        &self.asset_list
    }

    pub fn asset_list_mut(&mut self) -> &mut SmartList<crate::Asset> {
        &mut self.asset_list
    }

    pub fn eval_asset_list(&self) -> teaql_core::eval::EvalResult<&SmartList<crate::Asset>> {
        if !self.is_loaded("asset_list") {
            teaql_core::eval::EvalResult::NotLoaded { failed_node: "asset_list".to_string(), attempted_path: "asset_list".to_string() }
        } else {
            teaql_core::eval::EvalResult::Value(&self.asset_list)
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

