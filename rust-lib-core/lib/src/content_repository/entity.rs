// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/content_repository
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
    name = "component_list",
    target = "Component",
    local_key = "id",
    foreign_key = "content_repository_id",
    many
))]
#[teaql(reverse_relation(
    name = "asset_list",
    target = "Asset",
    local_key = "id",
    foreign_key = "content_repository_id",
    many
))]
#[teaql(
    entity = "ContentRepository",
    table = "content_repository_data",
    data_service = "postgres"
)]
pub struct ContentRepository {
    #[teaql(id)]
    id: u64,

    // @source model.xml:135
    repository_id: i64,

    // @source model.xml:135
    format_name: String,
    #[teaql(version)]
    version: i64,
    // @source model.xml:135
    #[teaql(column = "tenant")]
    tenant_id: u64,
    // @source model.xml:135
    #[teaql(relation(target = "Tenant", local_key = "tenant_id", foreign_key = "id"))]
    tenant: Option<Box<crate::Tenant>>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl ContentRepository {
    pub const ENTITY_NAME: &'static str = "Content Repository";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            repository_id: 0_i64,
            format_name: String::new(),
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

    pub fn repository_id(&self) -> i64 {
        self.changed_repository_id()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.repository_id)
    }

    pub fn update_repository_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.repository_id = value.try_i64().unwrap_or(self.repository_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "repository_id", value);
        self
    }

    pub fn changed_repository_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "repository_id")
    }

    pub fn eval_repository_id(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("repository_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "repository_id".to_string(),
                attempted_path: "repository_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.repository_id())
        }
    }

    pub fn format_name(&self) -> String {
        self.changed_format_name()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.format_name.clone())
    }

    pub fn update_format_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.format_name = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.format_name.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "format_name", value);
        self
    }

    pub fn changed_format_name(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "format_name")
    }

    pub fn eval_format_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("format_name") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "format_name".to_string(),
                attempted_path: "format_name".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.format_name())
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
    pub fn component_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::Component>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "component_list",
        )
    }

    pub fn eval_component_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::Component>> {
        let relation = self.component_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "component_list".to_string(),
                attempted_path: "component_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn asset_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::Asset>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "asset_list",
        )
    }

    pub fn eval_asset_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::Asset>> {
        let relation = self.asset_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "asset_list".to_string(),
                attempted_path: "asset_list".to_string(),
            },
        }
    }
}
