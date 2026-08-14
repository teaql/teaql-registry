use std::sync::Arc;
use teaql_registry::blobstore::{BlobStore, MemoryBlobStore};
use teaql_registry::services::{
    AssetService, BlobStoreService, CleanupPolicy, CleanupService, ComponentService,
    RepositoryService,
};
use teaql_registry_core::{service_runtime, ServiceRuntimeConfig};

#[tokio::test(flavor = "multi_thread")]
async fn test_retention_and_cleanup_policy() {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime error"));
    runtime.ensure_schema().await.expect("Schema error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("cleanup-store"));
    blobstore.init().await.expect("Blobstore init error");

    let bs_name = format!("bs-cl-{}", uuid::Uuid::new_v4().simple());
    let bs = BlobStoreService::create(&runtime, &bs_name, "/tmp", true).await.unwrap();

    let repo_name = format!("repo-cl-{}", uuid::Uuid::new_v4().simple());
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
    .unwrap();

    let content_repo = RepositoryService::ensure_content_repository(&runtime, repo.id(), "maven2")
        .await
        .unwrap();

    // Create 4 versions of a component: v1.0.0, v1.0.1, v1.0.2, v1.0.3-SNAPSHOT
    for ver in ["1.0.0", "1.0.1", "1.0.2", "1.0.3-SNAPSHOT"] {
        let comp = ComponentService::create(
            &runtime,
            content_repo.id(),
            "com.example",
            "cleanup-test-lib",
            ver,
            ver,
            "jar",
        )
        .await
        .unwrap();

        let blob_info = blobstore.create_blob(format!("Payload for {}", ver).as_bytes()).await.unwrap();
        let asset_blob = AssetService::create_asset_blob(
            &runtime,
            bs.id(),
            &blob_info.blob_ref,
            blob_info.size,
            "application/java-archive",
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await
        .unwrap();

        AssetService::create(
            &runtime,
            content_repo.id(),
            comp.id(),
            asset_blob.id(),
            &format!("/com/example/cleanup-test-lib/{}/cleanup-test-lib-{}.jar", ver, ver),
            "jar",
        )
        .await
        .unwrap();
    }

    // Policy: Keep only latest 2 versions
    let policy = CleanupPolicy {
        max_versions_per_component: Some(2),
        snapshot_only: false,
    };

    let report = CleanupService::run_cleanup(&runtime, blobstore.as_ref(), &repo_name, &policy)
        .await
        .unwrap();

    assert_eq!(report.deleted_components_count, 2);
    assert_eq!(report.deleted_assets_count, 2);
    assert!(report.freed_bytes > 0);
}
