#[derive(Clone)]
pub struct SecurityUserRoleExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::SecurityUserRole>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityUserRoleExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<&'a crate::SecurityUserRole>,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::SecurityUserRole> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a crate::SecurityUserRole> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::SecurityUserRole {
        self.resolve()
            .expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
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

    pub fn get_security_user_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self
            .result
            .and_then("security_user_id", |entity| entity.eval_security_user_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_role_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self
            .result
            .and_then("security_role_id", |entity| entity.eval_security_role_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_tenant(self) -> crate::TenantExpression<'a> {
        let next = self
            .result
            .and_then("tenant", |entity| entity.eval_tenant());
        crate::TenantExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_user(self) -> crate::SecurityUserExpression<'a> {
        let next = self
            .result
            .and_then("security_user", |entity| entity.eval_security_user());
        crate::SecurityUserExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_role(self) -> crate::SecurityRoleExpression<'a> {
        let next = self
            .result
            .and_then("security_role", |entity| entity.eval_security_role());
        crate::SecurityRoleExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct SecurityUserRoleListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::SecurityUserRole>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityUserRoleListExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::SecurityUserRole>>,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityUserRole>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityUserRole>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::SecurityUserRole> {
        self.resolve()
            .expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| {
            teaql_core::eval::EvalResult::Value(list.len())
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::SecurityUserRoleExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityUserRoleExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::SecurityUserRoleExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityUserRoleExpression::new(next, self.root_desc.clone())
    }
}
