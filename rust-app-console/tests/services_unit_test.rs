use nexus_repository_service_core::{service_runtime, ServiceRuntimeConfig};
use nexus_repository_service_core_workspace::services::{
    AssetService, BlobStoreService, ComponentService, RepositoryService, SecurityService,
};

async fn get_test_runtime() -> nexus_repository_service_core::ServiceRuntime {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = service_runtime(config).await.expect("connect error");
    runtime.ensure_schema().await.expect("schema error");
    runtime
}

#[tokio::test(flavor = "multi_thread")]
async fn test_blob_store_service() {
    let runtime = get_test_runtime().await;

    let unique_name = format!("bs-{}", uuid::Uuid::new_v4().simple());
    let bs = BlobStoreService::create(&runtime, &unique_name, "/tmp/blobs", true)
        .await
        .expect("create blobstore failed");

    assert_eq!(bs.name(), unique_name);
    assert_eq!(bs.path(), "/tmp/blobs");

    let found = BlobStoreService::find_by_name(&runtime, &unique_name)
        .await
        .expect("find blobstore failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), unique_name);

    let all = BlobStoreService::list(&runtime).await.expect("list failed");
    assert!(all.iter().any(|b| b.name() == unique_name));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_repository_service() {
    let runtime = get_test_runtime().await;

    let bs = BlobStoreService::create(
        &runtime,
        &format!("bs-for-repo-{}", uuid::Uuid::new_v4().simple()),
        "/tmp/blobs",
        true,
    )
    .await
    .unwrap();

    let repo_name = format!("repo-{}", uuid::Uuid::new_v4().simple());
    let repo = RepositoryService::create(
        &runtime,
        &repo_name,
        "maven2-hosted",
        "HOSTED",
        "MAVEN2",
        "ALLOW_WRITE",
        bs.id(),
        true,
        "",
    )
    .await
    .expect("create repo failed");

    assert_eq!(repo.name(), repo_name);
    assert_eq!(repo.online(), true);

    let found = RepositoryService::find_by_name(&runtime, &repo_name)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), repo_name);

    let cr = RepositoryService::get_content_repository(&runtime, repo.id())
        .await
        .unwrap();
    assert!(cr.is_some());
    assert_eq!(cr.unwrap().format_name(), "MAVEN2");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_security_service() {
    let runtime = get_test_runtime().await;

    let username = format!("user-{}", uuid::Uuid::new_v4().simple());
    let user = SecurityService::create_user(
        &runtime,
        &username,
        "Test",
        "User",
        "test@example.com",
        "sha512$hashedpass",
    )
    .await
    .expect("create user failed");

    assert_eq!(user.username(), username);
    assert_eq!(user.email(), "test@example.com");

    let found = SecurityService::find_user_by_username(&runtime, &username)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().username(), username);

    let users = SecurityService::list_users(&runtime).await.unwrap();
    assert!(users.iter().any(|u| u.username() == username));

    let unique_role_id = format!("role-{}", uuid::Uuid::new_v4().simple());
    SecurityService::create_role(&runtime, &unique_role_id, "Test Role", "Test Description", false)
        .await
        .expect("create_role failed");

    let roles = SecurityService::list_roles(&runtime).await.unwrap();
    assert!(roles.iter().any(|r| r.role_id() == unique_role_id));

    let unique_priv_id = format!("priv-{}", uuid::Uuid::new_v4().simple());
    SecurityService::create_privilege(
        &runtime,
        &unique_priv_id,
        "Test Priv",
        "Test Priv Description",
        "repository-view",
        "nx-repository-view-*-*-*",
        false,
    )
    .await
    .expect("create_privilege failed");

    let privs = SecurityService::list_privileges(&runtime).await.unwrap();
    assert!(privs.iter().any(|p| p.privilege_id() == unique_priv_id));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_component_and_asset_service() {
    let runtime = get_test_runtime().await;

    let bs = BlobStoreService::create(
        &runtime,
        &format!("bs-asset-{}", uuid::Uuid::new_v4().simple()),
        "/tmp/blobs",
        true,
    )
    .await
    .unwrap();

    let repo = RepositoryService::create(
        &runtime,
        &format!("repo-comp-{}", uuid::Uuid::new_v4().simple()),
        "maven2-hosted",
        "HOSTED",
        "MAVEN2",
        "ALLOW_WRITE",
        bs.id(),
        true,
        "",
    )
    .await
    .unwrap();

    let cr = RepositoryService::ensure_content_repository(&runtime, repo.id(), "MAVEN2")
        .await
        .unwrap();

    // Test ComponentService find_or_create
    let comp = ComponentService::find_or_create(
        &runtime,
        cr.id(),
        "org.apache.commons",
        "commons-lang3",
        "3.12.0",
        "jar",
    )
    .await
    .expect("find_or_create component failed");

    assert_eq!(comp.namespace(), "org.apache.commons");
    assert_eq!(comp.name(), "commons-lang3");
    assert_eq!(comp.version_name(), "3.12.0");

    let comp_by_id = ComponentService::get_by_id(&runtime, comp.id())
        .await
        .unwrap();
    assert!(comp_by_id.is_some());
    assert_eq!(comp_by_id.unwrap().id(), comp.id());

    let comp_list = ComponentService::list_by_content_repository(&runtime, cr.id(), 10, 0)
        .await
        .unwrap();
    assert!(comp_list.iter().any(|c| c.id() == comp.id()));

    // Test AssetService create_asset_blob & create_or_update_asset
    let blob_ref = format!("default@{}", uuid::Uuid::new_v4());
    let asset_blob = AssetService::create_asset_blob(
        &runtime,
        bs.id(),
        &blob_ref,
        1024,
        "application/java-archive",
        "sha1test",
        "sha256test",
        "md5test",
    )
    .await
    .expect("create_asset_blob failed");

    assert_eq!(asset_blob.blob_ref(), blob_ref);
    assert_eq!(asset_blob.blob_size(), 1024);

    let path = "/org/apache/commons/commons-lang3/3.12.0/commons-lang3-3.12.0.jar";
    let asset = AssetService::upsert_asset(
        &runtime,
        cr.id(),
        Some(comp.id()),
        asset_blob.id(),
        path,
        "jar",
    )
    .await
    .expect("upsert_asset failed");

    assert_eq!(asset.path(), path);

    let found_asset = AssetService::find_by_path(&runtime, cr.id(), path)
        .await
        .unwrap();
    assert!(found_asset.is_some());
    assert_eq!(found_asset.unwrap().id(), asset.id());

    let asset_list = AssetService::list_by_content_repository(&runtime, cr.id(), 10, 0)
        .await
        .unwrap();
    assert!(asset_list.iter().any(|a| a.id() == asset.id()));
}
