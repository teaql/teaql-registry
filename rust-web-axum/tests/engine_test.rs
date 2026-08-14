use nexus_repository_service_core::{service_runtime, ServiceRuntimeConfig};
use nexus_repository_service_core_workspace::{
    blobstore::{BlobStore, S3BlobStore},
    engine::{GroupEngine, HostedEngine},
    services::{BlobStoreService, RepositoryService},
};
use std::sync::Arc;

async fn setup_engine_env() -> (
    Arc<nexus_repository_service_core::ServiceRuntime>,
    Arc<dyn BlobStore>,
    u64,
) {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("connect error"));
    runtime.ensure_schema().await.expect("schema error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(S3BlobStore::from_env("engine-store"));
    blobstore.init().await.expect("init error");

    let bs_name = format!("bs-eng-{}", uuid::Uuid::new_v4().simple());
    let bs = BlobStoreService::create(&runtime, &bs_name, "/tmp/blobs", true)
        .await
        .unwrap();

    (runtime, blobstore, bs.id())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hosted_engine_write_policies() {
    let (runtime, blobstore, bs_id) = setup_engine_env().await;

    // 1. ALLOW_WRITE repo
    let allow_write_name = format!("repo-allow-write-{}", uuid::Uuid::new_v4().simple());
    let repo_allow_write = RepositoryService::create(
        &runtime,
        &allow_write_name,
        "raw-hosted",
        "HOSTED",
        "RAW",
        "ALLOW_WRITE",
        bs_id,
        true,
        "",
    )
    .await
    .unwrap();

    let data_v1 = b"Version 1 content";
    let data_v2 = b"Version 2 updated content";
    let path = "/test/file.txt";

    HostedEngine::handle_put(
        &runtime,
        &repo_allow_write,
        &blobstore,
        path,
        data_v1,
        "text/plain",
    )
    .await
    .expect("First upload should succeed");

    // Overwrite in ALLOW_WRITE should succeed
    HostedEngine::handle_put(
        &runtime,
        &repo_allow_write,
        &blobstore,
        path,
        data_v2,
        "text/plain",
    )
    .await
    .expect("Overwrite in ALLOW_WRITE should succeed");

    let (bytes, content_type) = HostedEngine::handle_get(&runtime, &repo_allow_write, &blobstore, path)
        .await
        .unwrap()
        .expect("Asset should exist");
    assert_eq!(bytes.as_ref(), data_v2);
    assert_eq!(content_type, "text/plain");

    // 2. ALLOW_ONCE repo
    let allow_once_name = format!("repo-allow-once-{}", uuid::Uuid::new_v4().simple());
    let repo_allow_once = RepositoryService::create(
        &runtime,
        &allow_once_name,
        "maven2-hosted",
        "HOSTED",
        "MAVEN2",
        "ALLOW_ONCE",
        bs_id,
        true,
        "",
    )
    .await
    .unwrap();

    let jar_path = "/com/example/demo/1.0.0/demo-1.0.0.jar";
    HostedEngine::handle_put(
        &runtime,
        &repo_allow_once,
        &blobstore,
        jar_path,
        b"jar v1",
        "application/java-archive",
    )
    .await
    .expect("First deployment in ALLOW_ONCE should succeed");
    println!("DEBUG: step 2 first put done");

    // Second upload to ALLOW_ONCE should return error
    let once_second_res = HostedEngine::handle_put(
        &runtime,
        &repo_allow_once,
        &blobstore,
        jar_path,
        b"jar v2 overwrite attempt",
        "application/java-archive",
    )
    .await;
    assert!(
        once_second_res.is_err(),
        "Expected ALLOW_ONCE repo to reject redeployment"
    );

    // 3. READ_ONLY repo
    let read_only_name = format!("repo-read-only-{}", uuid::Uuid::new_v4().simple());
    let repo_read_only = RepositoryService::create(
        &runtime,
        &read_only_name,
        "maven2-hosted",
        "HOSTED",
        "MAVEN2",
        "READ_ONLY",
        bs_id,
        true,
        "",
    )
    .await
    .unwrap();

    let ro_res = HostedEngine::handle_put(
        &runtime,
        &repo_read_only,
        &blobstore,
        "/com/example/demo/1.0.0/demo-1.0.0.jar",
        b"data",
        "application/java-archive",
    )
    .await;
    assert!(ro_res.is_err(), "Expected READ_ONLY repo to reject uploads");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hosted_engine_missing_asset() {
    let (runtime, blobstore, bs_id) = setup_engine_env().await;

    let repo_name = format!("repo-missing-{}", uuid::Uuid::new_v4().simple());
    let repo = RepositoryService::create(
        &runtime,
        &repo_name,
        "raw-hosted",
        "HOSTED",
        "RAW",
        "ALLOW_WRITE",
        bs_id,
        true,
        "",
    )
    .await
    .unwrap();

    // GET non-existent
    let missing_resp = HostedEngine::handle_get(&runtime, &repo, &blobstore, "/non/existent.txt")
        .await
        .unwrap();
    assert!(missing_resp.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_group_engine_routing() {
    let (runtime, blobstore, bs_id) = setup_engine_env().await;

    let member1_name = format!("member1-{}", uuid::Uuid::new_v4().simple());
    let member1 = RepositoryService::create(
        &runtime,
        &member1_name,
        "raw-hosted",
        "HOSTED",
        "RAW",
        "ALLOW_WRITE",
        bs_id,
        true,
        "",
    )
    .await
    .unwrap();

    let group_name = format!("group-{}", uuid::Uuid::new_v4().simple());
    let group = RepositoryService::create(
        &runtime,
        &group_name,
        "raw-group",
        "GROUP",
        "RAW",
        "READ_ONLY",
        bs_id,
        true,
        "",
    )
    .await
    .unwrap();

    // Upload to member1
    let path = "/shared/config.json";
    let payload = b"{\"env\": \"production\"}";
    HostedEngine::handle_put(
        &runtime,
        &member1,
        &blobstore,
        path,
        payload,
        "application/json",
    )
    .await
    .unwrap();

    // GroupEngine should resolve the asset from member1
    let (bytes, content_type) = GroupEngine::handle_get(&runtime, &group, &blobstore, path)
        .await
        .unwrap()
        .expect("GroupEngine should find asset in member1");
    assert_eq!(bytes.as_ref(), payload);
    assert_eq!(content_type, "application/json");
}
