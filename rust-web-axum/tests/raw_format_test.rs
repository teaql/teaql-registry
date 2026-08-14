use nexus_repository_service_core_workspace::format::raw::sanitize_raw_path;

#[test]
fn test_sanitize_raw_path_leading_slashes() {
    assert_eq!(sanitize_raw_path("file.txt"), "/file.txt");
    assert_eq!(sanitize_raw_path("/file.txt"), "/file.txt");
    assert_eq!(sanitize_raw_path("///nested/dir/app.zip"), "/nested/dir/app.zip");
}

#[test]
fn test_sanitize_raw_path_nested() {
    assert_eq!(
        sanitize_raw_path("/assets/images/logo.png"),
        "/assets/images/logo.png"
    );
    assert_eq!(
        sanitize_raw_path("dist/bundle.tar.gz"),
        "/dist/bundle.tar.gz"
    );
}
