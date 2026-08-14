#[derive(Clone)]
pub struct BlobStoreConfigurationExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::BlobStoreConfiguration>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> BlobStoreConfigurationExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::BlobStoreConfiguration>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::BlobStoreConfiguration> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::BlobStoreConfiguration> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::BlobStoreConfiguration {
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

    pub fn get_path(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("path", |entity| entity.eval_path());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_total_size(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("total_size", |entity| entity.eval_total_size());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_blob_count(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("blob_count", |entity| entity.eval_blob_count());
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

    pub fn get_blob_store_type_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("blob_store_type_id", |entity| entity.eval_blob_store_type_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_tenant(self) -> crate::TenantExpression<'a> {
        let next = self.result.and_then("tenant", |entity| entity.eval_tenant());
        crate::TenantExpression::new(next, self.root_desc.clone())
    }

    pub fn get_blob_store_type(self) -> crate::BlobStoreTypeExpression<'a> {
        let next = self.result.and_then("blob_store_type", |entity| entity.eval_blob_store_type());
        crate::BlobStoreTypeExpression::new(next, self.root_desc.clone())
    }
    pub fn blob_store_type_is_file(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("blob_store_type_id", |entity| {
            if !entity.is_loaded("blob_store_type_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_type_id".to_string(), attempted_path: "blob_store_type_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.blob_store_type_is_file())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn blob_store_type_is_s3(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("blob_store_type_id", |entity| {
            if !entity.is_loaded("blob_store_type_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "blob_store_type_id".to_string(), attempted_path: "blob_store_type_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.blob_store_type_is_s3())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_repository_configuration_list(self) -> crate::RepositoryConfigurationListExpression<'a> {
        let next = self.result.and_then("repository_configuration_list", |entity| entity.eval_repository_configuration_list());
        crate::RepositoryConfigurationListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_asset_blob_list(self) -> crate::AssetBlobListExpression<'a> {
        let next = self.result.and_then("asset_blob_list", |entity| entity.eval_asset_blob_list());
        crate::AssetBlobListExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct BlobStoreConfigurationListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::BlobStoreConfiguration>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> BlobStoreConfigurationListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::BlobStoreConfiguration>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::BlobStoreConfiguration>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::BlobStoreConfiguration>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::BlobStoreConfiguration> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::BlobStoreConfigurationExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::BlobStoreConfigurationExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::BlobStoreConfigurationExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::BlobStoreConfigurationExpression::new(next, self.root_desc.clone())
    }
}