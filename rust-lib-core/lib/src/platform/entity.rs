// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/platform
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
    name = "tenant_list",
    target = "Tenant",
    local_key = "id",
    foreign_key = "platform_id",
    many
))]
#[teaql(reverse_relation(
    name = "repository_type_list",
    target = "RepositoryType",
    local_key = "id",
    foreign_key = "platform_id",
    many
))]
#[teaql(reverse_relation(
    name = "repository_format_list",
    target = "RepositoryFormat",
    local_key = "id",
    foreign_key = "platform_id",
    many
))]
#[teaql(reverse_relation(
    name = "write_policy_list",
    target = "WritePolicy",
    local_key = "id",
    foreign_key = "platform_id",
    many
))]
#[teaql(reverse_relation(
    name = "blob_store_type_list",
    target = "BlobStoreType",
    local_key = "id",
    foreign_key = "platform_id",
    many
))]
#[teaql(reverse_relation(
    name = "user_status_list",
    target = "UserStatus",
    local_key = "id",
    foreign_key = "platform_id",
    many
))]
#[teaql(
    entity = "Platform",
    table = "platform_data",
    data_service = "postgres"
)]
pub struct Platform {
    #[teaql(id)]
    id: u64,

    // @source model.xml:12
    name: String,

    // @source model.xml:12
    platform_version: String,
    #[teaql(version)]
    version: i64,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl Platform {
    pub const ENTITY_NAME: &'static str = "TeaQL Registry Platform";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            name: String::new(),
            platform_version: String::new(),
            version: 0_i64,
            dynamic: BTreeMap::new(),
            __teaql_runtime_state: root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn attach_runtime_state_recursive(&mut self, root: teaql_runtime::EntityRuntimeState) {
        root.adopt_mutations_from(self.__teaql_runtime_state());
        self.__teaql_replace_runtime_state(root.clone());
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

    pub fn name(&self) -> String {
        self.changed_name()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.name.clone())
    }

    pub fn update_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.name = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.name.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "name", value);
        self
    }

    pub fn changed_name(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state().get(&self.entity_key(), "name")
    }

    pub fn eval_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("name") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "name".to_string(),
                attempted_path: "name".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.name())
        }
    }

    pub fn platform_version(&self) -> String {
        self.changed_platform_version()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.platform_version.clone())
    }

    pub fn update_platform_version(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.platform_version = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.platform_version.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "platform_version", value);
        self
    }

    pub fn changed_platform_version(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "platform_version")
    }

    pub fn eval_platform_version(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("platform_version") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "platform_version".to_string(),
                attempted_path: "platform_version".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.platform_version())
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
    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn tenant_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::Tenant>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "tenant_list",
        )
    }

    pub fn eval_tenant_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::Tenant>> {
        let relation = self.tenant_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "tenant_list".to_string(),
                attempted_path: "tenant_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn repository_type_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::RepositoryType>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "repository_type_list",
        )
    }

    pub fn eval_repository_type_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::RepositoryType>> {
        let relation = self.repository_type_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "repository_type_list".to_string(),
                attempted_path: "repository_type_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn repository_format_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::RepositoryFormat>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "repository_format_list",
        )
    }

    pub fn eval_repository_format_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::RepositoryFormat>> {
        let relation = self.repository_format_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "repository_format_list".to_string(),
                attempted_path: "repository_format_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn write_policy_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::WritePolicy>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "write_policy_list",
        )
    }

    pub fn eval_write_policy_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::WritePolicy>> {
        let relation = self.write_policy_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "write_policy_list".to_string(),
                attempted_path: "write_policy_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn blob_store_type_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::BlobStoreType>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "blob_store_type_list",
        )
    }

    pub fn eval_blob_store_type_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::BlobStoreType>> {
        let relation = self.blob_store_type_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "blob_store_type_list".to_string(),
                attempted_path: "blob_store_type_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn user_status_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::UserStatus>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "user_status_list",
        )
    }

    pub fn eval_user_status_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::UserStatus>> {
        let relation = self.user_status_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "user_status_list".to_string(),
                attempted_path: "user_status_list".to_string(),
            },
        }
    }
}
