#[derive(Clone)]
pub struct SecurityRolePrivilegeExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::SecurityRolePrivilege>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityRolePrivilegeExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<&'a crate::SecurityRolePrivilege>,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::SecurityRolePrivilege> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a crate::SecurityRolePrivilege> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::SecurityRolePrivilege {
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

    pub fn get_security_role_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self
            .result
            .and_then("security_role_id", |entity| entity.eval_security_role_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_privilege_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("security_privilege_id", |entity| {
            entity.eval_security_privilege_id()
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_tenant(self) -> crate::TenantExpression<'a> {
        let next = self
            .result
            .and_then("tenant", |entity| entity.eval_tenant());
        crate::TenantExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_role(self) -> crate::SecurityRoleExpression<'a> {
        let next = self
            .result
            .and_then("security_role", |entity| entity.eval_security_role());
        crate::SecurityRoleExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_privilege(self) -> crate::SecurityPrivilegeExpression<'a> {
        let next = self.result.and_then("security_privilege", |entity| {
            entity.eval_security_privilege()
        });
        crate::SecurityPrivilegeExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct SecurityRolePrivilegeListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::SecurityRolePrivilege>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityRolePrivilegeListExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<
            &'a teaql_core::SmartList<crate::SecurityRolePrivilege>,
        >,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityRolePrivilege>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityRolePrivilege>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::SecurityRolePrivilege> {
        self.resolve()
            .expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| {
            teaql_core::eval::EvalResult::Value(list.len())
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::SecurityRolePrivilegeExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityRolePrivilegeExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::SecurityRolePrivilegeExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityRolePrivilegeExpression::new(next, self.root_desc.clone())
    }
}
