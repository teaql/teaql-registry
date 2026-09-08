use teaql_runtime::{CheckObjectStatus, CheckResults, ObjectLocation, TypedChecker, UserContext};

pub trait PersonalAccessTokenCheckerLogic: Send + Sync {
    fn check_and_fix_personal_access_token(
        &self,
        _context: &UserContext,
        _entity: &mut crate::PersonalAccessToken,
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
pub struct NoopPersonalAccessTokenChecker;

impl PersonalAccessTokenCheckerLogic for NoopPersonalAccessTokenChecker {}

#[derive(Clone, Debug)]
pub struct PersonalAccessTokenChecker<L = NoopPersonalAccessTokenChecker> {
    logic: L,
}

impl Default for PersonalAccessTokenChecker<NoopPersonalAccessTokenChecker> {
    fn default() -> Self {
        Self {
            logic: NoopPersonalAccessTokenChecker,
        }
    }
}

impl<L> PersonalAccessTokenChecker<L>
where
    L: PersonalAccessTokenCheckerLogic,
{
    pub fn new(logic: L) -> Self {
        Self { logic }
    }
}

impl<L> TypedChecker<crate::PersonalAccessToken> for PersonalAccessTokenChecker<L>
where
    L: PersonalAccessTokenCheckerLogic,
{
    fn check_and_fix_typed(
        &self,
        context: &UserContext,
        entity: &mut crate::PersonalAccessToken,
        status: CheckObjectStatus,
        location: &ObjectLocation,
        results: &mut CheckResults,
    ) {
        if status.is_create() {
            entity.update_created_at(context.fix_time());
            context.record_fix_evidence(teaql_runtime::FixEvidence::new(
                "PersonalAccessToken",
                "created_at",
                teaql_runtime::FixEvidenceSource::Clock,
                "graphClock",
            ));
        }

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
        if status.is_update() && !entity.is_loaded("security_user_id") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("security_user"),
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
        if status.is_update() && !entity.is_loaded("token_id") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("token_id"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("token_hash") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("token_hash"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("description") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("description"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("scopes") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("scopes"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("created_at") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("created_at"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("expires_at_epoch_millis") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("expires_at_epoch_millis"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("revoked") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("revoked"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("revoked_at_epoch_millis") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("revoked_at_epoch_millis"),
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
            .check_and_fix_personal_access_token(context, entity, status, location, results);
    }
}
