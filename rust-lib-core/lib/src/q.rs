use teaql_core::Expr;

use crate::*;

pub struct PurposedQuery<T> {
    pub inner: T,
    pub purpose: String,
}

impl<T> PurposedQuery<T> {
    pub fn new(inner: T, purpose: impl Into<String>) -> Self {
        Self { inner, purpose: purpose.into() }
    }
}

pub struct Q;

impl Q {
    pub fn platforms() -> PlatformRequest {
        PlatformRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn platforms_minimal() -> PlatformRequest {
        PlatformRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn platforms_with_children() -> PlatformRequest {
        PlatformRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn tenants() -> TenantRequest {
        TenantRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn tenants_minimal() -> TenantRequest {
        TenantRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn tenants_with_children() -> TenantRequest {
        TenantRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn repository_types() -> RepositoryTypeRequest {
        RepositoryTypeRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn repository_types_minimal() -> RepositoryTypeRequest {
        RepositoryTypeRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn repository_types_with_children() -> RepositoryTypeRequest {
        RepositoryTypeRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn repository_formats() -> RepositoryFormatRequest {
        RepositoryFormatRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn repository_formats_minimal() -> RepositoryFormatRequest {
        RepositoryFormatRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn repository_formats_with_children() -> RepositoryFormatRequest {
        RepositoryFormatRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn write_policies() -> WritePolicyRequest {
        WritePolicyRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn write_policies_minimal() -> WritePolicyRequest {
        WritePolicyRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn write_policies_with_children() -> WritePolicyRequest {
        WritePolicyRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn blob_store_types() -> BlobStoreTypeRequest {
        BlobStoreTypeRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn blob_store_types_minimal() -> BlobStoreTypeRequest {
        BlobStoreTypeRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn blob_store_types_with_children() -> BlobStoreTypeRequest {
        BlobStoreTypeRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn user_statuses() -> UserStatusRequest {
        UserStatusRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn user_statuses_minimal() -> UserStatusRequest {
        UserStatusRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn user_statuses_with_children() -> UserStatusRequest {
        UserStatusRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn blob_store_configurations() -> BlobStoreConfigurationRequest {
        BlobStoreConfigurationRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn blob_store_configurations_minimal() -> BlobStoreConfigurationRequest {
        BlobStoreConfigurationRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn blob_store_configurations_with_children() -> BlobStoreConfigurationRequest {
        BlobStoreConfigurationRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn repository_configurations() -> RepositoryConfigurationRequest {
        RepositoryConfigurationRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn repository_configurations_minimal() -> RepositoryConfigurationRequest {
        RepositoryConfigurationRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn repository_configurations_with_children() -> RepositoryConfigurationRequest {
        RepositoryConfigurationRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn content_repositories() -> ContentRepositoryRequest {
        ContentRepositoryRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn content_repositories_minimal() -> ContentRepositoryRequest {
        ContentRepositoryRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn content_repositories_with_children() -> ContentRepositoryRequest {
        ContentRepositoryRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn components() -> ComponentRequest {
        ComponentRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn components_minimal() -> ComponentRequest {
        ComponentRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn components_with_children() -> ComponentRequest {
        ComponentRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn asset_blobs() -> AssetBlobRequest {
        AssetBlobRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn asset_blobs_minimal() -> AssetBlobRequest {
        AssetBlobRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn asset_blobs_with_children() -> AssetBlobRequest {
        AssetBlobRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn assets() -> AssetRequest {
        AssetRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn assets_minimal() -> AssetRequest {
        AssetRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn assets_with_children() -> AssetRequest {
        AssetRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn security_users() -> SecurityUserRequest {
        SecurityUserRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn security_users_minimal() -> SecurityUserRequest {
        SecurityUserRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn security_users_with_children() -> SecurityUserRequest {
        SecurityUserRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn security_roles() -> SecurityRoleRequest {
        SecurityRoleRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn security_roles_minimal() -> SecurityRoleRequest {
        SecurityRoleRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn security_roles_with_children() -> SecurityRoleRequest {
        SecurityRoleRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }

    pub fn security_privileges() -> SecurityPrivilegeRequest {
        SecurityPrivilegeRequest::new()
            .select_self()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn security_privileges_minimal() -> SecurityPrivilegeRequest {
        SecurityPrivilegeRequest::new()
            .and_filter(Expr::gt("version", 0_i64))
    }

    pub fn security_privileges_with_children() -> SecurityPrivilegeRequest {
        SecurityPrivilegeRequest::new()
            .unlimited()
            .select_self_fields()
            .enhance_children_if_needed()
    }
}