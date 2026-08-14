use teaql_registry::security::TokenService;

#[test]
fn test_personal_access_token_lifecycle() {
    let (secret, token) = TokenService::create_token(
        "developer1",
        "CI/CD Token for GitHub Actions",
        vec!["read".to_string(), "write".to_string()],
        Some(30),
    );

    assert!(secret.starts_with("tql_pat_"));
    assert_eq!(token.username, "developer1");
    assert_eq!(token.scopes.len(), 2);

    // Validate token with required scope
    let valid_user = TokenService::validate_token(&secret, "write");
    assert_eq!(valid_user, Some("developer1".to_string()));

    let read_user = TokenService::validate_token(&secret, "read");
    assert_eq!(read_user, Some("developer1".to_string()));

    let invalid_scope = TokenService::validate_token(&secret, "admin");
    assert_eq!(invalid_scope, None);

    // Revoke token
    let revoked = TokenService::revoke_token(&token.id);
    assert!(revoked);

    // Validate revoked token fails
    assert_eq!(TokenService::validate_token(&secret, "write"), None);
}
