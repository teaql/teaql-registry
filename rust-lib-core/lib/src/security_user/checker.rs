use teaql_runtime::{CheckObjectStatus, CheckResults, ObjectLocation, TypedChecker, UserContext};

pub trait SecurityUserCheckerLogic: Send + Sync {
    fn check_and_fix_security_user(
        &self,
        _context: &UserContext,
        _entity: &mut crate::SecurityUser,
        _status: CheckObjectStatus,
        _location: &ObjectLocation,
        _results: &mut CheckResults,
    ) {
    }

    fn required(
        &self,
        value: bool,
        field: &str,
        location: &ObjectLocation,
        results: &mut CheckResults,
    ) {
        if !value {
            results.push(teaql_runtime::CheckResult::required(
                location.clone().member(field),
            ));
        }
    }

    fn required_option<V>(
        &self,
        value: Option<&V>,
        field: &str,
        location: &ObjectLocation,
        results: &mut CheckResults,
    ) {
        if value.is_none() {
            results.push(teaql_runtime::CheckResult::required(
                location.clone().member(field),
            ));
        }
    }

    fn required_text(
        &self,
        value: &str,
        field: &str,
        location: &ObjectLocation,
        results: &mut CheckResults,
    ) {
        if value.trim().is_empty() {
            results.push(teaql_runtime::CheckResult::required(
                location.clone().member(field),
            ));
        }
    }

    fn min_string_length(
        &self,
        value: &str,
        field: &str,
        min_len: usize,
        location: &ObjectLocation,
        results: &mut CheckResults,
    ) {
        if value.chars().count() < min_len {
            results.push(teaql_runtime::CheckResult::min_str(
                location.clone().member(field),
                min_len as u64,
                value.to_owned(),
            ));
        }
    }

    fn max_string_length(
        &self,
        value: &str,
        field: &str,
        max_len: usize,
        location: &ObjectLocation,
        results: &mut CheckResults,
    ) {
        if value.chars().count() > max_len {
            results.push(teaql_runtime::CheckResult::max_str(
                location.clone().member(field),
                max_len as u64,
                value.to_owned(),
            ));
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NoopSecurityUserChecker;

impl SecurityUserCheckerLogic for NoopSecurityUserChecker {}

#[derive(Clone, Debug)]
pub struct SecurityUserChecker<L = NoopSecurityUserChecker> {
    logic: L,
}

impl Default for SecurityUserChecker<NoopSecurityUserChecker> {
    fn default() -> Self {
        Self {
            logic: NoopSecurityUserChecker,
        }
    }
}

impl<L> SecurityUserChecker<L>
where
    L: SecurityUserCheckerLogic,
{
    pub fn new(logic: L) -> Self {
        Self { logic }
    }
}

impl<L> TypedChecker<crate::SecurityUser> for SecurityUserChecker<L>
where
    L: SecurityUserCheckerLogic,
{
    fn check_and_fix_typed(
        &self,
        context: &UserContext,
        entity: &mut crate::SecurityUser,
        status: CheckObjectStatus,
        location: &ObjectLocation,
        results: &mut CheckResults,
    ) {
        if status.is_update() && !entity.is_loaded("id") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("id"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("tenant_id") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("tenant"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("username") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("username"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("first_name") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("first_name"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("last_name") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("last_name"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("password_hash") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("password_hash"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("user_status_id") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("user_status"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("email") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("email"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("version") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("version"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        self.logic
            .check_and_fix_security_user(context, entity, status, location, results);
    }
}
