
use std::collections::BTreeMap;
use crate::TeaqlRuntime;
use crate::Q;
use teaql_core::Entity as _;
use crate::request_support::TeaqlUserContextExt as _;
use crate::request_support::AuditedSave as _;

pub trait IntoU64 {
    fn into_u64(self) -> u64;
}

impl IntoU64 for u64 {
    fn into_u64(self) -> u64 {
        self
    }
}

impl IntoU64 for Option<&teaql_core::Value> {
    fn into_u64(self) -> u64 {
        self.and_then(|v| v.try_u64()).unwrap_or_default()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SampleDataScale {
    Tiny,
    Small,
    Medium,
}

pub struct SampleDataPlan {
    pub scale: SampleDataScale,
    pub seed: u64,
}

impl SampleDataPlan {
    pub fn small() -> Self {
        Self {
            scale: SampleDataScale::Small,
            seed: 0,
        }
    }
}

pub struct SampleDataReport {
    pub generated: BTreeMap<&'static str, usize>,
    pub skipped: Vec<SampleDataSkipped>,
}

pub struct SampleDataSkipped {
    pub entity: &'static str,
    pub reason: String,
}

pub struct SampleDataState {
    pub plan: SampleDataPlan,
    pub references: BTreeMap<&'static str, Vec<u64>>,
    pub generated: BTreeMap<&'static str, usize>,
    pub skipped: Vec<SampleDataSkipped>,
}

impl SampleDataState {
    pub fn new(plan: SampleDataPlan) -> Self {
        Self {
            plan,
            references: BTreeMap::new(),
            generated: BTreeMap::new(),
            skipped: Vec::new(),
        }
    }

    pub fn add_reference(&mut self, entity: &'static str, id: u64) {
        self.references.entry(entity).or_default().push(id);
    }

    pub fn ids(&self, entity: &'static str) -> &[u64] {
        self.references.get(entity).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn pick_id(&self, entity: &'static str, salt: usize) -> Option<u64> {
        let ids = self.ids(entity);
        if ids.is_empty() {
            None
        } else {
            Some(ids[salt % ids.len()])
        }
    }

    pub fn pick_unused_id(&self, entity: &'static str, salt: usize, used: &std::collections::HashSet<u64>) -> Option<u64> {
        let ids = self.ids(entity);
        if ids.is_empty() {
            return None;
        }

        let best_id = ids[salt % ids.len()];
        if !used.contains(&best_id) {
            return Some(best_id);
        }

        for id in ids {
            if !used.contains(id) {
                return Some(*id);
            }
        }

        Some(best_id)
    }

    pub fn record_generated(&mut self, entity: &'static str) {
        *self.generated.entry(entity).or_default() += 1;
    }

    pub fn record_skipped(&mut self, entity: &'static str, reason: String) {
        self.skipped.push(SampleDataSkipped { entity, reason });
    }

    pub fn into_report(self) -> SampleDataReport {
        SampleDataReport {
            generated: self.generated,
            skipped: self.skipped,
        }
    }
}

pub async fn generate_sample_data<C>(
    ctx: &C,
    plan: SampleDataPlan,
) -> Result<SampleDataReport, String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    log::info!("Starting sample data generation. Scale: {:?}, Seed: {}", plan.scale, plan.seed);
    let mut state = SampleDataState::new(plan);

    load_root_content_repositories(ctx, &mut state).await?; //depth: 0
    load_root_platforms(ctx, &mut state).await?; //depth: 0

    load_constant_blob_store_types(ctx, &mut state).await?;
    load_constant_repository_formats(ctx, &mut state).await?;
    load_constant_repository_types(ctx, &mut state).await?;
    load_constant_user_statuses(ctx, &mut state).await?;
    load_constant_write_policies(ctx, &mut state).await?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_components(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_tenants(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_blob_store_configurations(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_security_privileges(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_security_roles(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_security_users(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_asset_blobs(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_repository_configurations(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;

    ctx.user_context().transaction_data(|| async {
        Box::pin(generate_assets(ctx, &mut state)).await.map_err(|e| {
            teaql_runtime::DataServiceError::Runtime(teaql_runtime::RuntimeError::Graph(e))
        })
    }).await.map_err(|e| e.to_string())?;


    let report = state.into_report();
    log::info!("Sample data generation completed successfully. Generated: {} tables, Skipped: {} tables.", report.generated.len(), report.skipped.len());
    Ok(report)
}

async fn load_root_content_repositories<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    let list = Q::content_repositories().purpose("Init Sample Data").execute_for_list(ctx).await.unwrap_or_default();
    for item in list {
        state.add_reference(crate::ContentRepository::ENTITY_NAME, item.id().into_u64());
    }
    Ok(())
}

async fn load_root_platforms<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    let list = Q::platforms().purpose("Init Sample Data").execute_for_list(ctx).await.unwrap_or_default();
    for item in list {
        state.add_reference(crate::Platform::ENTITY_NAME, item.id().into_u64());
    }
    Ok(())
}

async fn load_constant_blob_store_types<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    let list = Q::blob_store_types().purpose("Init Sample Data").execute_for_list(ctx).await.unwrap_or_default();
    for item in list {
        state.add_reference(crate::BlobStoreType::ENTITY_NAME, item.id().into_u64());
    }
    Ok(())
}

async fn load_constant_repository_formats<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    let list = Q::repository_formats().purpose("Init Sample Data").execute_for_list(ctx).await.unwrap_or_default();
    for item in list {
        state.add_reference(crate::RepositoryFormat::ENTITY_NAME, item.id().into_u64());
    }
    Ok(())
}

async fn load_constant_repository_types<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    let list = Q::repository_types().purpose("Init Sample Data").execute_for_list(ctx).await.unwrap_or_default();
    for item in list {
        state.add_reference(crate::RepositoryType::ENTITY_NAME, item.id().into_u64());
    }
    Ok(())
}

async fn load_constant_user_statuses<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    let list = Q::user_statuses().purpose("Init Sample Data").execute_for_list(ctx).await.unwrap_or_default();
    for item in list {
        state.add_reference(crate::UserStatus::ENTITY_NAME, item.id().into_u64());
    }
    Ok(())
}

async fn load_constant_write_policies<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
    let list = Q::write_policies().purpose("Init Sample Data").execute_for_list(ctx).await.unwrap_or_default();
    for item in list {
        state.add_reference(crate::WritePolicy::ENTITY_NAME, item.id().into_u64());
    }
    Ok(())
}

async fn generate_components<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Content Repository").is_empty() {
            state.record_skipped(crate::Component::ENTITY_NAME, "Required dependency Content Repository is missing in reference pool".to_string());
            log::info!("Skipped generating Component: Required dependency Content Repository is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Component (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::components().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Content Repository", i as usize, &used_refs) {
                    entity.update_content_repository_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_namespace(format!("{} {}", "org.apache.commons", i + 1));

                entity.update_name(format!("{} {}", "commons-lang3", i + 1));

                entity.update_version_name(format!("{} {}", "3.12.0", i + 1));

                entity.update_normalized_version(format!("{} {}", "3.12.0", i + 1));

                entity.update_kind(format!("{} {}", "jar", i + 1));



entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::Component::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Component: {}/{}", i, fanout);
        }

    }

    log::info!("Successfully generated sample records for Component.");
    Ok(())
}


async fn generate_tenants<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Nexus Platform").is_empty() {
            state.record_skipped(crate::Tenant::ENTITY_NAME, "Required dependency Nexus Platform is missing in reference pool".to_string());
            log::info!("Skipped generating Tenant: Required dependency Nexus Platform is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Tenant (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::tenants().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Nexus Platform", i as usize, &used_refs) {
                    entity.update_platform_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_name(format!("{} {}", "Default Tenant", i + 1));

                entity.update_code(format!("{} {}", "default", i + 1));

                entity.update_description(format!("{} {}", "Default Tenant Environment", i + 1));




        let entity = entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::Tenant::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Tenant: {}/{}", i, fanout);
        }

        state.add_reference(crate::Tenant::ENTITY_NAME, entity.id().into_u64());
    }

    log::info!("Successfully generated sample records for Tenant.");
    Ok(())
}


async fn generate_blob_store_configurations<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Tenant").is_empty() {
            state.record_skipped(crate::BlobStoreConfiguration::ENTITY_NAME, "Required dependency Tenant is missing in reference pool".to_string());
            log::info!("Skipped generating Blob Store Configuration: Required dependency Tenant is missing in reference pool.");
            return Ok(());
        }

        if state.ids("Blob Store Type").is_empty() {
            state.record_skipped(crate::BlobStoreConfiguration::ENTITY_NAME, "Required dependency Blob Store Type is missing in reference pool".to_string());
            log::info!("Skipped generating Blob Store Configuration: Required dependency Blob Store Type is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Blob Store Configuration (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::blob_store_configurations().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Tenant", i as usize, &used_refs) {
                    entity.update_tenant_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                if let Some(ref_id) = state.pick_unused_id("Blob Store Type", i as usize, &used_refs) {
                    entity.update_blob_store_type_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_name(format!("{} {}", "default", i + 1));

                entity.update_path(format!("{} {}", "/opt/nexus/blobs/default", i + 1));

                {
                    let max_val: u64 = "1073741824l".parse().unwrap_or(1000);
                    let rand_val = (i as u64 + state.plan.seed) % max_val.max(1) + 1;
                    entity.update_total_size(rand_val as i64);
                }

                {
                    let max_val: u64 = "100".parse().unwrap_or(1000);
                    let rand_val = (i as u64 + state.plan.seed) % max_val.max(1) + 1;
                    entity.update_blob_count(rand_val as i64);
                }



        let entity = entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::BlobStoreConfiguration::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Blob Store Configuration: {}/{}", i, fanout);
        }

        state.add_reference(crate::BlobStoreConfiguration::ENTITY_NAME, entity.id().into_u64());
    }

    log::info!("Successfully generated sample records for Blob Store Configuration.");
    Ok(())
}


async fn generate_security_privileges<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Tenant").is_empty() {
            state.record_skipped(crate::SecurityPrivilege::ENTITY_NAME, "Required dependency Tenant is missing in reference pool".to_string());
            log::info!("Skipped generating Security Privilege: Required dependency Tenant is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Security Privilege (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::security_privileges().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Tenant", i as usize, &used_refs) {
                    entity.update_tenant_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_privilege_id(format!("{} {}", "nx-all", i + 1));

                entity.update_name(format!("{} {}", "All Privileges", i + 1));

                entity.update_description(format!("{} {}", "All administrative permissions", i + 1));

                entity.update_privilege_type(format!("{} {}", "wildcard", i + 1));

                entity.update_permission_pattern(format!("{} {}", "*.*.*", i + 1));




entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::SecurityPrivilege::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Security Privilege: {}/{}", i, fanout);
        }

    }

    log::info!("Successfully generated sample records for Security Privilege.");
    Ok(())
}


async fn generate_security_roles<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Tenant").is_empty() {
            state.record_skipped(crate::SecurityRole::ENTITY_NAME, "Required dependency Tenant is missing in reference pool".to_string());
            log::info!("Skipped generating Security Role: Required dependency Tenant is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Security Role (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::security_roles().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Tenant", i as usize, &used_refs) {
                    entity.update_tenant_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_role_id(format!("{} {}", "nx-admin", i + 1));

                entity.update_name(format!("{} {}", "Nexus Administrator Role", i + 1));

                entity.update_description(format!("{} {}", "Full administrator access", i + 1));




entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::SecurityRole::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Security Role: {}/{}", i, fanout);
        }

    }

    log::info!("Successfully generated sample records for Security Role.");
    Ok(())
}


async fn generate_security_users<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Tenant").is_empty() {
            state.record_skipped(crate::SecurityUser::ENTITY_NAME, "Required dependency Tenant is missing in reference pool".to_string());
            log::info!("Skipped generating Security User: Required dependency Tenant is missing in reference pool.");
            return Ok(());
        }

        if state.ids("User Status").is_empty() {
            state.record_skipped(crate::SecurityUser::ENTITY_NAME, "Required dependency User Status is missing in reference pool".to_string());
            log::info!("Skipped generating Security User: Required dependency User Status is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Security User (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::security_users().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Tenant", i as usize, &used_refs) {
                    entity.update_tenant_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                if let Some(ref_id) = state.pick_unused_id("User Status", i as usize, &used_refs) {
                    entity.update_user_status_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_username(format!("{} {}", "admin", i + 1));

                entity.update_first_name(format!("{} {}", "Administrator", i + 1));

                entity.update_last_name(format!("{} {}", "User", i + 1));

                entity.update_password_hash(format!("{} {}", "sha512$hashedpassword", i + 1));

                entity.update_email(format!("{} {}", "admin@example.com", i + 1));



entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::SecurityUser::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Security User: {}/{}", i, fanout);
        }

    }

    log::info!("Successfully generated sample records for Security User.");
    Ok(())
}


async fn generate_asset_blobs<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Blob Store Configuration").is_empty() {
            state.record_skipped(crate::AssetBlob::ENTITY_NAME, "Required dependency Blob Store Configuration is missing in reference pool".to_string());
            log::info!("Skipped generating Asset Blob: Required dependency Blob Store Configuration is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Asset Blob (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::asset_blobs().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Blob Store Configuration", i as usize, &used_refs) {
                    entity.update_blob_store_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_blob_ref(format!("{} {}", "default@b7f83e20-5c6a-4d2a-89a1-8d2a6a12b4e8", i + 1));

                {
                    let max_val: u64 = "587392l".parse().unwrap_or(1000);
                    let rand_val = (i as u64 + state.plan.seed) % max_val.max(1) + 1;
                    entity.update_blob_size(rand_val as i64);
                }

                entity.update_content_type(format!("{} {}", "application/java-archive", i + 1));

                entity.update_sha1_checksum(format!("{} {}", "7e02b7e5e3a8ef5b4f8d2a6a12b4e8c1a9e3d5b7", i + 1));

                entity.update_sha256_checksum(format!("{} {}", "4a5c8e2a1b9f6d7c8e3a2b1c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e", i + 1));

                entity.update_md5_checksum(format!("{} {}", "d41d8cd98f00b204e9800998ecf8427e", i + 1));



        let entity = entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::AssetBlob::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Asset Blob: {}/{}", i, fanout);
        }

        state.add_reference(crate::AssetBlob::ENTITY_NAME, entity.id().into_u64());
    }

    log::info!("Successfully generated sample records for Asset Blob.");
    Ok(())
}


async fn generate_repository_configurations<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Tenant").is_empty() {
            state.record_skipped(crate::RepositoryConfiguration::ENTITY_NAME, "Required dependency Tenant is missing in reference pool".to_string());
            log::info!("Skipped generating Repository Configuration: Required dependency Tenant is missing in reference pool.");
            return Ok(());
        }

        if state.ids("Repository Type").is_empty() {
            state.record_skipped(crate::RepositoryConfiguration::ENTITY_NAME, "Required dependency Repository Type is missing in reference pool".to_string());
            log::info!("Skipped generating Repository Configuration: Required dependency Repository Type is missing in reference pool.");
            return Ok(());
        }

        if state.ids("Repository Format").is_empty() {
            state.record_skipped(crate::RepositoryConfiguration::ENTITY_NAME, "Required dependency Repository Format is missing in reference pool".to_string());
            log::info!("Skipped generating Repository Configuration: Required dependency Repository Format is missing in reference pool.");
            return Ok(());
        }

        if state.ids("Write Policy").is_empty() {
            state.record_skipped(crate::RepositoryConfiguration::ENTITY_NAME, "Required dependency Write Policy is missing in reference pool".to_string());
            log::info!("Skipped generating Repository Configuration: Required dependency Write Policy is missing in reference pool.");
            return Ok(());
        }

        if state.ids("Blob Store Configuration").is_empty() {
            state.record_skipped(crate::RepositoryConfiguration::ENTITY_NAME, "Required dependency Blob Store Configuration is missing in reference pool".to_string());
            log::info!("Skipped generating Repository Configuration: Required dependency Blob Store Configuration is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1 + 1 + 1 + 1 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Repository Configuration (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::repository_configurations().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Tenant", i as usize, &used_refs) {
                    entity.update_tenant_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                if let Some(ref_id) = state.pick_unused_id("Repository Type", i as usize, &used_refs) {
                    entity.update_repository_type_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                if let Some(ref_id) = state.pick_unused_id("Repository Format", i as usize, &used_refs) {
                    entity.update_repository_format_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                if let Some(ref_id) = state.pick_unused_id("Write Policy", i as usize, &used_refs) {
                    entity.update_write_policy_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                if let Some(ref_id) = state.pick_unused_id("Blob Store Configuration", i as usize, &used_refs) {
                    entity.update_blob_store_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                entity.update_name(format!("{} {}", "maven-releases", i + 1));

                entity.update_recipe_name(format!("{} {}", "maven2-hosted", i + 1));


                entity.update_remote_url(format!("{} {}", "//repo1.maven.org/maven2", i + 1));



entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::RepositoryConfiguration::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Repository Configuration: {}/{}", i, fanout);
        }

    }

    log::info!("Successfully generated sample records for Repository Configuration.");
    Ok(())
}


async fn generate_assets<C>(
    ctx: &C,
    state: &mut SampleDataState,
) -> Result<(), String>
where
    C: TeaqlRuntime + ?Sized + crate::TeaqlRepositoryProvider,
{
        if state.ids("Content Repository").is_empty() {
            state.record_skipped(crate::Asset::ENTITY_NAME, "Required dependency Content Repository is missing in reference pool".to_string());
            log::info!("Skipped generating Asset: Required dependency Content Repository is missing in reference pool.");
            return Ok(());
        }

        if state.ids("Asset Blob").is_empty() {
            state.record_skipped(crate::Asset::ENTITY_NAME, "Required dependency Asset Blob is missing in reference pool".to_string());
            log::info!("Skipped generating Asset: Required dependency Asset Blob is missing in reference pool.");
            return Ok(());
        }


    let object_fields_count = 0 + 1 + 1;
    let base_fanout = std::cmp::max(1, object_fields_count) * 20;

    let fanout = match state.plan.scale {
        SampleDataScale::Tiny => base_fanout,
        SampleDataScale::Small => base_fanout * 5,
        SampleDataScale::Medium => base_fanout * 50,
    };

    log::info!("Generating sample data for Asset (expected: {})...", fanout);

    for i in 0..fanout {
        let mut entity = Q::assets().purpose("Init Sample Data").new_entity(ctx);
        let mut used_refs = std::collections::HashSet::new();

                if let Some(ref_id) = state.pick_unused_id("Content Repository", i as usize, &used_refs) {
                    entity.update_content_repository_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                if let Some(ref_id) = state.pick_unused_id("Asset Blob", i as usize, &used_refs) {
                    entity.update_asset_blob_id(ref_id);
                    used_refs.insert(ref_id);
                } else {
                    // Optional relation was missing in reference pool
                }
                {
                    let max_val: u64 = "1l".parse().unwrap_or(1000);
                    let rand_val = (i as u64 + state.plan.seed) % max_val.max(1) + 1;
                    entity.update_component_id(rand_val as i64);
                }

                entity.update_path(format!("{} {}", "/org/apache/commons/commons-lang3/3.12.0/commons-lang3-3.12.0.jar", i + 1));

                entity.update_kind(format!("{} {}", "jar", i + 1));



entity.audit_as("Init Sample Data").save(ctx).await.map_err(|e| e.to_string())?;

        state.record_generated(crate::Asset::ENTITY_NAME);

        if i % 20 == 0 {
            log::info!("Generating Asset: {}/{}", i, fanout);
        }

    }

    log::info!("Successfully generated sample records for Asset.");
    Ok(())
}
