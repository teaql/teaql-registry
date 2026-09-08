// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/service_log
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
    entity = "ServiceLog",
    table = "service_log_data",
    data_service = "postgres"
)]
pub struct ServiceLog {
    #[teaql(id)]
    id: u64,

    // @source model.xml:238
    event_time: teaql_core::time::Timestamp,

    // @source model.xml:238
    log_type: String,

    // @source model.xml:238
    operator_id: i64,

    // @source model.xml:238
    operator_name: String,

    // @source model.xml:238
    client_ip: String,

    // @source model.xml:238
    action: String,

    // @source model.xml:238
    repository_name: String,

    // @source model.xml:238
    artifact_path: String,

    // @source model.xml:238
    format_name: String,

    // @source model.xml:238
    content_size: i64,

    // @source model.xml:238
    status: String,

    // @source model.xml:238
    error_message: String,
    #[teaql(version)]
    version: i64,
    // @source model.xml:238
    #[teaql(column = "tenant")]
    tenant_id: u64,
    // @source model.xml:238
    #[teaql(relation(target = "Tenant", local_key = "tenant_id", foreign_key = "id"))]
    tenant: Option<Box<crate::Tenant>>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl ServiceLog {
    pub const ENTITY_NAME: &'static str = "Service Log";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            event_time: teaql_core::time::Timestamp::now(),
            log_type: String::new(),
            operator_id: 0_i64,
            operator_name: String::new(),
            client_ip: String::new(),
            action: String::new(),
            repository_name: String::new(),
            artifact_path: String::new(),
            format_name: String::new(),
            content_size: 0_i64,
            status: String::new(),
            error_message: String::new(),
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

    pub fn event_time(&self) -> teaql_core::time::Timestamp {
        self.changed_event_time()
            .and_then(|value| value.try_timestamp())
            .unwrap_or(self.event_time)
    }

    pub fn update_event_time(&mut self, value: teaql_core::time::Timestamp) -> &mut Self {
        self.event_time = value;
        let value = teaql_core::Value::from(value);
        self.__teaql_runtime_state()
            .set(self.entity_key(), "event_time", value);
        self
    }
    pub fn changed_event_time(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "event_time")
    }

    pub fn eval_event_time(&self) -> teaql_core::eval::EvalResult<teaql_core::time::Timestamp> {
        if !self.is_loaded("event_time") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "event_time".to_string(),
                attempted_path: "event_time".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.event_time())
        }
    }

    pub fn log_type(&self) -> String {
        self.changed_log_type()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.log_type.clone())
    }

    pub fn update_log_type(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.log_type = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.log_type.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "log_type", value);
        self
    }

    pub fn changed_log_type(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "log_type")
    }

    pub fn eval_log_type(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("log_type") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "log_type".to_string(),
                attempted_path: "log_type".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.log_type())
        }
    }

    pub fn operator_id(&self) -> i64 {
        self.changed_operator_id()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.operator_id)
    }

    pub fn update_operator_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.operator_id = value.try_i64().unwrap_or(self.operator_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "operator_id", value);
        self
    }

    pub fn changed_operator_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "operator_id")
    }

    pub fn eval_operator_id(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("operator_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "operator_id".to_string(),
                attempted_path: "operator_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.operator_id())
        }
    }

    pub fn operator_name(&self) -> String {
        self.changed_operator_name()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.operator_name.clone())
    }

    pub fn update_operator_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.operator_name = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.operator_name.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "operator_name", value);
        self
    }

    pub fn changed_operator_name(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "operator_name")
    }

    pub fn eval_operator_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("operator_name") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "operator_name".to_string(),
                attempted_path: "operator_name".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.operator_name())
        }
    }

    pub fn client_ip(&self) -> String {
        self.changed_client_ip()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.client_ip.clone())
    }

    pub fn update_client_ip(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.client_ip = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.client_ip.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "client_ip", value);
        self
    }

    pub fn changed_client_ip(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "client_ip")
    }

    pub fn eval_client_ip(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("client_ip") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "client_ip".to_string(),
                attempted_path: "client_ip".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.client_ip())
        }
    }

    pub fn action(&self) -> String {
        self.changed_action()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.action.clone())
    }

    pub fn update_action(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.action = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.action.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "action", value);
        self
    }

    pub fn changed_action(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "action")
    }

    pub fn eval_action(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("action") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "action".to_string(),
                attempted_path: "action".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.action())
        }
    }

    pub fn repository_name(&self) -> String {
        self.changed_repository_name()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.repository_name.clone())
    }

    pub fn update_repository_name(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.repository_name = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.repository_name.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "repository_name", value);
        self
    }

    pub fn changed_repository_name(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "repository_name")
    }

    pub fn eval_repository_name(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("repository_name") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "repository_name".to_string(),
                attempted_path: "repository_name".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.repository_name())
        }
    }

    pub fn artifact_path(&self) -> String {
        self.changed_artifact_path()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.artifact_path.clone())
    }

    pub fn update_artifact_path(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.artifact_path = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.artifact_path.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "artifact_path", value);
        self
    }

    pub fn changed_artifact_path(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "artifact_path")
    }

    pub fn eval_artifact_path(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("artifact_path") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "artifact_path".to_string(),
                attempted_path: "artifact_path".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.artifact_path())
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

    pub fn content_size(&self) -> i64 {
        self.changed_content_size()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.content_size)
    }

    pub fn update_content_size(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.content_size = value.try_i64().unwrap_or(self.content_size.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "content_size", value);
        self
    }

    pub fn changed_content_size(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "content_size")
    }

    pub fn eval_content_size(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("content_size") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "content_size".to_string(),
                attempted_path: "content_size".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.content_size())
        }
    }

    pub fn status(&self) -> String {
        self.changed_status()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.status.clone())
    }

    pub fn update_status(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.status = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.status.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "status", value);
        self
    }

    pub fn changed_status(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "status")
    }

    pub fn eval_status(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("status") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "status".to_string(),
                attempted_path: "status".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.status())
        }
    }

    pub fn error_message(&self) -> String {
        self.changed_error_message()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.error_message.clone())
    }

    pub fn update_error_message(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.error_message = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.error_message.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "error_message", value);
        self
    }

    pub fn changed_error_message(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "error_message")
    }

    pub fn eval_error_message(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("error_message") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "error_message".to_string(),
                attempted_path: "error_message".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.error_message())
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
}
