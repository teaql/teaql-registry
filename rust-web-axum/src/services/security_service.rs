use anyhow::{anyhow, Result};
use teaql_registry_core::{
    Q, SecurityPrivilege, SecurityRole, SecurityUser, ServiceRuntime,
};
use teaql_core::{Entity, SmartList};
use crate::services::SaveAuditedExt;

use crate::context::NexusContextExt;

pub struct SecurityService;

impl SecurityService {
    pub async fn find_user_by_username(
        ctx: &ServiceRuntime,
        username: &str,
    ) -> Result<Option<SecurityUser>> {
        let rows = Q::security_users_minimal()
            .select_self_fields()
            .with_username_is(username)
            .limit(1)
            .comment("what: Load user by username")
            .purpose("why: Authenticate incoming request")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find user: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn find_user_by_tenant_and_username(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        username: &str,
    ) -> Result<Option<SecurityUser>> {
        let rows = Q::security_users_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .with_username_is(username)
            .limit(1)
            .comment("what: Load user by username and tenant")
            .purpose("why: Authenticate incoming request")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find user: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn list_users(ctx: &ServiceRuntime) -> Result<SmartList<SecurityUser>> {
        let rows = Q::security_users_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: List all users for tenant")
            .purpose("why: REST security users list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list users: {}", e))?;
        Ok(rows)
    }

    pub async fn list_users_by_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
    ) -> Result<SmartList<SecurityUser>> {
        let rows = Q::security_users_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .limit(100)
            .comment("what: List all users for tenant")
            .purpose("why: REST security users list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list users: {}", e))?;
        Ok(rows)
    }

    pub async fn create_user(
        ctx: &ServiceRuntime,
        username: &str,
        first_name: &str,
        last_name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<SecurityUser> {
        let tenant_id = ctx.tenant_id();
        Self::create_user_with_tenant(ctx, tenant_id, username, first_name, last_name, email, password_hash).await
    }

    pub async fn create_user_with_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        username: &str,
        first_name: &str,
        last_name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<SecurityUser> {
        let mut user = Q::security_users()
            .purpose("why: Create new user account")
            .new_entity(ctx);

        user.update_tenant_id(tenant_id);
        user.update_username(username);
        user.update_first_name(first_name);
        user.update_last_name(last_name);
        user.update_email(email);
        user.update_password_hash(password_hash);
        user.update_user_status_to_active();

        user.clone()
            .audit_as("Creating user account")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save user: {}", e))?;

        Ok(user)
    }

    pub async fn list_roles(ctx: &ServiceRuntime) -> Result<SmartList<SecurityRole>> {
        let rows = Q::security_roles_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: List all roles for tenant")
            .purpose("why: REST security roles list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list roles: {}", e))?;
        Ok(rows)
    }

    pub async fn list_roles_by_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
    ) -> Result<SmartList<SecurityRole>> {
        let rows = Q::security_roles_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .limit(100)
            .comment("what: List all roles for tenant")
            .purpose("why: REST security roles list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list roles: {}", e))?;
        Ok(rows)
    }

    pub async fn list_privileges(ctx: &ServiceRuntime) -> Result<SmartList<SecurityPrivilege>> {
        let rows = Q::security_privileges_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: List all privileges for tenant")
            .purpose("why: REST security privileges list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list privileges: {}", e))?;
        Ok(rows)
    }

    pub async fn list_privileges_by_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
    ) -> Result<SmartList<SecurityPrivilege>> {
        let rows = Q::security_privileges_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .limit(100)
            .comment("what: List all privileges for tenant")
            .purpose("why: REST security privileges list API")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list privileges: {}", e))?;
        Ok(rows)
    }

    pub async fn create_role(
        ctx: &ServiceRuntime,
        role_id: &str,
        name: &str,
        description: &str,
        read_only: bool,
    ) -> Result<SecurityRole> {
        let tenant_id = ctx.tenant_id();
        Self::create_role_with_tenant(ctx, tenant_id, role_id, name, description, read_only).await
    }

    pub async fn create_role_with_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        role_id: &str,
        name: &str,
        description: &str,
        read_only: bool,
    ) -> Result<SecurityRole> {
        let mut role = Q::security_roles()
            .purpose("why: Create new security role")
            .new_entity(ctx);

        role.update_tenant_id(tenant_id);
        role.update_role_id(role_id);
        role.update_name(name);
        role.update_description(description);
        role.update_read_only(read_only);

        role.clone()
            .audit_as("Creating security role")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save security role: {}", e))?;

        Ok(role)
    }

    pub async fn create_privilege(
        ctx: &ServiceRuntime,
        privilege_id: &str,
        name: &str,
        description: &str,
        privilege_type: &str,
        permission_pattern: &str,
        read_only: bool,
    ) -> Result<SecurityPrivilege> {
        let tenant_id = ctx.tenant_id();
        Self::create_privilege_with_tenant(
            ctx,
            tenant_id,
            privilege_id,
            name,
            description,
            privilege_type,
            permission_pattern,
            read_only,
        )
        .await
    }

    pub async fn create_privilege_with_tenant(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        privilege_id: &str,
        name: &str,
        description: &str,
        privilege_type: &str,
        permission_pattern: &str,
        read_only: bool,
    ) -> Result<SecurityPrivilege> {
        let mut priv_entity = Q::security_privileges()
            .purpose("why: Create new security privilege")
            .new_entity(ctx);

        priv_entity.update_tenant_id(tenant_id);
        priv_entity.update_privilege_id(privilege_id);
        priv_entity.update_name(name);
        priv_entity.update_description(description);
        priv_entity.update_privilege_type(privilege_type);
        priv_entity.update_permission_pattern(permission_pattern);
        priv_entity.update_read_only(read_only);

        priv_entity
            .clone()
            .audit_as("Creating security privilege")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save security privilege: {}", e))?;

        Ok(priv_entity)
    }
}
