#[derive(Clone)]
pub struct PlatformExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::Platform>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> PlatformExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::Platform>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::Platform> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::Platform> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::Platform {
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

    pub fn get_version(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("version", |entity| entity.eval_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_repository_type_list(self) -> crate::RepositoryTypeListExpression<'a> {
        let next = self.result.and_then("repository_type_list", |entity| entity.eval_repository_type_list());
        crate::RepositoryTypeListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_repository_format_list(self) -> crate::RepositoryFormatListExpression<'a> {
        let next = self.result.and_then("repository_format_list", |entity| entity.eval_repository_format_list());
        crate::RepositoryFormatListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_write_policy_list(self) -> crate::WritePolicyListExpression<'a> {
        let next = self.result.and_then("write_policy_list", |entity| entity.eval_write_policy_list());
        crate::WritePolicyListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_blob_store_type_list(self) -> crate::BlobStoreTypeListExpression<'a> {
        let next = self.result.and_then("blob_store_type_list", |entity| entity.eval_blob_store_type_list());
        crate::BlobStoreTypeListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_user_status_list(self) -> crate::UserStatusListExpression<'a> {
        let next = self.result.and_then("user_status_list", |entity| entity.eval_user_status_list());
        crate::UserStatusListExpression::new(next, self.root_desc.clone())
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
pub struct PlatformListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Platform>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> PlatformListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Platform>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::Platform>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::Platform>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::Platform> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::PlatformExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::PlatformExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::PlatformExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::PlatformExpression::new(next, self.root_desc.clone())
    }
}