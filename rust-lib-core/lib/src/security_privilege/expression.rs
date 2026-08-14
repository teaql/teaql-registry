#[derive(Clone)]
pub struct SecurityPrivilegeExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::SecurityPrivilege>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityPrivilegeExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::SecurityPrivilege>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::SecurityPrivilege> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::SecurityPrivilege> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::SecurityPrivilege {
        self.resolve().expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_privilege_id(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("privilege_id", |entity| entity.eval_privilege_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_name(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("name", |entity| entity.eval_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_description(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("description", |entity| entity.eval_description());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_privilege_type(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("privilege_type", |entity| entity.eval_privilege_type());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_permission_pattern(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("permission_pattern", |entity| entity.eval_permission_pattern());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_read_only(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("read_only", |entity| entity.eval_read_only());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_version(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("version", |entity| entity.eval_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_tenant_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("tenant_id", |entity| entity.eval_tenant_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_tenant(self) -> crate::TenantExpression<'a> {
        let next = self.result.and_then("tenant", |entity| entity.eval_tenant());
        crate::TenantExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct SecurityPrivilegeListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::SecurityPrivilege>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityPrivilegeListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::SecurityPrivilege>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityPrivilege>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityPrivilege>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::SecurityPrivilege> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::SecurityPrivilegeExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityPrivilegeExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::SecurityPrivilegeExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityPrivilegeExpression::new(next, self.root_desc.clone())
    }
}