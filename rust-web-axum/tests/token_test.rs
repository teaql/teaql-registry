#![recursion_limit = "256"]

use teaql_registry::security::TokenService;

#[test]
fn test_personal_access_token_lifecycle() {
    let (secret, token) = TokenService::create_token(
        7,
        42,
        "developer1",
        "CI/CD Token for GitHub Actions",
        vec![
            "repository:read".to_string(),
            "repository:write".to_string(),
        ],
        Some(30),
    );

    assert!(secret.starts_with("tql_pat_"));
    assert_eq!(token.username, "developer1");
    assert_eq!(token.scopes.len(), 2);

    // Validate token with required scope
    let valid_user =
        TokenService::validate_token(&secret, "repository:write").expect("valid token");
    assert_eq!(valid_user.tenant_id, 7);
    assert_eq!(valid_user.user_id, 42);
    assert_eq!(valid_user.username, "developer1");

    let read_user = TokenService::validate_token(&secret, "repository:read").expect("valid token");
    assert_eq!(read_user.username, "developer1");

    let invalid_scope = TokenService::validate_token(&secret, "admin");
    assert_eq!(invalid_scope, None);

    // Revoke token
    assert!(!TokenService::revoke_token(8, 42, &token.id));
    assert!(!TokenService::revoke_token(7, 43, &token.id));
    let revoked = TokenService::revoke_token(7, 42, &token.id);
    assert!(revoked);

    // Validate revoked token fails
    assert_eq!(
        TokenService::validate_token(&secret, "repository:write"),
        None
    );
}

#[test]
fn zero_day_token_means_no_expiration() {
    let (_secret, token) = TokenService::create_token(
        7,
        42,
        "developer1",
        "non-expiring token",
        vec!["repository:read".to_string()],
        Some(0),
    );
    assert_eq!(token.expires_at, None);
}
