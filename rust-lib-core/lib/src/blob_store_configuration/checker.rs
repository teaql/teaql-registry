use teaql_runtime::{CheckObjectStatus, CheckResults, ObjectLocation, TypedChecker, UserContext};

pub trait BlobStoreConfigurationCheckerLogic: Send + Sync {
    fn check_and_fix_blob_store_configuration(
        &self,
        _context: &UserContext,
        _entity: &mut crate::BlobStoreConfiguration,
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
pub struct NoopBlobStoreConfigurationChecker;

impl BlobStoreConfigurationCheckerLogic for NoopBlobStoreConfigurationChecker {}

#[derive(Clone, Debug)]
pub struct BlobStoreConfigurationChecker<L = NoopBlobStoreConfigurationChecker> {
    logic: L,
}

impl Default for BlobStoreConfigurationChecker<NoopBlobStoreConfigurationChecker> {
    fn default() -> Self {
        Self {
            logic: NoopBlobStoreConfigurationChecker,
        }
    }
}

impl<L> BlobStoreConfigurationChecker<L>
where
    L: BlobStoreConfigurationCheckerLogic,
{
    pub fn new(logic: L) -> Self {
        Self { logic }
    }
}

impl<L> TypedChecker<crate::BlobStoreConfiguration> for BlobStoreConfigurationChecker<L>
where
    L: BlobStoreConfigurationCheckerLogic,
{
    fn check_and_fix_typed(
        &self,
        context: &UserContext,
        entity: &mut crate::BlobStoreConfiguration,
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
        if status.is_update() && !entity.is_loaded("name") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("name"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("blob_store_type_id") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("blob_store_type"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("path") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("path"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("total_size") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("total_size"),
                )
                .with_message("Mutation requires a fully loaded entity"),
            );
        }
        if status.is_update() && !entity.is_loaded("blob_count") {
            results.push(
                teaql_runtime::CheckResult::new(
                    teaql_runtime::CheckRule::InvalidType,
                    location.clone().member("blob_count"),
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
            .check_and_fix_blob_store_configuration(context, entity, status, location, results);
    }
}
