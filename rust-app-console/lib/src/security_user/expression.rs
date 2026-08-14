#[derive(Clone)]
pub struct SecurityUserExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a crate::SecurityUser>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityUserExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a crate::SecurityUser>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a crate::SecurityUser> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a crate::SecurityUser> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a crate::SecurityUser {
        self.resolve().expect("Relation was legitimately null in database!")
    }

    pub fn get_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("id", |entity| entity.eval_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_username(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("username", |entity| entity.eval_username());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_first_name(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("first_name", |entity| entity.eval_first_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_last_name(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("last_name", |entity| entity.eval_last_name());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_password_hash(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("password_hash", |entity| entity.eval_password_hash());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn get_email(self) -> crate::ValueExpression<'a, String> {
        let next = self.result.and_then("email", |entity| entity.eval_email());
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

    pub fn get_user_status_id(self) -> crate::ValueExpression<'a, u64> {
        let next = self.result.and_then("user_status_id", |entity| entity.eval_user_status_id());
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
    pub fn get_platform(self) -> crate::PlatformExpression<'a> {
        let next = self.result.and_then("platform", |entity| entity.eval_platform());
        crate::PlatformExpression::new(next, self.root_desc.clone())
    }

    pub fn get_user_status(self) -> crate::UserStatusExpression<'a> {
        let next = self.result.and_then("user_status", |entity| entity.eval_user_status());
        crate::UserStatusExpression::new(next, self.root_desc.clone())
    }
    pub fn user_status_is_active(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("user_status_id", |entity| {
            if !entity.is_loaded("user_status_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "user_status_id".to_string(), attempted_path: "user_status_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.user_status_is_active())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn user_status_is_disabled(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("user_status_id", |entity| {
            if !entity.is_loaded("user_status_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "user_status_id".to_string(), attempted_path: "user_status_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.user_status_is_disabled())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn user_status_is_locked(self) -> crate::ValueExpression<'a, bool> {
        let next = self.result.and_then("user_status_id", |entity| {
            if !entity.is_loaded("user_status_id") {
                teaql_core::eval::EvalResult::NotLoaded { failed_node: "user_status_id".to_string(), attempted_path: "user_status_id".to_string() }
            } else {
                teaql_core::eval::EvalResult::Value(entity.user_status_is_locked())
            }
        });
        crate::ValueExpression::new(next, self.root_desc.clone())
    }
}

#[derive(Clone)]
pub struct SecurityUserListExpression<'a> {
    result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::SecurityUser>>,
    root_desc: std::sync::Arc<String>,
}

impl<'a> SecurityUserListExpression<'a> {
    pub fn new(result: teaql_core::eval::EvalResult<&'a teaql_core::SmartList<crate::SecurityUser>>, root_desc: std::sync::Arc<String>) -> Self {
        Self { result, root_desc }
    }

    fn resolve(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityUser>> {
        match &self.result {
            teaql_core::eval::EvalResult::Value(v) => Some(*v),
            teaql_core::eval::EvalResult::Null => None,
            teaql_core::eval::EvalResult::NotLoaded { failed_node, attempted_path } => {
                crate::trigger_logic_bug_panic(&self.root_desc, &failed_node, &attempted_path)
            }
        }
    }

    pub fn eval(&self) -> Option<&'a teaql_core::SmartList<crate::SecurityUser>> {
        self.resolve()
    }

    pub fn unwrap(&self) -> &'a teaql_core::SmartList<crate::SecurityUser> {
        self.resolve().expect("List relation was legitimately null in database!")
    }

    pub fn size(&self) -> crate::ValueExpression<'a, usize> {
        let next = self.result.clone().and_then("size", |list| teaql_core::eval::EvalResult::Value(list.len()));
        crate::ValueExpression::new(next, self.root_desc.clone())
    }

    pub fn first(&self) -> crate::SecurityUserExpression<'a> {
        let next = self.result.clone().and_then("first", |list| {
            if let Some(item) = list.first() {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityUserExpression::new(next, self.root_desc.clone())
    }

    pub fn get(&self, index: usize) -> crate::SecurityUserExpression<'a> {
        let next = self.result.clone().and_then("get", |list| {
            if let Some(item) = list.get(index) {
                teaql_core::eval::EvalResult::Value(item)
            } else {
                teaql_core::eval::EvalResult::Null
            }
        });
        crate::SecurityUserExpression::new(next, self.root_desc.clone())
    }
}