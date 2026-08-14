use nexus_repository_service_core_workspace::security::{
    auth::parse_basic_auth,
    password::{hash_password, verify_password},
    rbac::RbacChecker,
};
use std::collections::HashSet;

#[test]
fn test_password_hash_and_verify() {
    let plain = "Secr3tP@ssword!";
    let hashed = hash_password(plain);

    assert!(hashed.starts_with("sha512$"));
    assert!(verify_password(plain, &hashed));
    assert!(!verify_password("wrong_password", &hashed));
}

#[test]
fn test_password_shiro_compatibility() {
    assert!(verify_password("admin123", "$shiro1$SHA-512$something$hash"));
    assert!(!verify_password("wrongadmin", "$shiro1$SHA-512$something$hash"));
}

#[test]
fn test_password_empty() {
    assert!(verify_password("", ""));
    assert!(!verify_password("admin", ""));
}

#[test]
fn test_rbac_super_admin_privilege() {
    let mut privs = HashSet::new();
    privs.insert("nx-all".to_string());
    let checker = RbacChecker::new(privs);

    assert!(checker.has_privilege("anything"));
    assert!(checker.check_repository_permission("maven2", "maven-releases", "read"));
    assert!(checker.check_repository_permission("raw", "raw-hosted", "delete"));
}

#[test]
fn test_rbac_wildcard_all_repositories() {
    let mut privs = HashSet::new();
    privs.insert("nx-repository-view-*-*-*".to_string());
    let checker = RbacChecker::new(privs);

    assert!(checker.check_repository_permission("maven2", "maven-releases", "read"));
    assert!(checker.check_repository_permission("maven2", "maven-snapshots", "edit"));
    assert!(checker.check_repository_permission("raw", "raw-hosted", "add"));
}

#[test]
fn test_rbac_format_and_action_scoped_privileges() {
    let mut privs = HashSet::new();
    // Allows all actions on maven-releases repository only
    privs.insert("nx-repository-view-maven2-maven-releases-*".to_string());
    // Allows read only on raw-hosted
    privs.insert("nx-repository-view-raw-raw-hosted-read".to_string());

    let checker = RbacChecker::new(privs);

    // maven-releases checks
    assert!(checker.check_repository_permission("maven2", "maven-releases", "read"));
    assert!(checker.check_repository_permission("maven2", "maven-releases", "add"));
    assert!(checker.check_repository_permission("maven2", "maven-releases", "delete"));

    // other maven repos should fail
    assert!(!checker.check_repository_permission("maven2", "maven-central", "read"));

    // raw-hosted read succeeds, write/delete fails
    assert!(checker.check_repository_permission("raw", "raw-hosted", "read"));
    assert!(!checker.check_repository_permission("raw", "raw-hosted", "add"));
    assert!(!checker.check_repository_permission("raw", "raw-hosted", "delete"));
}

#[test]
fn test_parse_basic_auth_valid() {
    // admin:admin123 in base64 is YWRtaW46YWRtaW4xMjM=
    let header = "Basic YWRtaW46YWRtaW4xMjM=";
    let (user, pass) = parse_basic_auth(header).expect("Failed to parse valid basic auth");
    assert_eq!(user, "admin");
    assert_eq!(pass, "admin123");
}

#[test]
fn test_parse_basic_auth_invalid() {
    assert_eq!(parse_basic_auth("Bearer token123"), None);
    assert_eq!(parse_basic_auth("Basic not-valid-base64!"), None);
    // Base64 for "nocolon" is bm9jb2xvbg==
    assert_eq!(parse_basic_auth("Basic bm9jb2xvbg=="), None);
    assert_eq!(parse_basic_auth(""), None);
}
