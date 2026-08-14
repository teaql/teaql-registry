use nexus_repository_service_core_workspace::blobstore::FileBlobStore;
use tempfile::tempdir;

#[tokio::test]
async fn test_file_blobstore_create_read_and_checksums() {
    use md5::Md5;
    use sha1::Sha1;
    use sha2::{Digest, Sha256};

    let temp = tempdir().expect("tempdir failed");
    let store = FileBlobStore::new(temp.path(), "test-store");
    store.init().await.expect("init failed");

    let payload = b"Hello Nexus Rust BlobStore!";
    let info = store.create_blob(payload).await.expect("create_blob failed");

    let mut sha1_h = Sha1::new();
    sha1_h.update(payload);
    let expected_sha1 = hex::encode(sha1_h.finalize());

    let mut sha256_h = Sha256::new();
    sha256_h.update(payload);
    let expected_sha256 = hex::encode(sha256_h.finalize());

    let mut md5_h = Md5::new();
    md5_h.update(payload);
    let expected_md5 = hex::encode(md5_h.finalize());

    assert_eq!(info.size, payload.len() as i64);
    assert_eq!(info.checksums.sha1, expected_sha1);
    assert_eq!(info.checksums.sha256, expected_sha256);
    assert_eq!(info.checksums.md5, expected_md5);
    assert!(info.blob_ref.starts_with("test-store@"));

    // Read blob back
    let data = store.read_blob(&info.blob_ref).await.expect("read_blob failed");
    assert_eq!(data.as_ref(), payload);
}

#[tokio::test]
async fn test_file_blobstore_empty_blob() {
    let temp = tempdir().expect("tempdir failed");
    let store = FileBlobStore::new(temp.path(), "empty-store");
    store.init().await.expect("init failed");

    let payload = b"";
    let info = store.create_blob(payload).await.expect("create_blob failed");

    assert_eq!(info.size, 0);
    assert_eq!(info.checksums.sha1, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert_eq!(
        info.checksums.sha256,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(info.checksums.md5, "d41d8cd98f00b204e9800998ecf8427e");

    let data = store.read_blob(&info.blob_ref).await.expect("read_blob failed");
    assert_eq!(data.len(), 0);
}

#[tokio::test]
async fn test_file_blobstore_binary_payload() {
    let temp = tempdir().expect("tempdir failed");
    let store = FileBlobStore::new(temp.path(), "bin-store");
    store.init().await.expect("init failed");

    // 128KB pseudo-binary payload
    let mut payload = Vec::with_capacity(128 * 1024);
    for i in 0..(128 * 1024) {
        payload.push((i % 256) as u8);
    }

    let info = store.create_blob(&payload).await.expect("create_blob failed");
    assert_eq!(info.size, payload.len() as i64);

    let read_back = store.read_blob(&info.blob_ref).await.expect("read_blob failed");
    assert_eq!(read_back.as_ref(), payload.as_slice());
}

#[tokio::test]
async fn test_file_blobstore_delete_and_non_existent() {
    let temp = tempdir().expect("tempdir failed");
    let store = FileBlobStore::new(temp.path(), "del-store");
    store.init().await.expect("init failed");

    let info = store.create_blob(b"Ephemeral content").await.unwrap();
    assert!(store.read_blob(&info.blob_ref).await.is_ok());

    store.delete_blob(&info.blob_ref).await.expect("delete_blob failed");
    let read_res = store.read_blob(&info.blob_ref).await;
    assert!(read_res.is_err(), "Expected error when reading deleted blob");

    let non_existent = store.read_blob("del-store@00000000-0000-0000-0000-000000000000").await;
    assert!(non_existent.is_err());
}

#[tokio::test]
async fn test_file_blobstore_path_prefixing() {
    let temp = tempdir().expect("tempdir failed");
    let store = FileBlobStore::new(temp.path(), "prefix-store");
    
    let path = store.get_blob_path("ab123456-7890");
    let path_str = path.to_string_lossy();
    assert!(path_str.contains("/content/ab/ab123456-7890"));

    let short_path = store.get_blob_path("a");
    let short_path_str = short_path.to_string_lossy();
    assert!(short_path_str.contains("/content/00/a"));
}
