use crate::*;
use teaql_core::TeaqlEntity;

use teaql_provider_postgres::PostgresProviderExt as _;

pub type DataServiceDialect = teaql_provider_postgres::PostgresDialect;
pub type DataServiceMutationExecutor = teaql_provider_postgres::PgMutationExecutor;
pub type DataServiceMutationError = teaql_provider_postgres::MutationExecutorError;
pub type DataServiceIdGenerator = teaql_provider_postgres::PgIdSpaceGenerator;
pub type DataServicePool = deadpool_postgres::Pool;
pub type DataServiceExecutor = ServiceRuntimeExecutor;
pub type ServiceRuntime = teaql_runtime::UserContext;

pub const DATABASE_URL_ENV: &str = "TEAQL_REGISTRY_SERVICE_CORE_DATABASE_URL";
pub const DATABASE_USER_ENV: &str = "TEAQL_REGISTRY_SERVICE_CORE_DATABASE_USER";
pub const DATABASE_PASSWORD_ENV: &str = "TEAQL_REGISTRY_SERVICE_CORE_DATABASE_PASSWORD";
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceRuntimeConfig {
    pub database_url: String,
    pub database_user: String,
    pub database_password: String,
}

impl ServiceRuntimeConfig {
    pub fn from_env() -> Result<Self, ServiceRuntimeError> {
        Ok(Self {
            database_url: env_value(DATABASE_URL_ENV)?,
            database_user: env_value(DATABASE_USER_ENV)?,
            database_password: env_value(DATABASE_PASSWORD_ENV)?,
        })
    }
}

#[derive(Debug)]
pub enum ServiceRuntimeError {
    MissingEnv {
        name: &'static str,
        source: std::env::VarError,
    },
    ConnectionError(String),
    Runtime(teaql_runtime::RuntimeError),
}

impl std::fmt::Display for ServiceRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceRuntimeError::MissingEnv { name, source } => {
                write!(f, "missing environment variable {name}: {source}")
            }
            ServiceRuntimeError::ConnectionError(err) => write!(f, "connection error: {err}"),
            ServiceRuntimeError::Runtime(err) => write!(f, "runtime error: {err}"),
        }
    }
}

impl std::error::Error for ServiceRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServiceRuntimeError::MissingEnv { source, .. } => Some(source),
            ServiceRuntimeError::ConnectionError(_) => None,
            ServiceRuntimeError::Runtime(err) => Some(err),
        }
    }
}

impl From<teaql_runtime::RuntimeError> for ServiceRuntimeError {
    fn from(err: teaql_runtime::RuntimeError) -> Self {
        ServiceRuntimeError::Runtime(err)
    }
}

#[derive(Clone)]
pub struct LocalSchemaProvider;

impl teaql_data_service::SchemaProvider for LocalSchemaProvider {
    fn get_entity(&self, name: &str) -> Option<std::sync::Arc<teaql_core::EntityDescriptor>> {
        match name {
            "Platform" => Some(std::sync::Arc::new(crate::Platform::entity_descriptor())),
            "Tenant" => Some(std::sync::Arc::new(crate::Tenant::entity_descriptor())),
            "RepositoryType" => Some(std::sync::Arc::new(
                crate::RepositoryType::entity_descriptor(),
            )),
            "RepositoryFormat" => Some(std::sync::Arc::new(
                crate::RepositoryFormat::entity_descriptor(),
            )),
            "WritePolicy" => Some(std::sync::Arc::new(crate::WritePolicy::entity_descriptor())),
            "BlobStoreType" => Some(std::sync::Arc::new(
                crate::BlobStoreType::entity_descriptor(),
            )),
            "UserStatus" => Some(std::sync::Arc::new(crate::UserStatus::entity_descriptor())),
            "BlobStoreConfiguration" => Some(std::sync::Arc::new(
                crate::BlobStoreConfiguration::entity_descriptor(),
            )),
            "RepositoryConfiguration" => Some(std::sync::Arc::new(
                crate::RepositoryConfiguration::entity_descriptor(),
            )),
            "ContentRepository" => Some(std::sync::Arc::new(
                crate::ContentRepository::entity_descriptor(),
            )),
            "Component" => Some(std::sync::Arc::new(crate::Component::entity_descriptor())),
            "AssetBlob" => Some(std::sync::Arc::new(crate::AssetBlob::entity_descriptor())),
            "Asset" => Some(std::sync::Arc::new(crate::Asset::entity_descriptor())),
            "SecurityUser" => Some(std::sync::Arc::new(crate::SecurityUser::entity_descriptor())),
            "SecurityRole" => Some(std::sync::Arc::new(crate::SecurityRole::entity_descriptor())),
            "SecurityPrivilege" => Some(std::sync::Arc::new(
                crate::SecurityPrivilege::entity_descriptor(),
            )),
            "SecurityUserRole" => Some(std::sync::Arc::new(
                crate::SecurityUserRole::entity_descriptor(),
            )),
            "SecurityRolePrivilege" => Some(std::sync::Arc::new(
                crate::SecurityRolePrivilege::entity_descriptor(),
            )),
            "ServiceLog" => Some(std::sync::Arc::new(crate::ServiceLog::entity_descriptor())),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ServiceRuntimeExecutor {
    inner: teaql_sql::SqlDataServiceExecutor<
        DataServiceDialect,
        DataServiceMutationExecutor,
        LocalSchemaProvider,
    >,
}

impl ServiceRuntimeExecutor {
    pub fn new(inner: DataServiceMutationExecutor) -> Self {
        Self {
            inner: teaql_sql::SqlDataServiceExecutor::new(
                DataServiceDialect::default(),
                inner,
                LocalSchemaProvider,
            ),
        }
    }
}

impl teaql_data_service::DataServiceExecutor for ServiceRuntimeExecutor {
    type Error = teaql_sql::SqlExecutorError<DataServiceMutationError>;
    fn capabilities(&self) -> teaql_data_service::DataServiceCapabilities {
        teaql_data_service::DataServiceExecutor::capabilities(&self.inner)
    }
}

impl teaql_data_service::QueryExecutor for ServiceRuntimeExecutor {
    async fn query(
        &self,
        request: teaql_data_service::QueryRequest,
    ) -> Result<teaql_data_service::QueryResult, Self::Error> {
        teaql_data_service::QueryExecutor::query(&self.inner, request).await
    }
}

impl teaql_data_service::StreamQueryExecutor for ServiceRuntimeExecutor {
    fn query_stream(
        &self,
        request: teaql_data_service::QueryRequest,
        chunk_size: usize,
    ) -> teaql_data_service::QueryStream<'_, Self::Error> {
        teaql_data_service::StreamQueryExecutor::query_stream(&self.inner, request, chunk_size)
    }
}

impl teaql_data_service::MutationExecutor for ServiceRuntimeExecutor {
    async fn mutate(
        &self,
        request: teaql_data_service::MutationRequest,
    ) -> Result<teaql_data_service::MutationResult, Self::Error> {
        teaql_data_service::MutationExecutor::mutate(&self.inner, request).await
    }
}

impl teaql_data_service::TransactionExecutor for ServiceRuntimeExecutor {
    type Tx<'a>
        = teaql_sql::SqlDataServiceTransaction<
        'a,
        DataServiceDialect,
        <DataServiceMutationExecutor as teaql_sql::SqlTransactionTransport>::Tx<'a>,
        LocalSchemaProvider,
    >
    where
        Self: 'a;

    async fn begin(&self) -> Result<Self::Tx<'_>, Self::Error> {
        teaql_data_service::TransactionExecutor::begin(&self.inner).await
    }
}

pub async fn service_runtime_from_env() -> Result<ServiceRuntime, ServiceRuntimeError> {
    service_runtime(ServiceRuntimeConfig::from_env()?).await
}

pub async fn service_runtime(
    config: ServiceRuntimeConfig,
) -> Result<ServiceRuntime, ServiceRuntimeError> {
    let pool = connect_data_service_pool(&config).await?;
    service_runtime_from_pool(pool).await
}

pub async fn service_runtime_from_pool(
    pool: DataServicePool,
) -> Result<ServiceRuntime, ServiceRuntimeError> {
    let id_generator = DataServiceIdGenerator::new(pool.clone());
    let mutation_executor = DataServiceMutationExecutor::new(pool);
    let mut context = module_with_behaviors_and_checkers().into_context();
    context.set_internal_id_generator(id_generator);
    context.use_postgres_provider(mutation_executor.clone());
    let executor = ServiceRuntimeExecutor::new(mutation_executor);
    context.register_executor(executor.clone());
    context.insert_resource(executor);

    // Load runtime configuration only. Schema installation is an explicit application action.
    let env_config = teaql_tool_core::audit_config_from_env(&[
        "platform_data",
        "tenant_data",
        "repository_type_data",
        "repository_format_data",
        "write_policy_data",
        "blob_store_type_data",
        "user_status_data",
        "blob_store_configuration_data",
        "repository_configuration_data",
        "content_repository_data",
        "component_data",
        "asset_blob_data",
        "asset_data",
        "security_user_data",
        "security_role_data",
        "security_privilege_data",
        "security_user_role_data",
        "security_role_privilege_data",
        "service_log_data",
    ]);
    context.insert_resource(env_config.config.clone());
    context.insert_resource(env_config);

    Ok(context)
}

fn env_value(name: &'static str) -> Result<String, ServiceRuntimeError> {
    std::env::var(name).map_err(|source| ServiceRuntimeError::MissingEnv { name, source })
}

async fn connect_data_service_pool(
    config: &ServiceRuntimeConfig,
) -> Result<DataServicePool, ServiceRuntimeError> {
    let pg_config = config
        .database_url
        .parse::<tokio_postgres::Config>()
        .map_err(|e| ServiceRuntimeError::ConnectionError(e.to_string()))?;
    let mgr = deadpool_postgres::Manager::new(pg_config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(mgr)
        .build()
        .map_err(|e| ServiceRuntimeError::ConnectionError(e.to_string()))?;
    Ok(pool)
}
pub fn repository_registry() -> teaql_runtime::InMemoryEntityRegistry {
    teaql_runtime::InMemoryEntityRegistry::new()
        .with_entity("Platform")
        .with_entity("Tenant")
        .with_entity("RepositoryType")
        .with_entity("RepositoryFormat")
        .with_entity("WritePolicy")
        .with_entity("BlobStoreType")
        .with_entity("UserStatus")
        .with_entity("BlobStoreConfiguration")
        .with_entity("RepositoryConfiguration")
        .with_entity("ContentRepository")
        .with_entity("Component")
        .with_entity("AssetBlob")
        .with_entity("Asset")
        .with_entity("SecurityUser")
        .with_entity("SecurityRole")
        .with_entity("SecurityPrivilege")
        .with_entity("SecurityUserRole")
        .with_entity("SecurityRolePrivilege")
        .with_entity("ServiceLog")
}

pub fn behavior_registry() -> teaql_runtime::InMemoryEntityDataServiceBehaviorRegistry {
    teaql_runtime::InMemoryEntityDataServiceBehaviorRegistry::new()
        .with_behavior("Platform", PlatformBehavior::default())
        .with_behavior("Tenant", TenantBehavior::default())
        .with_behavior("RepositoryType", RepositoryTypeBehavior::default())
        .with_behavior("RepositoryFormat", RepositoryFormatBehavior::default())
        .with_behavior("WritePolicy", WritePolicyBehavior::default())
        .with_behavior("BlobStoreType", BlobStoreTypeBehavior::default())
        .with_behavior("UserStatus", UserStatusBehavior::default())
        .with_behavior(
            "BlobStoreConfiguration",
            BlobStoreConfigurationBehavior::default(),
        )
        .with_behavior(
            "RepositoryConfiguration",
            RepositoryConfigurationBehavior::default(),
        )
        .with_behavior("ContentRepository", ContentRepositoryBehavior::default())
        .with_behavior("Component", ComponentBehavior::default())
        .with_behavior("AssetBlob", AssetBlobBehavior::default())
        .with_behavior("Asset", AssetBehavior::default())
        .with_behavior("SecurityUser", SecurityUserBehavior::default())
        .with_behavior("SecurityRole", SecurityRoleBehavior::default())
        .with_behavior("SecurityPrivilege", SecurityPrivilegeBehavior::default())
        .with_behavior("SecurityUserRole", SecurityUserRoleBehavior::default())
        .with_behavior(
            "SecurityRolePrivilege",
            SecurityRolePrivilegeBehavior::default(),
        )
        .with_behavior("ServiceLog", ServiceLogBehavior::default())
}

pub fn checker_registry() -> teaql_runtime::InMemoryCheckerRegistry {
    teaql_runtime::InMemoryCheckerRegistry::new()
        .with_checker(teaql_runtime::TypedEntityChecker::<Platform, _>::new(
            PlatformChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<Tenant, _>::new(
            TenantChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<RepositoryType, _>::new(
            RepositoryTypeChecker::default(),
        ))
        .with_checker(
            teaql_runtime::TypedEntityChecker::<RepositoryFormat, _>::new(
                RepositoryFormatChecker::default(),
            ),
        )
        .with_checker(teaql_runtime::TypedEntityChecker::<WritePolicy, _>::new(
            WritePolicyChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<BlobStoreType, _>::new(
            BlobStoreTypeChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<UserStatus, _>::new(
            UserStatusChecker::default(),
        ))
        .with_checker(
            teaql_runtime::TypedEntityChecker::<BlobStoreConfiguration, _>::new(
                BlobStoreConfigurationChecker::default(),
            ),
        )
        .with_checker(teaql_runtime::TypedEntityChecker::<
            RepositoryConfiguration,
            _,
        >::new(RepositoryConfigurationChecker::default()))
        .with_checker(
            teaql_runtime::TypedEntityChecker::<ContentRepository, _>::new(
                ContentRepositoryChecker::default(),
            ),
        )
        .with_checker(teaql_runtime::TypedEntityChecker::<Component, _>::new(
            ComponentChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<AssetBlob, _>::new(
            AssetBlobChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<Asset, _>::new(
            AssetChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<SecurityUser, _>::new(
            SecurityUserChecker::default(),
        ))
        .with_checker(teaql_runtime::TypedEntityChecker::<SecurityRole, _>::new(
            SecurityRoleChecker::default(),
        ))
        .with_checker(
            teaql_runtime::TypedEntityChecker::<SecurityPrivilege, _>::new(
                SecurityPrivilegeChecker::default(),
            ),
        )
        .with_checker(
            teaql_runtime::TypedEntityChecker::<SecurityUserRole, _>::new(
                SecurityUserRoleChecker::default(),
            ),
        )
        .with_checker(
            teaql_runtime::TypedEntityChecker::<SecurityRolePrivilege, _>::new(
                SecurityRolePrivilegeChecker::default(),
            ),
        )
        .with_checker(teaql_runtime::TypedEntityChecker::<ServiceLog, _>::new(
            ServiceLogChecker::default(),
        ))
}

fn ensure_generated_bootstrap<'a>(
    context: &'a teaql_runtime::UserContext,
) -> teaql_runtime::GeneratedSchemaBootstrapFuture<'a> {
    Box::pin(async move {
        use teaql_core::Entity as _;
        let root_rows = crate::Q::platforms()
            .select_self_fields()
            .with_id_is(1_u64)
            .comment("what: locate generated Domain Root")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        let domain_root = if let Some(entity) = root_rows.data.into_iter().next() {
            entity
        } else {
            let mut entity = Platform::runtime_new(context.entity_runtime_state());
            entity.update_id(1_u64);
            context.initialize_generated_bootstrap_entity(
                &mut entity,
                Platform::ENTITY_NAME,
                1_u64,
            )?;
            entity.update_name("TeaQL Registry Platform");
            entity.update_platform_version("1.0.0");
            teaql_runtime::AuditedSaveExt::save(
                entity.audit_as("create generated Domain Root Platform"),
                context,
            )
            .await?
        };
        context.set_generated_bootstrap_active_root(Platform::ENTITY_NAME, domain_root.id())?;
        let rows_constant_repository_type_1001 = crate::Q::repository_types()
            .select_self_fields()
            .with_id_is(1001_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_type_1001) =
            rows_constant_repository_type_1001.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_type_1001.platform_id() != 1_u64 {
                constant_repository_type_1001.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_type_1001.name() != "Hosted" {
                constant_repository_type_1001.update_name("Hosted");
                changed = true;
            }
            if constant_repository_type_1001.code() != "HOSTED" {
                constant_repository_type_1001.update_code("HOSTED");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_type_1001
                        .audit_as("reconcile model constant RepositoryType(1001)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_type_1001 =
                RepositoryType::runtime_new(context.entity_runtime_state());
            constant_repository_type_1001.update_id(1001_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_type_1001,
                RepositoryType::ENTITY_NAME,
                1001_u64,
            )?;
            constant_repository_type_1001.update_platform_id(1_u64);
            constant_repository_type_1001.update_name("Hosted");
            constant_repository_type_1001.update_code("HOSTED");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_type_1001
                    .audit_as("create model constant RepositoryType(1001)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_type_1002 = crate::Q::repository_types()
            .select_self_fields()
            .with_id_is(1002_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_type_1002) =
            rows_constant_repository_type_1002.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_type_1002.platform_id() != 1_u64 {
                constant_repository_type_1002.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_type_1002.name() != "Proxy" {
                constant_repository_type_1002.update_name("Proxy");
                changed = true;
            }
            if constant_repository_type_1002.code() != "PROXY" {
                constant_repository_type_1002.update_code("PROXY");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_type_1002
                        .audit_as("reconcile model constant RepositoryType(1002)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_type_1002 =
                RepositoryType::runtime_new(context.entity_runtime_state());
            constant_repository_type_1002.update_id(1002_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_type_1002,
                RepositoryType::ENTITY_NAME,
                1002_u64,
            )?;
            constant_repository_type_1002.update_platform_id(1_u64);
            constant_repository_type_1002.update_name("Proxy");
            constant_repository_type_1002.update_code("PROXY");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_type_1002
                    .audit_as("create model constant RepositoryType(1002)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_type_1003 = crate::Q::repository_types()
            .select_self_fields()
            .with_id_is(1003_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_type_1003) =
            rows_constant_repository_type_1003.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_type_1003.platform_id() != 1_u64 {
                constant_repository_type_1003.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_type_1003.name() != "Group" {
                constant_repository_type_1003.update_name("Group");
                changed = true;
            }
            if constant_repository_type_1003.code() != "GROUP" {
                constant_repository_type_1003.update_code("GROUP");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_type_1003
                        .audit_as("reconcile model constant RepositoryType(1003)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_type_1003 =
                RepositoryType::runtime_new(context.entity_runtime_state());
            constant_repository_type_1003.update_id(1003_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_type_1003,
                RepositoryType::ENTITY_NAME,
                1003_u64,
            )?;
            constant_repository_type_1003.update_platform_id(1_u64);
            constant_repository_type_1003.update_name("Group");
            constant_repository_type_1003.update_code("GROUP");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_type_1003
                    .audit_as("create model constant RepositoryType(1003)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1001 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1001_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1001) =
            rows_constant_repository_format_1001.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1001.platform_id() != 1_u64 {
                constant_repository_format_1001.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1001.name() != "Maven2" {
                constant_repository_format_1001.update_name("Maven2");
                changed = true;
            }
            if constant_repository_format_1001.code() != "MAVEN2" {
                constant_repository_format_1001.update_code("MAVEN2");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1001
                        .audit_as("reconcile model constant RepositoryFormat(1001)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1001 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1001.update_id(1001_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1001,
                RepositoryFormat::ENTITY_NAME,
                1001_u64,
            )?;
            constant_repository_format_1001.update_platform_id(1_u64);
            constant_repository_format_1001.update_name("Maven2");
            constant_repository_format_1001.update_code("MAVEN2");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1001
                    .audit_as("create model constant RepositoryFormat(1001)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1002 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1002_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1002) =
            rows_constant_repository_format_1002.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1002.platform_id() != 1_u64 {
                constant_repository_format_1002.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1002.name() != "Raw" {
                constant_repository_format_1002.update_name("Raw");
                changed = true;
            }
            if constant_repository_format_1002.code() != "RAW" {
                constant_repository_format_1002.update_code("RAW");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1002
                        .audit_as("reconcile model constant RepositoryFormat(1002)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1002 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1002.update_id(1002_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1002,
                RepositoryFormat::ENTITY_NAME,
                1002_u64,
            )?;
            constant_repository_format_1002.update_platform_id(1_u64);
            constant_repository_format_1002.update_name("Raw");
            constant_repository_format_1002.update_code("RAW");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1002
                    .audit_as("create model constant RepositoryFormat(1002)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1003 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1003_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1003) =
            rows_constant_repository_format_1003.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1003.platform_id() != 1_u64 {
                constant_repository_format_1003.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1003.name() != "Docker" {
                constant_repository_format_1003.update_name("Docker");
                changed = true;
            }
            if constant_repository_format_1003.code() != "DOCKER" {
                constant_repository_format_1003.update_code("DOCKER");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1003
                        .audit_as("reconcile model constant RepositoryFormat(1003)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1003 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1003.update_id(1003_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1003,
                RepositoryFormat::ENTITY_NAME,
                1003_u64,
            )?;
            constant_repository_format_1003.update_platform_id(1_u64);
            constant_repository_format_1003.update_name("Docker");
            constant_repository_format_1003.update_code("DOCKER");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1003
                    .audit_as("create model constant RepositoryFormat(1003)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1004 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1004_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1004) =
            rows_constant_repository_format_1004.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1004.platform_id() != 1_u64 {
                constant_repository_format_1004.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1004.name() != "npm" {
                constant_repository_format_1004.update_name("npm");
                changed = true;
            }
            if constant_repository_format_1004.code() != "NPM" {
                constant_repository_format_1004.update_code("NPM");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1004
                        .audit_as("reconcile model constant RepositoryFormat(1004)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1004 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1004.update_id(1004_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1004,
                RepositoryFormat::ENTITY_NAME,
                1004_u64,
            )?;
            constant_repository_format_1004.update_platform_id(1_u64);
            constant_repository_format_1004.update_name("npm");
            constant_repository_format_1004.update_code("NPM");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1004
                    .audit_as("create model constant RepositoryFormat(1004)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1005 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1005_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1005) =
            rows_constant_repository_format_1005.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1005.platform_id() != 1_u64 {
                constant_repository_format_1005.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1005.name() != "PyPI" {
                constant_repository_format_1005.update_name("PyPI");
                changed = true;
            }
            if constant_repository_format_1005.code() != "PYPI" {
                constant_repository_format_1005.update_code("PYPI");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1005
                        .audit_as("reconcile model constant RepositoryFormat(1005)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1005 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1005.update_id(1005_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1005,
                RepositoryFormat::ENTITY_NAME,
                1005_u64,
            )?;
            constant_repository_format_1005.update_platform_id(1_u64);
            constant_repository_format_1005.update_name("PyPI");
            constant_repository_format_1005.update_code("PYPI");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1005
                    .audit_as("create model constant RepositoryFormat(1005)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1006 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1006_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1006) =
            rows_constant_repository_format_1006.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1006.platform_id() != 1_u64 {
                constant_repository_format_1006.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1006.name() != "Go Modules" {
                constant_repository_format_1006.update_name("Go Modules");
                changed = true;
            }
            if constant_repository_format_1006.code() != "GOMOD" {
                constant_repository_format_1006.update_code("GOMOD");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1006
                        .audit_as("reconcile model constant RepositoryFormat(1006)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1006 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1006.update_id(1006_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1006,
                RepositoryFormat::ENTITY_NAME,
                1006_u64,
            )?;
            constant_repository_format_1006.update_platform_id(1_u64);
            constant_repository_format_1006.update_name("Go Modules");
            constant_repository_format_1006.update_code("GOMOD");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1006
                    .audit_as("create model constant RepositoryFormat(1006)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1007 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1007_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1007) =
            rows_constant_repository_format_1007.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1007.platform_id() != 1_u64 {
                constant_repository_format_1007.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1007.name() != "Cargo" {
                constant_repository_format_1007.update_name("Cargo");
                changed = true;
            }
            if constant_repository_format_1007.code() != "CARGO" {
                constant_repository_format_1007.update_code("CARGO");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1007
                        .audit_as("reconcile model constant RepositoryFormat(1007)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1007 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1007.update_id(1007_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1007,
                RepositoryFormat::ENTITY_NAME,
                1007_u64,
            )?;
            constant_repository_format_1007.update_platform_id(1_u64);
            constant_repository_format_1007.update_name("Cargo");
            constant_repository_format_1007.update_code("CARGO");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1007
                    .audit_as("create model constant RepositoryFormat(1007)"),
                context,
            )
            .await?;
        }
        let rows_constant_repository_format_1008 = crate::Q::repository_formats()
            .select_self_fields()
            .with_id_is(1008_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_repository_format_1008) =
            rows_constant_repository_format_1008.data.into_iter().next()
        {
            let mut changed = false;
            if constant_repository_format_1008.platform_id() != 1_u64 {
                constant_repository_format_1008.update_platform_id(1_u64);
                changed = true;
            }
            if constant_repository_format_1008.name() != "NuGet" {
                constant_repository_format_1008.update_name("NuGet");
                changed = true;
            }
            if constant_repository_format_1008.code() != "NUGET" {
                constant_repository_format_1008.update_code("NUGET");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_repository_format_1008
                        .audit_as("reconcile model constant RepositoryFormat(1008)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_repository_format_1008 =
                RepositoryFormat::runtime_new(context.entity_runtime_state());
            constant_repository_format_1008.update_id(1008_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_repository_format_1008,
                RepositoryFormat::ENTITY_NAME,
                1008_u64,
            )?;
            constant_repository_format_1008.update_platform_id(1_u64);
            constant_repository_format_1008.update_name("NuGet");
            constant_repository_format_1008.update_code("NUGET");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_repository_format_1008
                    .audit_as("create model constant RepositoryFormat(1008)"),
                context,
            )
            .await?;
        }
        let rows_constant_write_policy_1001 = crate::Q::write_policies()
            .select_self_fields()
            .with_id_is(1001_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_write_policy_1001) =
            rows_constant_write_policy_1001.data.into_iter().next()
        {
            let mut changed = false;
            if constant_write_policy_1001.platform_id() != 1_u64 {
                constant_write_policy_1001.update_platform_id(1_u64);
                changed = true;
            }
            if constant_write_policy_1001.name() != "Allow Write" {
                constant_write_policy_1001.update_name("Allow Write");
                changed = true;
            }
            if constant_write_policy_1001.code() != "ALLOW_WRITE" {
                constant_write_policy_1001.update_code("ALLOW_WRITE");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_write_policy_1001
                        .audit_as("reconcile model constant WritePolicy(1001)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_write_policy_1001 =
                WritePolicy::runtime_new(context.entity_runtime_state());
            constant_write_policy_1001.update_id(1001_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_write_policy_1001,
                WritePolicy::ENTITY_NAME,
                1001_u64,
            )?;
            constant_write_policy_1001.update_platform_id(1_u64);
            constant_write_policy_1001.update_name("Allow Write");
            constant_write_policy_1001.update_code("ALLOW_WRITE");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_write_policy_1001.audit_as("create model constant WritePolicy(1001)"),
                context,
            )
            .await?;
        }
        let rows_constant_write_policy_1002 = crate::Q::write_policies()
            .select_self_fields()
            .with_id_is(1002_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_write_policy_1002) =
            rows_constant_write_policy_1002.data.into_iter().next()
        {
            let mut changed = false;
            if constant_write_policy_1002.platform_id() != 1_u64 {
                constant_write_policy_1002.update_platform_id(1_u64);
                changed = true;
            }
            if constant_write_policy_1002.name() != "Allow Once" {
                constant_write_policy_1002.update_name("Allow Once");
                changed = true;
            }
            if constant_write_policy_1002.code() != "ALLOW_ONCE" {
                constant_write_policy_1002.update_code("ALLOW_ONCE");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_write_policy_1002
                        .audit_as("reconcile model constant WritePolicy(1002)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_write_policy_1002 =
                WritePolicy::runtime_new(context.entity_runtime_state());
            constant_write_policy_1002.update_id(1002_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_write_policy_1002,
                WritePolicy::ENTITY_NAME,
                1002_u64,
            )?;
            constant_write_policy_1002.update_platform_id(1_u64);
            constant_write_policy_1002.update_name("Allow Once");
            constant_write_policy_1002.update_code("ALLOW_ONCE");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_write_policy_1002.audit_as("create model constant WritePolicy(1002)"),
                context,
            )
            .await?;
        }
        let rows_constant_write_policy_1003 = crate::Q::write_policies()
            .select_self_fields()
            .with_id_is(1003_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_write_policy_1003) =
            rows_constant_write_policy_1003.data.into_iter().next()
        {
            let mut changed = false;
            if constant_write_policy_1003.platform_id() != 1_u64 {
                constant_write_policy_1003.update_platform_id(1_u64);
                changed = true;
            }
            if constant_write_policy_1003.name() != "Read Only" {
                constant_write_policy_1003.update_name("Read Only");
                changed = true;
            }
            if constant_write_policy_1003.code() != "READ_ONLY" {
                constant_write_policy_1003.update_code("READ_ONLY");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_write_policy_1003
                        .audit_as("reconcile model constant WritePolicy(1003)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_write_policy_1003 =
                WritePolicy::runtime_new(context.entity_runtime_state());
            constant_write_policy_1003.update_id(1003_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_write_policy_1003,
                WritePolicy::ENTITY_NAME,
                1003_u64,
            )?;
            constant_write_policy_1003.update_platform_id(1_u64);
            constant_write_policy_1003.update_name("Read Only");
            constant_write_policy_1003.update_code("READ_ONLY");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_write_policy_1003.audit_as("create model constant WritePolicy(1003)"),
                context,
            )
            .await?;
        }
        let rows_constant_blob_store_type_1001 = crate::Q::blob_store_types()
            .select_self_fields()
            .with_id_is(1001_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_blob_store_type_1001) =
            rows_constant_blob_store_type_1001.data.into_iter().next()
        {
            let mut changed = false;
            if constant_blob_store_type_1001.platform_id() != 1_u64 {
                constant_blob_store_type_1001.update_platform_id(1_u64);
                changed = true;
            }
            if constant_blob_store_type_1001.name() != "File" {
                constant_blob_store_type_1001.update_name("File");
                changed = true;
            }
            if constant_blob_store_type_1001.code() != "FILE" {
                constant_blob_store_type_1001.update_code("FILE");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_blob_store_type_1001
                        .audit_as("reconcile model constant BlobStoreType(1001)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_blob_store_type_1001 =
                BlobStoreType::runtime_new(context.entity_runtime_state());
            constant_blob_store_type_1001.update_id(1001_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_blob_store_type_1001,
                BlobStoreType::ENTITY_NAME,
                1001_u64,
            )?;
            constant_blob_store_type_1001.update_platform_id(1_u64);
            constant_blob_store_type_1001.update_name("File");
            constant_blob_store_type_1001.update_code("FILE");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_blob_store_type_1001.audit_as("create model constant BlobStoreType(1001)"),
                context,
            )
            .await?;
        }
        let rows_constant_blob_store_type_1002 = crate::Q::blob_store_types()
            .select_self_fields()
            .with_id_is(1002_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_blob_store_type_1002) =
            rows_constant_blob_store_type_1002.data.into_iter().next()
        {
            let mut changed = false;
            if constant_blob_store_type_1002.platform_id() != 1_u64 {
                constant_blob_store_type_1002.update_platform_id(1_u64);
                changed = true;
            }
            if constant_blob_store_type_1002.name() != "S3" {
                constant_blob_store_type_1002.update_name("S3");
                changed = true;
            }
            if constant_blob_store_type_1002.code() != "S3" {
                constant_blob_store_type_1002.update_code("S3");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_blob_store_type_1002
                        .audit_as("reconcile model constant BlobStoreType(1002)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_blob_store_type_1002 =
                BlobStoreType::runtime_new(context.entity_runtime_state());
            constant_blob_store_type_1002.update_id(1002_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_blob_store_type_1002,
                BlobStoreType::ENTITY_NAME,
                1002_u64,
            )?;
            constant_blob_store_type_1002.update_platform_id(1_u64);
            constant_blob_store_type_1002.update_name("S3");
            constant_blob_store_type_1002.update_code("S3");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_blob_store_type_1002.audit_as("create model constant BlobStoreType(1002)"),
                context,
            )
            .await?;
        }
        let rows_constant_user_status_1001 = crate::Q::user_statuses()
            .select_self_fields()
            .with_id_is(1001_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_user_status_1001) =
            rows_constant_user_status_1001.data.into_iter().next()
        {
            let mut changed = false;
            if constant_user_status_1001.platform_id() != 1_u64 {
                constant_user_status_1001.update_platform_id(1_u64);
                changed = true;
            }
            if constant_user_status_1001.name() != "Active" {
                constant_user_status_1001.update_name("Active");
                changed = true;
            }
            if constant_user_status_1001.code() != "ACTIVE" {
                constant_user_status_1001.update_code("ACTIVE");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_user_status_1001.audit_as("reconcile model constant UserStatus(1001)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_user_status_1001 =
                UserStatus::runtime_new(context.entity_runtime_state());
            constant_user_status_1001.update_id(1001_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_user_status_1001,
                UserStatus::ENTITY_NAME,
                1001_u64,
            )?;
            constant_user_status_1001.update_platform_id(1_u64);
            constant_user_status_1001.update_name("Active");
            constant_user_status_1001.update_code("ACTIVE");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_user_status_1001.audit_as("create model constant UserStatus(1001)"),
                context,
            )
            .await?;
        }
        let rows_constant_user_status_1002 = crate::Q::user_statuses()
            .select_self_fields()
            .with_id_is(1002_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_user_status_1002) =
            rows_constant_user_status_1002.data.into_iter().next()
        {
            let mut changed = false;
            if constant_user_status_1002.platform_id() != 1_u64 {
                constant_user_status_1002.update_platform_id(1_u64);
                changed = true;
            }
            if constant_user_status_1002.name() != "Disabled" {
                constant_user_status_1002.update_name("Disabled");
                changed = true;
            }
            if constant_user_status_1002.code() != "DISABLED" {
                constant_user_status_1002.update_code("DISABLED");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_user_status_1002.audit_as("reconcile model constant UserStatus(1002)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_user_status_1002 =
                UserStatus::runtime_new(context.entity_runtime_state());
            constant_user_status_1002.update_id(1002_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_user_status_1002,
                UserStatus::ENTITY_NAME,
                1002_u64,
            )?;
            constant_user_status_1002.update_platform_id(1_u64);
            constant_user_status_1002.update_name("Disabled");
            constant_user_status_1002.update_code("DISABLED");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_user_status_1002.audit_as("create model constant UserStatus(1002)"),
                context,
            )
            .await?;
        }
        let rows_constant_user_status_1003 = crate::Q::user_statuses()
            .select_self_fields()
            .with_id_is(1003_u64)
            .comment("what: locate generated constant")
            .purpose("why: idempotent runtime bootstrap")
            .execute_for_list(context)
            .await
            .map_err(|e| teaql_runtime::RuntimeError::Graph(e.to_string()))?;
        if let Some(mut constant_user_status_1003) =
            rows_constant_user_status_1003.data.into_iter().next()
        {
            let mut changed = false;
            if constant_user_status_1003.platform_id() != 1_u64 {
                constant_user_status_1003.update_platform_id(1_u64);
                changed = true;
            }
            if constant_user_status_1003.name() != "Locked" {
                constant_user_status_1003.update_name("Locked");
                changed = true;
            }
            if constant_user_status_1003.code() != "LOCKED" {
                constant_user_status_1003.update_code("LOCKED");
                changed = true;
            }
            if changed {
                let _ = teaql_runtime::AuditedSaveExt::save(
                    constant_user_status_1003.audit_as("reconcile model constant UserStatus(1003)"),
                    context,
                )
                .await?;
            }
        } else {
            let mut constant_user_status_1003 =
                UserStatus::runtime_new(context.entity_runtime_state());
            constant_user_status_1003.update_id(1003_u64);
            context.initialize_generated_bootstrap_entity(
                &mut constant_user_status_1003,
                UserStatus::ENTITY_NAME,
                1003_u64,
            )?;
            constant_user_status_1003.update_platform_id(1_u64);
            constant_user_status_1003.update_name("Locked");
            constant_user_status_1003.update_code("LOCKED");
            let _ = teaql_runtime::AuditedSaveExt::save(
                constant_user_status_1003.audit_as("create model constant UserStatus(1003)"),
                context,
            )
            .await?;
        }
        Ok(())
    })
}

/// Canonical KSML field to selected JSON wire name, consumed by HTTP/TFP adapters.
pub fn generated_wire_field_mappings(
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> {
    std::collections::BTreeMap::from([
        (
            "Platform".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("platform_version".to_owned(), "platformVersion".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "Tenant".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("platform".to_owned(), "platform".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("code".to_owned(), "code".to_owned()),
                ("description".to_owned(), "description".to_owned()),
                ("enabled".to_owned(), "enabled".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "RepositoryType".to_owned(),
            std::collections::BTreeMap::from([
                ("platform".to_owned(), "platform".to_owned()),
                ("id".to_owned(), "id".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("code".to_owned(), "code".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "RepositoryFormat".to_owned(),
            std::collections::BTreeMap::from([
                ("platform".to_owned(), "platform".to_owned()),
                ("id".to_owned(), "id".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("code".to_owned(), "code".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "WritePolicy".to_owned(),
            std::collections::BTreeMap::from([
                ("platform".to_owned(), "platform".to_owned()),
                ("id".to_owned(), "id".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("code".to_owned(), "code".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "BlobStoreType".to_owned(),
            std::collections::BTreeMap::from([
                ("platform".to_owned(), "platform".to_owned()),
                ("id".to_owned(), "id".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("code".to_owned(), "code".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "UserStatus".to_owned(),
            std::collections::BTreeMap::from([
                ("platform".to_owned(), "platform".to_owned()),
                ("id".to_owned(), "id".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("code".to_owned(), "code".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "BlobStoreConfiguration".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("blob_store_type".to_owned(), "blobStoreType".to_owned()),
                ("path".to_owned(), "path".to_owned()),
                ("total_size".to_owned(), "totalSize".to_owned()),
                ("blob_count".to_owned(), "blobCount".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "RepositoryConfiguration".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("recipe_name".to_owned(), "recipeName".to_owned()),
                ("repository_type".to_owned(), "repositoryType".to_owned()),
                (
                    "repository_format".to_owned(),
                    "repositoryFormat".to_owned(),
                ),
                ("write_policy".to_owned(), "writePolicy".to_owned()),
                ("blob_store".to_owned(), "blobStore".to_owned()),
                ("online".to_owned(), "online".to_owned()),
                ("remote_url".to_owned(), "remoteUrl".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "ContentRepository".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("repository_id".to_owned(), "repositoryId".to_owned()),
                ("format_name".to_owned(), "formatName".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "Component".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                (
                    "content_repository".to_owned(),
                    "contentRepository".to_owned(),
                ),
                ("namespace".to_owned(), "namespace".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("version_name".to_owned(), "versionName".to_owned()),
                (
                    "normalized_version".to_owned(),
                    "normalizedVersion".to_owned(),
                ),
                ("kind".to_owned(), "kind".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "AssetBlob".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("blob_store".to_owned(), "blobStore".to_owned()),
                ("blob_ref".to_owned(), "blobRef".to_owned()),
                ("blob_size".to_owned(), "blobSize".to_owned()),
                ("content_type".to_owned(), "contentType".to_owned()),
                ("sha1_checksum".to_owned(), "sha1Checksum".to_owned()),
                ("sha256_checksum".to_owned(), "sha256Checksum".to_owned()),
                ("md5_checksum".to_owned(), "md5Checksum".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "Asset".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                (
                    "content_repository".to_owned(),
                    "contentRepository".to_owned(),
                ),
                ("component_id".to_owned(), "componentId".to_owned()),
                ("asset_blob".to_owned(), "assetBlob".to_owned()),
                ("path".to_owned(), "path".to_owned()),
                ("kind".to_owned(), "kind".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "SecurityUser".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("username".to_owned(), "username".to_owned()),
                ("first_name".to_owned(), "firstName".to_owned()),
                ("last_name".to_owned(), "lastName".to_owned()),
                ("password_hash".to_owned(), "passwordHash".to_owned()),
                ("user_status".to_owned(), "userStatus".to_owned()),
                ("email".to_owned(), "email".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "SecurityRole".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("role_id".to_owned(), "roleId".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("description".to_owned(), "description".to_owned()),
                ("read_only".to_owned(), "readOnly".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "SecurityPrivilege".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("privilege_id".to_owned(), "privilegeId".to_owned()),
                ("name".to_owned(), "name".to_owned()),
                ("description".to_owned(), "description".to_owned()),
                ("privilege_type".to_owned(), "privilegeType".to_owned()),
                (
                    "permission_pattern".to_owned(),
                    "permissionPattern".to_owned(),
                ),
                ("read_only".to_owned(), "readOnly".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "SecurityUserRole".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("security_user".to_owned(), "securityUser".to_owned()),
                ("security_role".to_owned(), "securityRole".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "SecurityRolePrivilege".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("security_role".to_owned(), "securityRole".to_owned()),
                (
                    "security_privilege".to_owned(),
                    "securityPrivilege".to_owned(),
                ),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
        (
            "ServiceLog".to_owned(),
            std::collections::BTreeMap::from([
                ("id".to_owned(), "id".to_owned()),
                ("tenant".to_owned(), "tenant".to_owned()),
                ("event_time".to_owned(), "eventTime".to_owned()),
                ("log_type".to_owned(), "logType".to_owned()),
                ("operator_id".to_owned(), "operatorId".to_owned()),
                ("operator_name".to_owned(), "operatorName".to_owned()),
                ("client_ip".to_owned(), "clientIp".to_owned()),
                ("action".to_owned(), "action".to_owned()),
                ("repository_name".to_owned(), "repositoryName".to_owned()),
                ("artifact_path".to_owned(), "artifactPath".to_owned()),
                ("format_name".to_owned(), "formatName".to_owned()),
                ("content_size".to_owned(), "contentSize".to_owned()),
                ("status".to_owned(), "status".to_owned()),
                ("error_message".to_owned(), "errorMessage".to_owned()),
                ("version".to_owned(), "version".to_owned()),
            ]),
        ),
    ])
}

/// Accepted legacy aliases; empty until explicitly declared by the model.
pub fn generated_wire_field_aliases(
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>> {
    std::collections::BTreeMap::new()
}

pub fn module() -> teaql_runtime::RuntimeModule {
    teaql_runtime::RuntimeModule::new()
        .entity::<Platform>()
        .entity::<Tenant>()
        .entity::<RepositoryType>()
        .entity::<RepositoryFormat>()
        .entity::<WritePolicy>()
        .entity::<BlobStoreType>()
        .entity::<UserStatus>()
        .entity::<BlobStoreConfiguration>()
        .entity::<RepositoryConfiguration>()
        .entity::<ContentRepository>()
        .entity::<Component>()
        .entity::<AssetBlob>()
        .entity::<Asset>()
        .entity::<SecurityUser>()
        .entity::<SecurityRole>()
        .entity::<SecurityPrivilege>()
        .entity::<SecurityUserRole>()
        .entity::<SecurityRolePrivilege>()
        .entity::<ServiceLog>()
        .generated_schema_bootstrap(ensure_generated_bootstrap)
}

pub fn module_with_checkers() -> teaql_runtime::RuntimeModule {
    let mut module = teaql_runtime::RuntimeModule::new();
    module = module.entity::<Platform>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<Platform, _>::new(
        PlatformChecker::default(),
    ));
    module = module.entity::<Tenant>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<Tenant, _>::new(
        TenantChecker::default(),
    ));
    module = module.entity::<RepositoryType>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<RepositoryType, _>::new(
        RepositoryTypeChecker::default(),
    ));
    module = module.entity::<RepositoryFormat>();
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<RepositoryFormat, _>::new(
            RepositoryFormatChecker::default(),
        ),
    );
    module = module.entity::<WritePolicy>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<WritePolicy, _>::new(
        WritePolicyChecker::default(),
    ));
    module = module.entity::<BlobStoreType>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<BlobStoreType, _>::new(
        BlobStoreTypeChecker::default(),
    ));
    module = module.entity::<UserStatus>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<UserStatus, _>::new(
        UserStatusChecker::default(),
    ));
    module = module.entity::<BlobStoreConfiguration>();
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<BlobStoreConfiguration, _>::new(
            BlobStoreConfigurationChecker::default(),
        ),
    );
    module = module.entity::<RepositoryConfiguration>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<
        RepositoryConfiguration,
        _,
    >::new(RepositoryConfigurationChecker::default()));
    module = module.entity::<ContentRepository>();
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<ContentRepository, _>::new(
            ContentRepositoryChecker::default(),
        ),
    );
    module = module.entity::<Component>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<Component, _>::new(
        ComponentChecker::default(),
    ));
    module = module.entity::<AssetBlob>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<AssetBlob, _>::new(
        AssetBlobChecker::default(),
    ));
    module = module.entity::<Asset>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<Asset, _>::new(
        AssetChecker::default(),
    ));
    module = module.entity::<SecurityUser>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<SecurityUser, _>::new(
        SecurityUserChecker::default(),
    ));
    module = module.entity::<SecurityRole>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<SecurityRole, _>::new(
        SecurityRoleChecker::default(),
    ));
    module = module.entity::<SecurityPrivilege>();
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<SecurityPrivilege, _>::new(
            SecurityPrivilegeChecker::default(),
        ),
    );
    module = module.entity::<SecurityUserRole>();
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<SecurityUserRole, _>::new(
            SecurityUserRoleChecker::default(),
        ),
    );
    module = module.entity::<SecurityRolePrivilege>();
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<SecurityRolePrivilege, _>::new(
            SecurityRolePrivilegeChecker::default(),
        ),
    );
    module = module.entity::<ServiceLog>();
    module = module.checker(teaql_runtime::TypedEntityChecker::<ServiceLog, _>::new(
        ServiceLogChecker::default(),
    ));
    module = module.generated_schema_bootstrap(ensure_generated_bootstrap);
    module
}

pub fn module_with_behaviors() -> teaql_runtime::RuntimeModule {
    let mut module = teaql_runtime::RuntimeModule::new();
    module = module.entity_with_behavior::<Platform, _>(PlatformBehavior::default());
    module = module.entity_with_behavior::<Tenant, _>(TenantBehavior::default());
    module = module.entity_with_behavior::<RepositoryType, _>(RepositoryTypeBehavior::default());
    module =
        module.entity_with_behavior::<RepositoryFormat, _>(RepositoryFormatBehavior::default());
    module = module.entity_with_behavior::<WritePolicy, _>(WritePolicyBehavior::default());
    module = module.entity_with_behavior::<BlobStoreType, _>(BlobStoreTypeBehavior::default());
    module = module.entity_with_behavior::<UserStatus, _>(UserStatusBehavior::default());
    module = module.entity_with_behavior::<BlobStoreConfiguration, _>(
        BlobStoreConfigurationBehavior::default(),
    );
    module = module.entity_with_behavior::<RepositoryConfiguration, _>(
        RepositoryConfigurationBehavior::default(),
    );
    module =
        module.entity_with_behavior::<ContentRepository, _>(ContentRepositoryBehavior::default());
    module = module.entity_with_behavior::<Component, _>(ComponentBehavior::default());
    module = module.entity_with_behavior::<AssetBlob, _>(AssetBlobBehavior::default());
    module = module.entity_with_behavior::<Asset, _>(AssetBehavior::default());
    module = module.entity_with_behavior::<SecurityUser, _>(SecurityUserBehavior::default());
    module = module.entity_with_behavior::<SecurityRole, _>(SecurityRoleBehavior::default());
    module =
        module.entity_with_behavior::<SecurityPrivilege, _>(SecurityPrivilegeBehavior::default());
    module =
        module.entity_with_behavior::<SecurityUserRole, _>(SecurityUserRoleBehavior::default());
    module = module
        .entity_with_behavior::<SecurityRolePrivilege, _>(SecurityRolePrivilegeBehavior::default());
    module = module.entity_with_behavior::<ServiceLog, _>(ServiceLogBehavior::default());
    module = module.generated_schema_bootstrap(ensure_generated_bootstrap);
    module
}

pub fn module_with_behaviors_and_checkers() -> teaql_runtime::RuntimeModule {
    let mut module = teaql_runtime::RuntimeModule::new();
    module = module.entity_with_behavior::<Platform, _>(PlatformBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<Platform, _>::new(
        PlatformChecker::default(),
    ));
    module = module.entity_with_behavior::<Tenant, _>(TenantBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<Tenant, _>::new(
        TenantChecker::default(),
    ));
    module = module.entity_with_behavior::<RepositoryType, _>(RepositoryTypeBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<RepositoryType, _>::new(
        RepositoryTypeChecker::default(),
    ));
    module =
        module.entity_with_behavior::<RepositoryFormat, _>(RepositoryFormatBehavior::default());
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<RepositoryFormat, _>::new(
            RepositoryFormatChecker::default(),
        ),
    );
    module = module.entity_with_behavior::<WritePolicy, _>(WritePolicyBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<WritePolicy, _>::new(
        WritePolicyChecker::default(),
    ));
    module = module.entity_with_behavior::<BlobStoreType, _>(BlobStoreTypeBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<BlobStoreType, _>::new(
        BlobStoreTypeChecker::default(),
    ));
    module = module.entity_with_behavior::<UserStatus, _>(UserStatusBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<UserStatus, _>::new(
        UserStatusChecker::default(),
    ));
    module = module.entity_with_behavior::<BlobStoreConfiguration, _>(
        BlobStoreConfigurationBehavior::default(),
    );
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<BlobStoreConfiguration, _>::new(
            BlobStoreConfigurationChecker::default(),
        ),
    );
    module = module.entity_with_behavior::<RepositoryConfiguration, _>(
        RepositoryConfigurationBehavior::default(),
    );
    module = module.checker(teaql_runtime::TypedEntityChecker::<
        RepositoryConfiguration,
        _,
    >::new(RepositoryConfigurationChecker::default()));
    module =
        module.entity_with_behavior::<ContentRepository, _>(ContentRepositoryBehavior::default());
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<ContentRepository, _>::new(
            ContentRepositoryChecker::default(),
        ),
    );
    module = module.entity_with_behavior::<Component, _>(ComponentBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<Component, _>::new(
        ComponentChecker::default(),
    ));
    module = module.entity_with_behavior::<AssetBlob, _>(AssetBlobBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<AssetBlob, _>::new(
        AssetBlobChecker::default(),
    ));
    module = module.entity_with_behavior::<Asset, _>(AssetBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<Asset, _>::new(
        AssetChecker::default(),
    ));
    module = module.entity_with_behavior::<SecurityUser, _>(SecurityUserBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<SecurityUser, _>::new(
        SecurityUserChecker::default(),
    ));
    module = module.entity_with_behavior::<SecurityRole, _>(SecurityRoleBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<SecurityRole, _>::new(
        SecurityRoleChecker::default(),
    ));
    module =
        module.entity_with_behavior::<SecurityPrivilege, _>(SecurityPrivilegeBehavior::default());
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<SecurityPrivilege, _>::new(
            SecurityPrivilegeChecker::default(),
        ),
    );
    module =
        module.entity_with_behavior::<SecurityUserRole, _>(SecurityUserRoleBehavior::default());
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<SecurityUserRole, _>::new(
            SecurityUserRoleChecker::default(),
        ),
    );
    module = module
        .entity_with_behavior::<SecurityRolePrivilege, _>(SecurityRolePrivilegeBehavior::default());
    module = module.checker(
        teaql_runtime::TypedEntityChecker::<SecurityRolePrivilege, _>::new(
            SecurityRolePrivilegeChecker::default(),
        ),
    );
    module = module.entity_with_behavior::<ServiceLog, _>(ServiceLogBehavior::default());
    module = module.checker(teaql_runtime::TypedEntityChecker::<ServiceLog, _>::new(
        ServiceLogChecker::default(),
    ));
    module = module.generated_schema_bootstrap(ensure_generated_bootstrap);
    module
}
