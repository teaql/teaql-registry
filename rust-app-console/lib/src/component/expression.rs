#[derive(Clone)]
pub struct ComponentExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::Component>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> ComponentExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::Component>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::Component> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::Component> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::Component {
        self.resolve().expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_namespace(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("namespace", |entity| entity.eval_namespace());
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

    pub fn get_normalized_version(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("normalized_version", |entity| entity.eval_normalized_version());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_kind(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("kind", |entity| entity.eval_kind());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_content_repository_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("content_repository_id", |entity| entity.eval_content_repository_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_content_repository(self) -> crate::ContentRepositoryExpression<'a> {
        let next = self.result.and_then("content_repository", |entity| entity.eval_content_repository());
        crate::ContentRepositoryExpression::new(next, self.root_desc.clone())
    }
    pub fn get_asset_list(self) -> crate::AssetListExpression<'a> {
        let next = self.result.and_then("asset_list", |entity| entity.eval_asset_list());
        crate::AssetListExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct ComponentListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Component>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> ComponentListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::Component>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::Component>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::Component>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::Component> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::ComponentExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::ComponentExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::ComponentExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::ComponentExpression::new(next, self.root_desc.clone())
    }
}