#[derive(Clone)]
pub struct AssetBlobExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::AssetBlob>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> AssetBlobExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::AssetBlob>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::AssetBlob> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::AssetBlob> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::AssetBlob {
        self.resolve().expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_blob_ref(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("blob_ref", |entity| entity.eval_blob_ref());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_blob_size(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("blob_size", |entity| entity.eval_blob_size());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_content_type(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("content_type", |entity| entity.eval_content_type());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_sha1_checksum(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("sha1_checksum", |entity| entity.eval_sha1_checksum());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_sha256_checksum(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("sha256_checksum", |entity| entity.eval_sha256_checksum());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_md5_checksum(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("md5_checksum", |entity| entity.eval_md5_checksum());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_version(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("version", |entity| entity.eval_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_blob_store_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("blob_store_id", |entity| entity.eval_blob_store_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_blob_store(self) -> crate::BlobStoreConfigurationExpression<'a> {
        let next = self.result.and_then("blob_store", |entity| entity.eval_blob_store());
        crate::BlobStoreConfigurationExpression::new(next, self.root_desc.clone())
    }
    pub fn get_asset_list(self) -> crate::AssetListExpression<'a> {
        let next = self.result.and_then("asset_list", |entity| entity.eval_asset_list());
        crate::AssetListExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct AssetBlobListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::AssetBlob>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> AssetBlobListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::AssetBlob>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::AssetBlob>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::AssetBlob>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::AssetBlob> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::AssetBlobExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::AssetBlobExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::AssetBlobExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::AssetBlobExpression::new(next, self.root_desc.clone())
    }
}