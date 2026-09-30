use crate::services::SaveAuditedExt;
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use teaql_core::{Entity, SmartList};
use teaql_registry_core::{
    SecurityPrivilege, SecurityRole, SecurityRolePrivilege, SecurityUser, SecurityUserRole,
    ServiceRuntime, Q,
};

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
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
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
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to find user: {}", e))?;
        Ok(rows.into_iter().next())
    }

    pub async fn list_users(ctx: &ServiceRuntime) -> Result<SmartList<SecurityUser>> {
        let rows = Q::security_users_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
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
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
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
        Self::create_user_with_tenant(
            ctx,
            tenant_id,
            username,
            first_name,
            last_name,
            email,
            password_hash,
        )
        .await
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
            .comment("what: create tenant security metadata")
            .purpose("why: provision durable registry authorization")
            .new_entity(ctx);

        user.update_tenant_id(tenant_id);
        user.update_username(username);
        user.update_first_name(first_name);
        user.update_last_name(last_name);
        user.update_email(email);
        user.update_password_hash(password_hash);
        user.update_user_status_to_active();

        let user = user
            .audit_as("Creating user account")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save user: {}", e))?;

        Ok(user)
    }

    pub async fn replace_password_hash(
        ctx: &ServiceRuntime,
        mut user: SecurityUser,
        password_hash: &str,
    ) -> Result<SecurityUser> {
        user.update_password_hash(password_hash);
        user.audit_as("Rotate registry account credential")
            .save_with(ctx)
            .await
    }

    pub async fn list_roles(ctx: &ServiceRuntime) -> Result<SmartList<SecurityRole>> {
        let rows = Q::security_roles_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
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
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to list roles: {}", e))?;
        Ok(rows)
    }

    pub async fn list_privileges(ctx: &ServiceRuntime) -> Result<SmartList<SecurityPrivilege>> {
        let rows = Q::security_privileges_minimal()
            .select_self_fields()
            .limit(100)
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
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
            .comment("what: query tenant security metadata")
            .purpose("why: authenticate and authorize registry operations")
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
            .comment("what: create tenant security metadata")
            .purpose("why: provision durable registry authorization")
            .new_entity(ctx);

        role.update_tenant_id(tenant_id);
        role.update_role_id(role_id);
        role.update_name(name);
        role.update_description(description);
        role.update_read_only(read_only);

        let role = role
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

    #[allow(clippy::too_many_arguments)]
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
            .comment("what: create tenant security metadata")
            .purpose("why: provision durable registry authorization")
            .new_entity(ctx);

        priv_entity.update_tenant_id(tenant_id);
        priv_entity.update_privilege_id(privilege_id);
        priv_entity.update_name(name);
        priv_entity.update_description(description);
        priv_entity.update_privilege_type(privilege_type);
        priv_entity.update_permission_pattern(permission_pattern);
        priv_entity.update_read_only(read_only);

        let priv_entity = priv_entity
            .audit_as("Creating security privilege")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to save security privilege: {}", e))?;

        Ok(priv_entity)
    }

    pub async fn find_role_by_tenant_and_key(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        role_key: &str,
    ) -> Result<Option<SecurityRole>> {
        let rows = Q::security_roles_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .with_role_id_is(role_key)
            .limit(1)
            .comment("what: locate a tenant security role")
            .purpose("why: create or evaluate a durable role assignment")
            .execute_for_list(ctx)
            .await
            .map_err(|error| anyhow!("Failed to find role: {error}"))?;
        Ok(rows.into_iter().next())
    }

    pub async fn find_privilege_by_tenant_and_key(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        privilege_key: &str,
    ) -> Result<Option<SecurityPrivilege>> {
        let rows = Q::security_privileges_minimal()
            .select_self_fields()
            .filter_by_tenant(tenant_id)
            .with_privilege_id_is(privilege_key)
            .limit(1)
            .comment("what: locate a tenant security privilege")
            .purpose("why: create or evaluate a durable privilege assignment")
            .execute_for_list(ctx)
            .await
            .map_err(|error| anyhow!("Failed to find privilege: {error}"))?;
        Ok(rows.into_iter().next())
    }

    pub async fn reconcile_privilege_pattern(
        ctx: &ServiceRuntime,
        mut privilege: SecurityPrivilege,
        privilege_type: &str,
        permission_pattern: &str,
    ) -> Result<SecurityPrivilege> {
        if privilege.privilege_type() == privilege_type
            && privilege.permission_pattern() == permission_pattern
        {
            return Ok(privilege);
        }
        privilege.update_privilege_type(privilege_type);
        privilege.update_permission_pattern(permission_pattern);
        privilege
            .audit_as("Reconcile registry privilege policy")
            .save_with(ctx)
            .await
    }

    pub async fn assign_role(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> Result<SecurityUserRole> {
        let rows = Q::security_user_roles_minimal()
            .select_self_fields()
            .with_security_user_matching(Q::security_users_minimal().with_id_is(user_id))
            .with_security_role_matching(Q::security_roles_minimal().with_id_is(role_id))
            .limit(1)
            .comment("what: locate an existing user role assignment")
            .purpose("why: keep tenant security provisioning idempotent")
            .execute_for_list(ctx)
            .await
            .map_err(|error| anyhow!("Failed to query user role assignment: {error}"))?;
        if let Some(assignment) = rows.into_iter().next() {
            return Ok(assignment);
        }

        let mut assignment = Q::security_user_roles()
            .comment("what: initialize a user role assignment")
            .purpose("why: bind an authenticated principal to explicit permissions")
            .new_entity(ctx);
        assignment.update_tenant_id(tenant_id);
        assignment.update_security_user_id(user_id);
        assignment.update_security_role_id(role_id);
        assignment
            .audit_as("Assign security role to user")
            .save_with(ctx)
            .await
    }

    pub async fn assign_privilege(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        role_id: u64,
        privilege_id: u64,
    ) -> Result<SecurityRolePrivilege> {
        let rows = Q::security_role_privileges_minimal()
            .select_self_fields()
            .with_security_role_matching(Q::security_roles_minimal().with_id_is(role_id))
            .with_security_privilege_matching(
                Q::security_privileges_minimal().with_id_is(privilege_id),
            )
            .limit(1)
            .comment("what: locate an existing role privilege assignment")
            .purpose("why: keep tenant security provisioning idempotent")
            .execute_for_list(ctx)
            .await
            .map_err(|error| anyhow!("Failed to query role privilege assignment: {error}"))?;
        if let Some(assignment) = rows.into_iter().next() {
            return Ok(assignment);
        }

        let mut assignment = Q::security_role_privileges()
            .comment("what: initialize a role privilege assignment")
            .purpose("why: bind a role to an explicit permission pattern")
            .new_entity(ctx);
        assignment.update_tenant_id(tenant_id);
        assignment.update_security_role_id(role_id);
        assignment.update_security_privilege_id(privilege_id);
        assignment
            .audit_as("Assign security privilege to role")
            .save_with(ctx)
            .await
    }

    pub async fn permissions_for_user(
        ctx: &ServiceRuntime,
        user_id: u64,
    ) -> Result<HashSet<String>> {
        let user_roles = Q::security_user_roles_minimal()
            .select_self_fields()
            .with_security_user_matching(Q::security_users_minimal().with_id_is(user_id))
            .limit(100)
            .comment("what: load roles assigned to the authenticated user")
            .purpose("why: authorize the current registry request")
            .execute_for_list(ctx)
            .await
            .map_err(|error| anyhow!("Failed to load user roles: {error}"))?;

        let mut permissions = HashSet::new();
        for user_role in user_roles {
            let role_privileges = Q::security_role_privileges_minimal()
                .select_self_fields()
                .with_security_role_matching(
                    Q::security_roles_minimal().with_id_is(user_role.security_role_id()),
                )
                .limit(100)
                .comment("what: load privileges assigned to an authenticated role")
                .purpose("why: authorize the current registry request")
                .execute_for_list(ctx)
                .await
                .map_err(|error| anyhow!("Failed to load role privileges: {error}"))?;

            for role_privilege in role_privileges {
                let privileges = Q::security_privileges_minimal()
                    .select_self_fields()
                    .with_id_is(role_privilege.security_privilege_id())
                    .limit(1)
                    .comment("what: load an assigned permission pattern")
                    .purpose("why: authorize the current registry request")
                    .execute_for_list(ctx)
                    .await
                    .map_err(|error| anyhow!("Failed to load assigned privilege: {error}"))?;
                if let Some(privilege) = privileges.into_iter().next() {
                    permissions.insert(privilege.permission_pattern().to_string());
                }
            }
        }
        Ok(permissions)
    }
}
