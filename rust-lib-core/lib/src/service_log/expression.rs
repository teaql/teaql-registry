#[derive(Clone)]
pub struct ServiceLogExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::ServiceLog>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> ServiceLogExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<&'a crate::ServiceLog>,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::ServiceLog> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a crate::ServiceLog> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::ServiceLog {
        self.resolve()
            .expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_event_time(self) -> crate::ValueExpression<'a, teaql_core::time::Timestamp> {
        let next = self
            .result
            .and_then("event_time", |entity| entity.eval_event_time());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_log_type(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("log_type", |entity| entity.eval_log_type());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_operator_id(self) -> crate::ValueExpression<'a, i64> {
        let next = self
            .result
            .and_then("operator_id", |entity| entity.eval_operator_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_operator_name(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("operator_name", |entity| entity.eval_operator_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_client_ip(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("client_ip", |entity| entity.eval_client_ip());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_action(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("action", |entity| entity.eval_action());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_repository_name(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("repository_name", |entity| entity.eval_repository_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_artifact_path(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("artifact_path", |entity| entity.eval_artifact_path());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_format_name(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("format_name", |entity| entity.eval_format_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_content_size(self) -> crate::ValueExpression<'a, i64> {
        let next = self
            .result
            .and_then("content_size", |entity| entity.eval_content_size());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_status(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("status", |entity| entity.eval_status());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_error_message(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("error_message", |entity| entity.eval_error_message());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_version(self) -> crate::ValueExpression<'a, i64> {
        let next = self
            .result
            .and_then("version", |entity| entity.eval_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_tenant_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self
            .result
            .and_then("tenant_id", |entity| entity.eval_tenant_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_tenant(self) -> crate::TenantExpression<'a> {
        let next = self
            .result
            .and_then("tenant", |entity| entity.eval_tenant());
        crate::TenantExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct ServiceLogListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::ServiceLog>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> ServiceLogListExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::ServiceLog>>,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::ServiceLog>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::ServiceLog>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::ServiceLog> {
        self.resolve()
            .expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| {
            teaql_core::eval::EvalResult::Value(list.len())
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::ServiceLogExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::ServiceLogExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::ServiceLogExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::ServiceLogExpression::new(next, self.root_desc.clone())
    }
}
