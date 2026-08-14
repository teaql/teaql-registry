
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

pub const DATABASE_URL_ENV: &str = "NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_URL";
pub const DATABASE_USER_ENV: &str = "NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_USER";
pub const DATABASE_PASSWORD_ENV: &str = "NEXUS_REPOSITORY_SERVICE_CORE_DATABASE_PASSWORD";
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
            "RepositoryType" => Some(std::sync::Arc::new(crate::RepositoryType::entity_descriptor())),
            "RepositoryFormat" => Some(std::sync::Arc::new(crate::RepositoryFormat::entity_descriptor())),
            "WritePolicy" => Some(std::sync::Arc::new(crate::WritePolicy::entity_descriptor())),
            "BlobStoreType" => Some(std::sync::Arc::new(crate::BlobStoreType::entity_descriptor())),
            "UserStatus" => Some(std::sync::Arc::new(crate::UserStatus::entity_descriptor())),
            "BlobStoreConfiguration" => Some(std::sync::Arc::new(crate::BlobStoreConfiguration::entity_descriptor())),
            "RepositoryConfiguration" => Some(std::sync::Arc::new(crate::RepositoryConfiguration::entity_descriptor())),
            "ContentRepository" => Some(std::sync::Arc::new(crate::ContentRepository::entity_descriptor())),
            "Component" => Some(std::sync::Arc::new(crate::Component::entity_descriptor())),
            "AssetBlob" => Some(std::sync::Arc::new(crate::AssetBlob::entity_descriptor())),
            "Asset" => Some(std::sync::Arc::new(crate::Asset::entity_descriptor())),
            "SecurityUser" => Some(std::sync::Arc::new(crate::SecurityUser::entity_descriptor())),
            "SecurityRole" => Some(std::sync::Arc::new(crate::SecurityRole::entity_descriptor())),
            "SecurityPrivilege" => Some(std::sync::Arc::new(crate::SecurityPrivilege::entity_descriptor())),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ServiceRuntimeExecutor {
    inner: teaql_sql::SqlDataServiceExecutor<
        DataServiceDialect,
        DataServiceMutationExecutor,
        LocalSchemaProvider
    >,
}

impl ServiceRuntimeExecutor {
    pub fn new(inner: DataServiceMutationExecutor) -> Self {
        Self {
            inner: teaql_sql::SqlDataServiceExecutor::new(
                DataServiceDialect::default(),
                inner,
                LocalSchemaProvider
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
    async fn query(&self, request: teaql_data_service::QueryRequest) -> Result<teaql_data_service::QueryResult, Self::Error> {
        teaql_data_service::QueryExecutor::query(&self.inner, request).await
    }
}

impl teaql_data_service::StreamQueryExecutor for ServiceRuntimeExecutor {
    fn query_stream(&self, request: teaql_data_service::QueryRequest, chunk_size: usize) -> teaql_data_service::QueryStream<'_, Self::Error> {
        teaql_data_service::StreamQueryExecutor::query_stream(&self.inner, request, chunk_size)
    }
}

impl teaql_data_service::MutationExecutor for ServiceRuntimeExecutor {
    async fn mutate(&self, request: teaql_data_service::MutationRequest) -> Result<teaql_data_service::MutationResult, Self::Error> {
        teaql_data_service::MutationExecutor::mutate(&self.inner, request).await
    }
}

impl teaql_data_service::TransactionExecutor for ServiceRuntimeExecutor {
    type Tx<'a> = teaql_sql::SqlDataServiceTransaction<'a, DataServiceDialect, <DataServiceMutationExecutor as teaql_sql::SqlTransactionTransport>::Tx<'a>, LocalSchemaProvider> where Self: 'a;

    async fn begin(&self) -> Result<Self::Tx<'_ >, Self::Error> {
        teaql_data_service::TransactionExecutor::begin(&self.inner).await
    }
}

pub async fn service_runtime_from_env() -> Result<ServiceRuntime, ServiceRuntimeError> {
    service_runtime(ServiceRuntimeConfig::from_env()?).await
}

pub async fn service_runtime(config: ServiceRuntimeConfig) -> Result<ServiceRuntime, ServiceRuntimeError> {
    let pool = connect_data_service_pool(&config).await?;
    service_runtime_from_pool(pool).await
}

pub async fn service_runtime_from_pool(pool: DataServicePool) -> Result<ServiceRuntime, ServiceRuntimeError> {
    let id_generator = DataServiceIdGenerator::new(pool.clone());
    let mutation_executor = DataServiceMutationExecutor::new(pool);let mut context = module_with_behaviors_and_checkers().into_context();
    context.set_internal_id_generator(id_generator);
    context.use_postgres_provider(mutation_executor.clone());
    let executor = ServiceRuntimeExecutor::new(mutation_executor);
    context.register_executor(executor.clone());
    context.insert_resource(executor);

    // 自动加载 Zero-Code 审计配置与 Schema 模式
    let env_config = teaql_tool_core::audit_config_from_env(&[
        "platform_data", "repository_type_data", "repository_format_data", "write_policy_data", "blob_store_type_data", "user_status_data", "blob_store_configuration_data", "repository_configuration_data", "content_repository_data", "component_data", "asset_blob_data", "asset_data", "security_user_data", "security_role_data", "security_privilege_data"
    ]);
    let schema_mode = env_config.schema_mode;
    context.insert_resource(env_config.config.clone());
    context.insert_resource(env_config);

    match schema_mode {
        teaql_tool_core::SchemaMode::Execute => {
            context.ensure_schema().await?;
        }
        teaql_tool_core::SchemaMode::DryRun => {
            // DryRun: 目前等效于验证
            context.ensure_schema().await?;
        }
        teaql_tool_core::SchemaMode::Verify => {
            context.ensure_schema().await?;
        }
    }

    Ok(context)
}



fn env_value(name: &'static str) -> Result<String, ServiceRuntimeError> {
    std::env::var(name).map_err(|source| ServiceRuntimeError::MissingEnv { name, source })
}

async fn connect_data_service_pool(config: &ServiceRuntimeConfig) -> Result<DataServicePool, ServiceRuntimeError> {
    let pg_config = config.database_url.parse::<tokio_postgres::Config>().map_err(|e| ServiceRuntimeError::ConnectionError(e.to_string()))?;
    let mgr = deadpool_postgres::Manager::new(pg_config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(mgr).build().map_err(|e| ServiceRuntimeError::ConnectionError(e.to_string()))?;
    Ok(pool)
}
pub fn repository_registry() -> teaql_runtime::InMemoryEntityRegistry {
    teaql_runtime::InMemoryEntityRegistry::new()
        .with_entity("Platform")
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
}

pub fn behavior_registry() -> teaql_runtime::InMemoryEntityDataServiceBehaviorRegistry {
    teaql_runtime::InMemoryEntityDataServiceBehaviorRegistry::new()
        .with_behavior("Platform", PlatformBehavior::default())
        .with_behavior("RepositoryType", RepositoryTypeBehavior::default())
        .with_behavior("RepositoryFormat", RepositoryFormatBehavior::default())
        .with_behavior("WritePolicy", WritePolicyBehavior::default())
        .with_behavior("BlobStoreType", BlobStoreTypeBehavior::default())
        .with_behavior("UserStatus", UserStatusBehavior::default())
        .with_behavior("BlobStoreConfiguration", BlobStoreConfigurationBehavior::default())
        .with_behavior("RepositoryConfiguration", RepositoryConfigurationBehavior::default())
        .with_behavior("ContentRepository", ContentRepositoryBehavior::default())
        .with_behavior("Component", ComponentBehavior::default())
        .with_behavior("AssetBlob", AssetBlobBehavior::default())
        .with_behavior("Asset", AssetBehavior::default())
        .with_behavior("SecurityUser", SecurityUserBehavior::default())
        .with_behavior("SecurityRole", SecurityRoleBehavior::default())
        .with_behavior("SecurityPrivilege", SecurityPrivilegeBehavior::default())
}

pub fn checker_registry() -> teaql_runtime::InMemoryCheckerRegistry {
    teaql_runtime::InMemoryCheckerRegistry::new()
        .with_checker(teaql_runtime::TypedEntityChecker::<Platform, _>::new(PlatformChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<RepositoryType, _>::new(RepositoryTypeChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<RepositoryFormat, _>::new(RepositoryFormatChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<WritePolicy, _>::new(WritePolicyChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<BlobStoreType, _>::new(BlobStoreTypeChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<UserStatus, _>::new(UserStatusChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<BlobStoreConfiguration, _>::new(BlobStoreConfigurationChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<RepositoryConfiguration, _>::new(RepositoryConfigurationChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<ContentRepository, _>::new(ContentRepositoryChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<Component, _>::new(ComponentChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<AssetBlob, _>::new(AssetBlobChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<Asset, _>::new(AssetChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<SecurityUser, _>::new(SecurityUserChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<SecurityRole, _>::new(SecurityRoleChecker::default()))
        .with_checker(teaql_runtime::TypedEntityChecker::<SecurityPrivilege, _>::new(SecurityPrivilegeChecker::default()))
}

pub fn module() -> teaql_runtime::RuntimeModule {
    teaql_runtime::RuntimeModule::new()
        .entity::<Platform>()
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
        .initial_graph(teaql_runtime::GraphNode::new("Platform")
            .value("id", 1_u64)
            .value("name", "Nexus Repository Platform")
            .value("version", "3.95.1-01"))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1001_u64)
            .value("name", "Hosted")
            .value("code", "HOSTED")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1002_u64)
            .value("name", "Proxy")
            .value("code", "PROXY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1003_u64)
            .value("name", "Group")
            .value("code", "GROUP")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1001_u64)
            .value("name", "Maven2")
            .value("code", "MAVEN2")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1002_u64)
            .value("name", "Raw")
            .value("code", "RAW")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1001_u64)
            .value("name", "Allow Write")
            .value("code", "ALLOW_WRITE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1002_u64)
            .value("name", "Allow Once")
            .value("code", "ALLOW_ONCE")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1003_u64)
            .value("name", "Read Only")
            .value("code", "READ_ONLY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1001_u64)
            .value("name", "File")
            .value("code", "FILE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1002_u64)
            .value("name", "S3")
            .value("code", "S3")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1001_u64)
            .value("name", "Active")
            .value("code", "ACTIVE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1002_u64)
            .value("name", "Disabled")
            .value("code", "DISABLED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1003_u64)
            .value("name", "Locked")
            .value("code", "LOCKED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
}

pub fn module_with_checkers() -> teaql_runtime::RuntimeModule {
    teaql_runtime::RuntimeModule::new()
        .entity::<Platform>()
        .checker(teaql_runtime::TypedEntityChecker::<Platform, _>::new(PlatformChecker::default()))
        .entity::<RepositoryType>()
        .checker(teaql_runtime::TypedEntityChecker::<RepositoryType, _>::new(RepositoryTypeChecker::default()))
        .entity::<RepositoryFormat>()
        .checker(teaql_runtime::TypedEntityChecker::<RepositoryFormat, _>::new(RepositoryFormatChecker::default()))
        .entity::<WritePolicy>()
        .checker(teaql_runtime::TypedEntityChecker::<WritePolicy, _>::new(WritePolicyChecker::default()))
        .entity::<BlobStoreType>()
        .checker(teaql_runtime::TypedEntityChecker::<BlobStoreType, _>::new(BlobStoreTypeChecker::default()))
        .entity::<UserStatus>()
        .checker(teaql_runtime::TypedEntityChecker::<UserStatus, _>::new(UserStatusChecker::default()))
        .entity::<BlobStoreConfiguration>()
        .checker(teaql_runtime::TypedEntityChecker::<BlobStoreConfiguration, _>::new(BlobStoreConfigurationChecker::default()))
        .entity::<RepositoryConfiguration>()
        .checker(teaql_runtime::TypedEntityChecker::<RepositoryConfiguration, _>::new(RepositoryConfigurationChecker::default()))
        .entity::<ContentRepository>()
        .checker(teaql_runtime::TypedEntityChecker::<ContentRepository, _>::new(ContentRepositoryChecker::default()))
        .entity::<Component>()
        .checker(teaql_runtime::TypedEntityChecker::<Component, _>::new(ComponentChecker::default()))
        .entity::<AssetBlob>()
        .checker(teaql_runtime::TypedEntityChecker::<AssetBlob, _>::new(AssetBlobChecker::default()))
        .entity::<Asset>()
        .checker(teaql_runtime::TypedEntityChecker::<Asset, _>::new(AssetChecker::default()))
        .entity::<SecurityUser>()
        .checker(teaql_runtime::TypedEntityChecker::<SecurityUser, _>::new(SecurityUserChecker::default()))
        .entity::<SecurityRole>()
        .checker(teaql_runtime::TypedEntityChecker::<SecurityRole, _>::new(SecurityRoleChecker::default()))
        .entity::<SecurityPrivilege>()
        .checker(teaql_runtime::TypedEntityChecker::<SecurityPrivilege, _>::new(SecurityPrivilegeChecker::default()))
        .initial_graph(teaql_runtime::GraphNode::new("Platform")
            .value("id", 1_u64)
            .value("name", "Nexus Repository Platform")
            .value("version", "3.95.1-01"))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1001_u64)
            .value("name", "Hosted")
            .value("code", "HOSTED")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1002_u64)
            .value("name", "Proxy")
            .value("code", "PROXY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1003_u64)
            .value("name", "Group")
            .value("code", "GROUP")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1001_u64)
            .value("name", "Maven2")
            .value("code", "MAVEN2")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1002_u64)
            .value("name", "Raw")
            .value("code", "RAW")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1001_u64)
            .value("name", "Allow Write")
            .value("code", "ALLOW_WRITE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1002_u64)
            .value("name", "Allow Once")
            .value("code", "ALLOW_ONCE")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1003_u64)
            .value("name", "Read Only")
            .value("code", "READ_ONLY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1001_u64)
            .value("name", "File")
            .value("code", "FILE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1002_u64)
            .value("name", "S3")
            .value("code", "S3")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1001_u64)
            .value("name", "Active")
            .value("code", "ACTIVE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1002_u64)
            .value("name", "Disabled")
            .value("code", "DISABLED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1003_u64)
            .value("name", "Locked")
            .value("code", "LOCKED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
}

pub fn module_with_behaviors() -> teaql_runtime::RuntimeModule {
    teaql_runtime::RuntimeModule::new()
        .entity_with_behavior::<Platform, _>(PlatformBehavior::default())
        .entity_with_behavior::<RepositoryType, _>(RepositoryTypeBehavior::default())
        .entity_with_behavior::<RepositoryFormat, _>(RepositoryFormatBehavior::default())
        .entity_with_behavior::<WritePolicy, _>(WritePolicyBehavior::default())
        .entity_with_behavior::<BlobStoreType, _>(BlobStoreTypeBehavior::default())
        .entity_with_behavior::<UserStatus, _>(UserStatusBehavior::default())
        .entity_with_behavior::<BlobStoreConfiguration, _>(BlobStoreConfigurationBehavior::default())
        .entity_with_behavior::<RepositoryConfiguration, _>(RepositoryConfigurationBehavior::default())
        .entity_with_behavior::<ContentRepository, _>(ContentRepositoryBehavior::default())
        .entity_with_behavior::<Component, _>(ComponentBehavior::default())
        .entity_with_behavior::<AssetBlob, _>(AssetBlobBehavior::default())
        .entity_with_behavior::<Asset, _>(AssetBehavior::default())
        .entity_with_behavior::<SecurityUser, _>(SecurityUserBehavior::default())
        .entity_with_behavior::<SecurityRole, _>(SecurityRoleBehavior::default())
        .entity_with_behavior::<SecurityPrivilege, _>(SecurityPrivilegeBehavior::default())
        .initial_graph(teaql_runtime::GraphNode::new("Platform")
            .value("id", 1_u64)
            .value("name", "Nexus Repository Platform")
            .value("version", "3.95.1-01"))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1001_u64)
            .value("name", "Hosted")
            .value("code", "HOSTED")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1002_u64)
            .value("name", "Proxy")
            .value("code", "PROXY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1003_u64)
            .value("name", "Group")
            .value("code", "GROUP")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1001_u64)
            .value("name", "Maven2")
            .value("code", "MAVEN2")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1002_u64)
            .value("name", "Raw")
            .value("code", "RAW")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1001_u64)
            .value("name", "Allow Write")
            .value("code", "ALLOW_WRITE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1002_u64)
            .value("name", "Allow Once")
            .value("code", "ALLOW_ONCE")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1003_u64)
            .value("name", "Read Only")
            .value("code", "READ_ONLY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1001_u64)
            .value("name", "File")
            .value("code", "FILE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1002_u64)
            .value("name", "S3")
            .value("code", "S3")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1001_u64)
            .value("name", "Active")
            .value("code", "ACTIVE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1002_u64)
            .value("name", "Disabled")
            .value("code", "DISABLED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1003_u64)
            .value("name", "Locked")
            .value("code", "LOCKED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
}

pub fn module_with_behaviors_and_checkers() -> teaql_runtime::RuntimeModule {
    teaql_runtime::RuntimeModule::new()
        .entity_with_behavior::<Platform, _>(PlatformBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<Platform, _>::new(PlatformChecker::default()))
        .entity_with_behavior::<RepositoryType, _>(RepositoryTypeBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<RepositoryType, _>::new(RepositoryTypeChecker::default()))
        .entity_with_behavior::<RepositoryFormat, _>(RepositoryFormatBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<RepositoryFormat, _>::new(RepositoryFormatChecker::default()))
        .entity_with_behavior::<WritePolicy, _>(WritePolicyBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<WritePolicy, _>::new(WritePolicyChecker::default()))
        .entity_with_behavior::<BlobStoreType, _>(BlobStoreTypeBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<BlobStoreType, _>::new(BlobStoreTypeChecker::default()))
        .entity_with_behavior::<UserStatus, _>(UserStatusBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<UserStatus, _>::new(UserStatusChecker::default()))
        .entity_with_behavior::<BlobStoreConfiguration, _>(BlobStoreConfigurationBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<BlobStoreConfiguration, _>::new(BlobStoreConfigurationChecker::default()))
        .entity_with_behavior::<RepositoryConfiguration, _>(RepositoryConfigurationBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<RepositoryConfiguration, _>::new(RepositoryConfigurationChecker::default()))
        .entity_with_behavior::<ContentRepository, _>(ContentRepositoryBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<ContentRepository, _>::new(ContentRepositoryChecker::default()))
        .entity_with_behavior::<Component, _>(ComponentBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<Component, _>::new(ComponentChecker::default()))
        .entity_with_behavior::<AssetBlob, _>(AssetBlobBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<AssetBlob, _>::new(AssetBlobChecker::default()))
        .entity_with_behavior::<Asset, _>(AssetBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<Asset, _>::new(AssetChecker::default()))
        .entity_with_behavior::<SecurityUser, _>(SecurityUserBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<SecurityUser, _>::new(SecurityUserChecker::default()))
        .entity_with_behavior::<SecurityRole, _>(SecurityRoleBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<SecurityRole, _>::new(SecurityRoleChecker::default()))
        .entity_with_behavior::<SecurityPrivilege, _>(SecurityPrivilegeBehavior::default())
        .checker(teaql_runtime::TypedEntityChecker::<SecurityPrivilege, _>::new(SecurityPrivilegeChecker::default()))
        .initial_graph(teaql_runtime::GraphNode::new("Platform")
            .value("id", 1_u64)
            .value("name", "Nexus Repository Platform")
            .value("version", "3.95.1-01"))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1001_u64)
            .value("name", "Hosted")
            .value("code", "HOSTED")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1002_u64)
            .value("name", "Proxy")
            .value("code", "PROXY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryType")
            .value("id", 1003_u64)
            .value("name", "Group")
            .value("code", "GROUP")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1001_u64)
            .value("name", "Maven2")
            .value("code", "MAVEN2")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("RepositoryFormat")
            .value("id", 1002_u64)
            .value("name", "Raw")
            .value("code", "RAW")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1001_u64)
            .value("name", "Allow Write")
            .value("code", "ALLOW_WRITE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1002_u64)
            .value("name", "Allow Once")
            .value("code", "ALLOW_ONCE")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("WritePolicy")
            .value("id", 1003_u64)
            .value("name", "Read Only")
            .value("code", "READ_ONLY")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1001_u64)
            .value("name", "File")
            .value("code", "FILE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("BlobStoreType")
            .value("id", 1002_u64)
            .value("name", "S3")
            .value("code", "S3")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1001_u64)
            .value("name", "Active")
            .value("code", "ACTIVE")
            .value("display_order", "number()")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1002_u64)
            .value("name", "Disabled")
            .value("code", "DISABLED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
        .initial_graph(teaql_runtime::GraphNode::new("UserStatus")
            .value("id", 1003_u64)
            .value("name", "Locked")
            .value("code", "LOCKED")
            .value("display_order", "1")
            .value("version", 1_i64)
            .value("platform_id", 1_u64))
}