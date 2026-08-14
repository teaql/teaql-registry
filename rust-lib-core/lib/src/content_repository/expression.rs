#[derive(Clone)]
pub struct ContentRepositoryExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::ContentRepository>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> ContentRepositoryExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::ContentRepository>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::ContentRepository> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::ContentRepository> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::ContentRepository {
        self.resolve().expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_repository_id(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("repository_id", |entity| entity.eval_repository_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_format_name(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("format_name", |entity| entity.eval_format_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_version(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("version", |entity| entity.eval_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_component_list(self) -> crate::ComponentListExpression<'a> {
        let next = self.result.and_then("component_list", |entity| entity.eval_component_list());
        crate::ComponentListExpression::new(next, self.root_desc.clone())
    }

    pub fn get_asset_list(self) -> crate::AssetListExpression<'a> {
        let next = self.result.and_then("asset_list", |entity| entity.eval_asset_list());
        crate::AssetListExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct ContentRepositoryListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::ContentRepository>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> ContentRepositoryListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::ContentRepository>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::ContentRepository>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::ContentRepository>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::ContentRepository> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::ContentRepositoryExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::ContentRepositoryExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::ContentRepositoryExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::ContentRepositoryExpression::new(next, self.root_desc.clone())
    }
}