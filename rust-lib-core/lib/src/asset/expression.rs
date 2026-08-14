#[derive(Clone)]
pub struct AssetExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::Asset>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> AssetExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::Asset>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::Asset> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::Asset> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::Asset {
        self.resolve().expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_component_id(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("component_id", |entity| entity.eval_component_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_path(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("path", |entity| entity.eval_path());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_kind(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("kind", |entity| entity.eval_kind());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_version(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("version", |entity| entity.eval_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_content_repository_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("content_repository_id", |entity| entity.eval_content_repository_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_asset_blob_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("asset_blob_id", |entity| entity.eval_asset_blob_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_content_repository(self) -> crate::ContentRepositoryExpression<'a> {
        let next = self.result.and_then("content_repository", |entity| entity.eval_content_repository());
        crate::ContentRepositoryExpression::new(next, self.root_desc.clone())
    }

    pub fn get_asset_blob(self) -> crate::AssetBlobExpression<'a> {
        let next = self.result.and_then("asset_blob", |entity| entity.eval_asset_blob());
        crate::AssetBlobExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct AssetListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Asset>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> AssetListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Asset>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::Asset>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::Asset>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::Asset> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::AssetExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::AssetExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::AssetExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::AssetExpression::new(next, self.root_desc.clone())
    }
}