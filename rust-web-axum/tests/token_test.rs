#![recursion_limit = "256"]

mod common;

use teaql_registry::{
    security::{hash_password, TokenService},
    services::{SecurityService, TenantService},
};
use teaql_registry_core::{service_runtime, SecurityUser, ServiceRuntime};

async fn test_context_and_user() -> (ServiceRuntime, SecurityUser, u64) {
    let context = service_runtime(common::runtime_config())
        .await
        .expect("runtime connect");
    context.ensure_schema().await.expect("schema init");
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let tenant = TenantService::create_tenant(
        &context,
        &format!("Token Test {suffix}"),
        &format!("token-test-{suffix}"),
    )
    .await
    .expect("create token tenant");
    let username = format!("token-test-{suffix}");
    let email = format!("{username}@example.invalid");
    let user = SecurityService::create_user_with_tenant(
        &context,
        tenant.id(),
        &username,
        "Token",
        "Test",
        &email,
        &hash_password("Token-Test-Password-42!"),
    )
    .await
    .expect("create token owner");
    let tenant_id = tenant.id();
    (context, user, tenant_id)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_personal_access_token_lifecycle() {
    let (context, user, tenant_id) = test_context_and_user().await;
    let username = user.username();
    let (secret, token) = TokenService::create_token(
        &context,
        tenant_id,
        user.id(),
        &username,
        "CI/CD Token for GitHub Actions",
        vec![
            "repository:read".to_string(),
            "repository:write".to_string(),
        ],
        Some(30),
    )
    .await
    .expect("create token");

    assert!(secret.starts_with("tql_pat_"));
    assert_eq!(token.username, username);
    assert_eq!(token.scopes.len(), 2);

    // Validate token with required scope
    let valid_user = TokenService::validate_token(&context, &secret, "repository:write")
        .await
        .expect("token lookup")
        .expect("valid token");
    assert_eq!(valid_user.tenant_id, tenant_id);
    assert_eq!(valid_user.user_id, user.id());
    assert_eq!(valid_user.username, username);

    let read_user = TokenService::validate_token(&context, &secret, "repository:read")
        .await
        .expect("token lookup")
        .expect("valid token");
    assert_eq!(read_user.username, username);

    let invalid_scope = TokenService::validate_token(&context, &secret, "admin")
        .await
        .expect("token lookup");
    assert_eq!(invalid_scope, None);

    // Revoke token
    assert!(
        !TokenService::revoke_token(&context, tenant_id + 1, user.id(), &token.id)
            .await
            .expect("wrong-tenant revoke")
    );
    assert!(
        !TokenService::revoke_token(&context, tenant_id, user.id() + 1, &token.id)
            .await
            .expect("wrong-user revoke")
    );
    let revoked = TokenService::revoke_token(&context, tenant_id, user.id(), &token.id)
        .await
        .expect("revoke token");
    assert!(revoked);

    // Validate revoked token fails
    assert_eq!(
        TokenService::validate_token(&context, &secret, "repository:write")
            .await
            .expect("token lookup"),
        None
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn zero_day_token_means_no_expiration_and_survives_a_new_runtime() {
    let (context, user, tenant_id) = test_context_and_user().await;
    let username = user.username();
    let (_secret, token) = TokenService::create_token(
        &context,
        tenant_id,
        user.id(),
        &username,
        "non-expiring token",
        vec!["repository:read".to_string()],
        Some(0),
    )
    .await
    .expect("create token");
    assert_eq!(token.expires_at, None);

    let (secret, _) = TokenService::create_token(
        &context,
        tenant_id,
        user.id(),
        &username,
        "restart persistence token",
        vec!["repository:read".to_string()],
        Some(0),
    )
    .await
    .expect("create persistence token");
    drop(context);

    let restarted_context = service_runtime(common::runtime_config())
        .await
        .expect("restart runtime connect");
    let principal = TokenService::validate_token(&restarted_context, &secret, "repository:read")
        .await
        .expect("persistent token lookup")
        .expect("token survives runtime recreation");
    assert_eq!(principal.user_id, user.id());
}
