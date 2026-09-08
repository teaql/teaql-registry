#[derive(Clone)]
pub struct PersonalAccessTokenExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::PersonalAccessToken>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> PersonalAccessTokenExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<&'a crate::PersonalAccessToken>,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::PersonalAccessToken> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a crate::PersonalAccessToken> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::PersonalAccessToken {
        self.resolve()
            .expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_username(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("username", |entity| entity.eval_username());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_token_id(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("token_id", |entity| entity.eval_token_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_token_hash(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("token_hash", |entity| entity.eval_token_hash());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_description(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("description", |entity| entity.eval_description());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_scopes(self) -> crate::ValueExpression<'a, String> {
        let next = self
            .result
            .and_then("scopes", |entity| entity.eval_scopes());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_created_at(self) -> crate::ValueExpression<'a, teaql_core::time::Timestamp> {
        let next = self
            .result
            .and_then("created_at", |entity| entity.eval_created_at());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_expires_at_epoch_millis(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("expires_at_epoch_millis", |entity| {
            entity.eval_expires_at_epoch_millis()
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_revoked(self) -> crate::ValueExpression<'a, bool> {
        let next = self
            .result
            .and_then("revoked", |entity| entity.eval_revoked());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_revoked_at_epoch_millis(self) -> crate::ValueExpression<'a, i64> {
        let next = self.result.and_then("revoked_at_epoch_millis", |entity| {
            entity.eval_revoked_at_epoch_millis()
        });
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
}

#[derive(Clone)]
pub struct PersonalAccessTokenListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::PersonalAccessToken>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> PersonalAccessTokenListExpression<'a> {
    pub fn new(
        result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::PersonalAccessToken>>,
        root_desc: std::sync::Arc<String>,
    ) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::PersonalAccessToken>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node,
                attempted_path,
            } => crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path),
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::PersonalAccessToken>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::PersonalAccessToken> {
        self.resolve()
            .expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| {
            teaql_core::eval::EvalResult::Value(list.len())
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::PersonalAccessTokenExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::PersonalAccessTokenExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::PersonalAccessTokenExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::PersonalAccessTokenExpression::new(next, self.root_desc.clone())
    }
}
