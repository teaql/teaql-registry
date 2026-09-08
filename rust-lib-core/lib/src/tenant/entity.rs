// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/tenant
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
    name = "blob_store_configuration_list",
    target = "BlobStoreConfiguration",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "repository_configuration_list",
    target = "RepositoryConfiguration",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "content_repository_list",
    target = "ContentRepository",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "security_user_list",
    target = "SecurityUser",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "security_role_list",
    target = "SecurityRole",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "security_privilege_list",
    target = "SecurityPrivilege",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "security_user_role_list",
    target = "SecurityUserRole",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "security_role_privilege_list",
    target = "SecurityRolePrivilege",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "personal_access_token_list",
    target = "PersonalAccessToken",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(reverse_relation(
    name = "service_log_list",
    target = "ServiceLog",
    local_key = "id",
    foreign_key = "tenant_id",
    many
))]
#[teaql(entity = "Tenant", table = "tenant_data", data_service = "postgres")]
pub struct Tenant {
    #[teaql(id)]
    id: u64,

    // @source model.xml:22
    name: String,

    // @source model.xml:22
    code: String,

    // @source model.xml:22
    description: String,

    // @source model.xml:22
    enabled: bool,
    #[teaql(version)]
    version: i64,
    // @source model.xml:22
    #[teaql(column = "platform")]
    platform_id: u64,
    // @source model.xml:22
    #[teaql(relation(target = "Platform", local_key = "platform_id", foreign_key = "id"))]
    platform: Option<Box<crate::Platform>>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl Tenant {
    pub const ENTITY_NAME: &'static str = "Tenant";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            name: String::new(),
            code: String::new(),
            description: String::new(),
            enabled: false,
            version: 0_i64,
            platform_id: 0_u64,
            platform: None,
            dynamic: BTreeMap::new(),
            __teaql_runtime_state: root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn attach_runtime_state_recursive(&mut self, root: teaql_runtime::EntityRuntimeState) {
        root.adopt_mutations_from(self.__teaql_runtime_state());
        self.__teaql_replace_runtime_state(root.clone());
        if let Some(entity) = &mut self.platform {
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

    pub fn code(&self) -> String {
        self.changed_code()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.code.clone())
    }

    pub fn update_code(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.code = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.code.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "code", value);
        self
    }

    pub fn changed_code(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state().get(&self.entity_key(), "code")
    }

    pub fn eval_code(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("code") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "code".to_string(),
                attempted_path: "code".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.code())
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

    pub fn enabled(&self) -> bool {
        self.changed_enabled()
            .and_then(|value| value.try_bool())
            .unwrap_or(self.enabled)
    }

    pub fn update_enabled(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.enabled = value.try_bool().unwrap_or(self.enabled.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "enabled", value);
        self
    }

    pub fn changed_enabled(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "enabled")
    }

    pub fn eval_enabled(&self) -> teaql_core::eval::EvalResult<bool> {
        if !self.is_loaded("enabled") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "enabled".to_string(),
                attempted_path: "enabled".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.enabled())
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
    pub fn platform_id(&self) -> u64 {
        self.changed_platform_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.platform_id)
    }

    pub fn update_platform_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.platform_id = value.try_u64().unwrap_or(self.platform_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "platform_id", value);
        self
    }

    pub fn changed_platform_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "platform_id")
    }

    pub fn eval_platform_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("platform_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "platform_id".to_string(),
                attempted_path: "platform_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.platform_id())
        }
    }
    pub fn platform(&self) -> Option<&crate::Platform> {
        self.platform.as_deref().or_else(|| {
            self.__teaql_runtime_state()
                .resolve_entity(self.platform_id())
        })
    }

    pub fn eval_platform(&self) -> teaql_core::eval::EvalResult<&crate::Platform> {
        match self.platform() {
            Some(v) => teaql_core::eval::EvalResult::Value(v),
            None if self.is_loaded("platform") => teaql_core::eval::EvalResult::Null,
            None => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "platform".to_string(),
                attempted_path: "platform".to_string(),
            },
        }
    }
    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn blob_store_configuration_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::BlobStoreConfiguration>>
    {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "blob_store_configuration_list",
        )
    }

    pub fn eval_blob_store_configuration_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::BlobStoreConfiguration>> {
        let relation = self.blob_store_configuration_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "blob_store_configuration_list".to_string(),
                attempted_path: "blob_store_configuration_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn repository_configuration_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::RepositoryConfiguration>>
    {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "repository_configuration_list",
        )
    }

    pub fn eval_repository_configuration_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::RepositoryConfiguration>> {
        let relation = self.repository_configuration_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "repository_configuration_list".to_string(),
                attempted_path: "repository_configuration_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn content_repository_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::ContentRepository>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "content_repository_list",
        )
    }

    pub fn eval_content_repository_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::ContentRepository>> {
        let relation = self.content_repository_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "content_repository_list".to_string(),
                attempted_path: "content_repository_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn security_user_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::SecurityUser>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "security_user_list",
        )
    }

    pub fn eval_security_user_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::SecurityUser>> {
        let relation = self.security_user_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_user_list".to_string(),
                attempted_path: "security_user_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn security_role_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::SecurityRole>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "security_role_list",
        )
    }

    pub fn eval_security_role_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::SecurityRole>> {
        let relation = self.security_role_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_role_list".to_string(),
                attempted_path: "security_role_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn security_privilege_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::SecurityPrivilege>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "security_privilege_list",
        )
    }

    pub fn eval_security_privilege_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::SecurityPrivilege>> {
        let relation = self.security_privilege_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_privilege_list".to_string(),
                attempted_path: "security_privilege_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn security_user_role_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::SecurityUserRole>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "security_user_role_list",
        )
    }

    pub fn eval_security_user_role_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::SecurityUserRole>> {
        let relation = self.security_user_role_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_user_role_list".to_string(),
                attempted_path: "security_user_role_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn security_role_privilege_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::SecurityRolePrivilege>>
    {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "security_role_privilege_list",
        )
    }

    pub fn eval_security_role_privilege_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::SecurityRolePrivilege>> {
        let relation = self.security_role_privilege_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "security_role_privilege_list".to_string(),
                attempted_path: "security_role_privilege_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn personal_access_token_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::PersonalAccessToken>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "personal_access_token_list",
        )
    }

    pub fn eval_personal_access_token_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::PersonalAccessToken>> {
        let relation = self.personal_access_token_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "personal_access_token_list".to_string(),
                attempted_path: "personal_access_token_list".to_string(),
            },
        }
    }

    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn service_log_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::ServiceLog>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "service_log_list",
        )
    }

    pub fn eval_service_log_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::ServiceLog>> {
        let relation = self.service_log_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "service_log_list".to_string(),
                attempted_path: "service_log_list".to_string(),
            },
        }
    }
}
