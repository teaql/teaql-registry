use std::sync::Arc;
use teaql_registry::blobstore::{BlobStore, MemoryBlobStore};
use teaql_registry::services::{AssetService, BlobStoreGcService, BlobStoreService};
use teaql_registry_core::{service_runtime, ServiceRuntimeConfig};

#[tokio::test(flavor = "multi_thread")]
async fn test_blobstore_garbage_collection() {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime error"));
    runtime.ensure_schema().await.expect("Schema error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("gc-store"));
    blobstore.init().await.expect("Blobstore init error");

    let bs_name = format!("bs-gc-{}", uuid::Uuid::new_v4().simple());
    let bs = BlobStoreService::create(&runtime, &bs_name, "/tmp", true).await.unwrap();

    // Create an orphaned blob (not attached to any Asset)
    let orphan_payload = b"Orphaned payload to be garbage collected";
    let blob_info = blobstore.create_blob(orphan_payload).await.unwrap();

    let orphan_asset_blob = AssetService::create_asset_blob(
        &runtime,
        bs.id(),
        &blob_info.blob_ref,
        blob_info.size,
        "application/octet-stream",
        &blob_info.checksums.sha1,
        &blob_info.checksums.sha256,
        &blob_info.checksums.md5,
    )
    .await
    .unwrap();

    assert!(blobstore.exists_blob(&blob_info.blob_ref).await.unwrap());

    // Run GC
    let report = BlobStoreGcService::run_gc(&runtime, blobstore.as_ref()).await.unwrap();

    assert!(report.scanned_blobs_count >= 1);
    assert!(report.orphaned_blobs_deleted >= 1);
    assert!(report.freed_bytes >= orphan_payload.len() as i64);

    // Verify blob is removed from storage
    assert!(!blobstore.exists_blob(&blob_info.blob_ref).await.unwrap());

    // Verify record is removed from DB
    let query_db = AssetService::get_asset_blob(&runtime, orphan_asset_blob.id()).await.unwrap();
    assert!(query_db.is_none());
}
