#[derive(Clone)]
pub struct RepositoryConfigurationExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::RepositoryConfiguration>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> RepositoryConfigurationExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::RepositoryConfiguration>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::RepositoryConfiguration> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::RepositoryConfiguration> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::RepositoryConfiguration {
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

    pub fn get_recipe_name(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("recipe_name", |entity| entity.eval_recipe_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_online(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("online", |entity| entity.eval_online());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_remote_url(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("remote_url", |entity| entity.eval_remote_url());
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

    pub fn get_repository_type_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("repository_type_id", |entity| entity.eval_repository_type_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_repository_format_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("repository_format_id", |entity| entity.eval_repository_format_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_write_policy_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("write_policy_id", |entity| entity.eval_write_policy_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_blob_store_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("blob_store_id", |entity| entity.eval_blob_store_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_platform(self) -> crate::PlatformExpression<'a> {
        let next = self.result.and_then("platform", |entity| entity.eval_platform());
        crate::PlatformExpression::new(next, self.root_desc.clone())
    }

    pub fn get_repository_type(self) -> crate::RepositoryTypeExpression<'a> {
        let next = self.result.and_then("repository_type", |entity| entity.eval_repository_type());
        crate::RepositoryTypeExpression::new(next, self.root_desc.clone())
    }

    pub fn get_repository_format(self) -> crate::RepositoryFormatExpression<'a> {
        let next = self.result.and_then("repository_format", |entity| entity.eval_repository_format());
        crate::RepositoryFormatExpression::new(next, self.root_desc.clone())
    }

    pub fn get_write_policy(self) -> crate::WritePolicyExpression<'a> {
        let next = self.result.and_then("write_policy", |entity| entity.eval_write_policy());
        crate::WritePolicyExpression::new(next, self.root_desc.clone())
    }

    pub fn get_blob_store(self) -> crate::BlobStoreConfigurationExpression<'a> {
        let next = self.result.and_then("blob_store", |entity| entity.eval_blob_store());
        crate::BlobStoreConfigurationExpression::new(next, self.root_desc.clone())
    }
    pub fn repository_type_is_hosted(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("repository_type_id", |entity| {
            if !entity.is_loaded("repository_type_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_type_id".to_string(), attempted_path: "repository_type_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.repository_type_is_hosted())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn repository_type_is_proxy(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("repository_type_id", |entity| {
            if !entity.is_loaded("repository_type_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_type_id".to_string(), attempted_path: "repository_type_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.repository_type_is_proxy())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn repository_type_is_group(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("repository_type_id", |entity| {
            if !entity.is_loaded("repository_type_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_type_id".to_string(), attempted_path: "repository_type_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.repository_type_is_group())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn repository_format_is_maven2(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("repository_format_id", |entity| {
            if !entity.is_loaded("repository_format_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_format_id".to_string(), attempted_path: "repository_format_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.repository_format_is_maven2())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn repository_format_is_raw(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("repository_format_id", |entity| {
            if !entity.is_loaded("repository_format_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "repository_format_id".to_string(), attempted_path: "repository_format_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.repository_format_is_raw())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn write_policy_is_allow_write(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("write_policy_id", |entity| {
            if !entity.is_loaded("write_policy_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "write_policy_id".to_string(), attempted_path: "write_policy_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.write_policy_is_allow_write())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn write_policy_is_allow_once(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("write_policy_id", |entity| {
            if !entity.is_loaded("write_policy_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "write_policy_id".to_string(), attempted_path: "write_policy_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.write_policy_is_allow_once())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn write_policy_is_read_only(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("write_policy_id", |entity| {
            if !entity.is_loaded("write_policy_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "write_policy_id".to_string(), attempted_path: "write_policy_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.write_policy_is_read_only())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_content_repository_list(self) -> crate::ContentRepositoryListExpression<'a> {
        let next = self.result.and_then("content_repository_list", |entity| entity.eval_content_repository_list());
        crate::ContentRepositoryListExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct RepositoryConfigurationListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::RepositoryConfiguration>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> RepositoryConfigurationListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::RepositoryConfiguration>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::RepositoryConfiguration>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::RepositoryConfiguration>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::RepositoryConfiguration> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::RepositoryConfigurationExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::RepositoryConfigurationExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::RepositoryConfigurationExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::RepositoryConfigurationExpression::new(next, self.root_desc.clone())
    }
}