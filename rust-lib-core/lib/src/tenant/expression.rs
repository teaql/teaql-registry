#[derive(Clone)]
pub struct TenantExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::Tenant>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> TenantExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::Tenant>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::Tenant> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::Tenant> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::Tenant {
        self.resolve().expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_name(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("name", |entity| entity.eval_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_code(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("code", |entity| entity.eval_code());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_description(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("description", |entity| entity.eval_description());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_enabled(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("enabled", |entity| entity.eval_enabled());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_version(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("version", |entity| entity.eval_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_platform_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("platform_id", |entity| entity.eval_platform_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_platform(self) -> crate::PlatformExpression<'a> {
        let next = self.result.and_then("platform", |entity| entity.eval_platform());
        crate::PlatformExpression::new(next, self.root_desc.clone())
    }
    pub fn get_blob_store_configuration_list(self) -> crate::BlobStoreConfigurationListExpression<'a> {
        let next = self.result.and_then("blob_store_configuration_list", |entity| entity.eval_blob_store_configuration_list());
        crate::BlobStoreConfigurationListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_repository_configuration_list(self) -> crate::RepositoryConfigurationListExpression<'a> {
        let next = self.result.and_then("repository_configuration_list", |entity| entity.eval_repository_configuration_list());
        crate::RepositoryConfigurationListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_user_list(self) -> crate::SecurityUserListExpression<'a> {
        let next = self.result.and_then("security_user_list", |entity| entity.eval_security_user_list());
        crate::SecurityUserListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_role_list(self) -> crate::SecurityRoleListExpression<'a> {
        let next = self.result.and_then("security_role_list", |entity| entity.eval_security_role_list());
        crate::SecurityRoleListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_security_privilege_list(self) -> crate::SecurityPrivilegeListExpression<'a> {
        let next = self.result.and_then("security_privilege_list", |entity| entity.eval_security_privilege_list());
        crate::SecurityPrivilegeListExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct TenantListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Tenant>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> TenantListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Tenant>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::Tenant>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::Tenant>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::Tenant> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::TenantExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::TenantExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::TenantExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::TenantExpression::new(next, self.root_desc.clone())
    }
}