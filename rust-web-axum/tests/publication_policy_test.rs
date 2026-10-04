//! Real PostgreSQL + observable blob storage; no mock policy/persistence layer.
mod common;

use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use teaql_registry::{
    blobstore::{BlobInfo, BlobStore, MemoryBlobStore},
    engine::publication::{publication_error_status, PublicationError},
    engine::{CargoEngine, GoModEngine, NpmEngine, NuGetEngine, PyPiEngine},
    format::npm::{NpmAttachment, NpmDist, NpmPackageDocument, NpmVersionDetail},
    services::{
        AssetService, BlobStoreService, ComponentService, RepositoryService, TenantService,
    },
};
use teaql_registry_core::{service_runtime, RepositoryConfiguration, ServiceRuntime};

struct CountingStore {
    inner: MemoryBlobStore,
    writes: AtomicUsize,
}

impl CountingStore {
    fn new() -> Self {
        Self {
            inner: MemoryBlobStore::new("publication-test"),
            writes: AtomicUsize::new(0),
        }
    }
    fn count(&self) -> usize {
        self.writes.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl BlobStore for CountingStore {
    async fn init(&self) -> Result<()> {
        self.inner.init().await
    }
    async fn create_blob(&self, data: &[u8]) -> Result<BlobInfo> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        // Yield after the preflight to expose check-then-write races reliably.
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        self.inner.create_blob(data).await
    }
    async fn read_blob(&self, reference: &str) -> Result<Bytes> {
        self.inner.read_blob(reference).await
    }
    async fn delete_blob(&self, reference: &str) -> Result<()> {
        self.inner.delete_blob(reference).await
    }
    async fn exists_blob(&self, reference: &str) -> Result<bool> {
        self.inner.exists_blob(reference).await
    }
    fn store_name(&self) -> &str {
        self.inner.store_name()
    }
}

async fn setup(
    format: &str,
    policy: &str,
) -> (ServiceRuntime, RepositoryConfiguration, CountingStore) {
    let ctx = service_runtime(common::runtime_config()).await.unwrap();
    ctx.ensure_schema().await.unwrap();
    if TenantService::find_tenant_by_code(&ctx, "default")
        .await
        .unwrap()
        .is_none()
    {
        TenantService::create_tenant_with_platform(&ctx, 1, "Default", "default", "Test")
            .await
            .unwrap();
    }
    let store_name = format!("publication-{}", uuid::Uuid::new_v4());
    let blob = BlobStoreService::create(&ctx, &store_name, "/tmp/publication-test", true)
        .await
        .unwrap();
    let repo_name = format!("publication-{}", uuid::Uuid::new_v4());
    let repo = RepositoryService::create(
        &ctx,
        &repo_name,
        &repo_name,
        "HOSTED",
        format,
        policy,
        blob.id(),
        true,
        "",
    )
    .await
    .unwrap();
    (ctx, repo, CountingStore::new())
}

fn npm_document(data: &[u8]) -> NpmPackageDocument {
    let version = "1.0.0";
    NpmPackageDocument {
        id: "@test/probe".into(),
        name: "@test/probe".into(),
        description: None,
        dist_tags: HashMap::new(),
        versions: HashMap::from([(
            version.into(),
            NpmVersionDetail {
                name: "@test/probe".into(),
                version: version.into(),
                description: None,
                dist: NpmDist {
                    shasum: String::new(),
                    tarball: "https://example.test/@test/probe/-/probe-1.0.0.tgz".into(),
                    integrity: None,
                },
            },
        )]),
        attachments: HashMap::from([(
            "@test/probe-1.0.0.tgz".into(),
            NpmAttachment {
                content_type: None,
                data: base64::engine::general_purpose::STANDARD.encode(data),
                length: Some(data.len()),
            },
        )]),
    }
}

async fn assert_empty_package_metadata(ctx: &ServiceRuntime, repo: &RepositoryConfiguration) {
    // Repository creation provisions its content repository, even READ_ONLY.
    // Rejected uploads must not create package/component/asset metadata.
    if let Some(content) = RepositoryService::get_content_repository(ctx, repo.id())
        .await
        .unwrap()
    {
        assert!(
            ComponentService::list_by_content_repository(ctx, content.id(), 10, 0)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            AssetService::list_by_content_repository_with_blobs(ctx, content.id(), 10, 0)
                .await
                .unwrap()
                .is_empty()
        );
    }
}

async fn upload(
    format: &str,
    ctx: &ServiceRuntime,
    repo: &RepositoryConfiguration,
    store: &dyn BlobStore,
    bytes: &[u8],
) -> Result<()> {
    match format {
        "CARGO" => {
            CargoEngine::upload_crate(
                ctx,
                repo,
                store,
                "probe",
                "1.0.0",
                &serde_json::json!({"name":"probe", "vers":"1.0.0"}),
                bytes,
            )
            .await
        }
        "NPM" => NpmEngine::publish_package(ctx, repo, store, &npm_document(bytes)).await,
        "GOMOD" => {
            GoModEngine::upload_artifact(
                ctx,
                repo,
                store,
                "example.test/probe",
                "v1.0.0",
                "zip",
                bytes,
            )
            .await
        }
        "PYPI" => {
            PyPiEngine::upload_file(
                ctx,
                repo,
                store,
                "probe",
                "1.0.0",
                "probe-1.0.0.tar.gz",
                bytes,
            )
            .await
        }
        "NUGET" => NuGetEngine::upload_package(ctx, repo, store, "Probe", "1.0.0", bytes).await,
        _ => panic!("Unsupported test format"),
    }
}

async fn download(
    format: &str,
    ctx: &ServiceRuntime,
    repo: &RepositoryConfiguration,
    store: &dyn BlobStore,
) -> Bytes {
    match format {
        "CARGO" => CargoEngine::get_crate_tarball(ctx, repo, store, "probe", "1.0.0")
            .await
            .unwrap()
            .unwrap(),
        "NPM" => NpmEngine::get_tarball(ctx, repo, store, "/@test/probe/-/probe-1.0.0.tgz")
            .await
            .unwrap()
            .unwrap(),
        "GOMOD" => {
            GoModEngine::get_file(ctx, repo, store, "/example.test/probe/@v/v1.0.0.zip")
                .await
                .unwrap()
                .unwrap()
                .0
        }
        "PYPI" => {
            PyPiEngine::get_package_file(ctx, repo, store, "probe-1.0.0.tar.gz")
                .await
                .unwrap()
                .unwrap()
                .0
        }
        "NUGET" => NuGetEngine::get_package_file(ctx, repo, store, "probe", "1.0.0")
            .await
            .unwrap()
            .unwrap(),
        _ => panic!("Unsupported test format"),
    }
}

async fn policy_cases(format: &str) {
    let (ctx, repo, store) = setup(format, "ALLOW_ONCE").await;
    upload(format, &ctx, &repo, &store, b"original")
        .await
        .unwrap();
    let writes = store.count();
    for payload in [b"original".as_slice(), b"replacement".as_slice()] {
        let error = upload(format, &ctx, &repo, &store, payload)
            .await
            .unwrap_err();
        assert!(
            matches!(
                error.downcast_ref::<PublicationError>(),
                Some(PublicationError::AlreadyExists(_))
            ),
            "{error:#}"
        );
        assert_eq!(
            publication_error_status(&error),
            axum::http::StatusCode::CONFLICT
        );
        assert_eq!(store.count(), writes, "Rejected upload wrote a blob");
        assert_eq!(
            download(format, &ctx, &repo, &store).await,
            &b"original"[..]
        );
    }
    if format == "CARGO" {
        use sha2::Digest;
        let index = CargoEngine::get_sparse_index(&ctx, &repo, &store, "probe")
            .await
            .unwrap()
            .unwrap();
        let record: serde_json::Value = serde_json::from_str(&index).unwrap();
        assert_eq!(
            record["cksum"],
            hex::encode(sha2::Sha256::digest(b"original"))
        );
    }

    let (ctx, repo, store) = setup(format, "READ_ONLY").await;
    let error = upload(format, &ctx, &repo, &store, b"original")
        .await
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<PublicationError>(),
        Some(PublicationError::ReadOnly)
    ));
    assert_eq!(
        publication_error_status(&error),
        axum::http::StatusCode::FORBIDDEN
    );
    assert_eq!(store.count(), 0);
    assert_empty_package_metadata(&ctx, &repo).await;

    let (ctx, repo, store) = setup(format, "ALLOW_WRITE").await;
    upload(format, &ctx, &repo, &store, b"original")
        .await
        .unwrap();
    upload(format, &ctx, &repo, &store, b"replacement")
        .await
        .unwrap();
    assert_eq!(
        download(format, &ctx, &repo, &store).await,
        &b"replacement"[..]
    );
}

async fn concurrent_case(format: &str) {
    let (ctx, repo, store) = setup(format, "ALLOW_ONCE").await;
    // Separate contexts, shared database; the lock must NOT be context-local.
    let other_ctx = service_runtime(common::runtime_config()).await.unwrap();
    let (first, second) = tokio::join!(
        upload(format, &ctx, &repo, &store, b"first"),
        upload(format, &other_ctx, &repo, &store, b"second"),
    );
    assert_ne!(
        first.is_ok(),
        second.is_ok(),
        "Exactly one concurrent publisher must win"
    );
    let (winner, loser) = if first.is_ok() {
        (b"first".as_slice(), second)
    } else {
        (b"second".as_slice(), first)
    };
    assert!(matches!(
        loser.unwrap_err().downcast_ref::<PublicationError>(),
        Some(PublicationError::AlreadyExists(_))
    ));
    assert_eq!(store.count(), if format == "CARGO" { 2 } else { 1 });
    assert_eq!(download(format, &ctx, &repo, &store).await, winner);
}

macro_rules! format_tests {
    ($policy:ident, $concurrent:ident, $format:literal) => {
        #[test]
        fn $policy() {
            common::run_with_large_stack(|| async {
                policy_cases($format).await;
            });
        }
        #[test]
        fn $concurrent() {
            common::run_with_large_stack(|| async {
                concurrent_case($format).await;
            });
        }
    };
}
format_tests!(cargo_policies, cargo_concurrent, "CARGO");
format_tests!(npm_policies, npm_concurrent, "NPM");
format_tests!(go_policies, go_concurrent, "GOMOD");
format_tests!(pypi_policies, pypi_concurrent, "PYPI");
format_tests!(nuget_policies, nuget_concurrent, "NUGET");

#[test]
fn go_and_pypi_allow_distinct_files_in_one_version() {
    common::run_with_large_stack(|| async {
        let (ctx, repo, store) = setup("GOMOD", "ALLOW_ONCE").await;
        for ext in ["mod", "info", "zip"] {
            GoModEngine::upload_artifact(
                &ctx,
                &repo,
                &store,
                "example.test/probe",
                "v1.0.0",
                ext,
                ext.as_bytes(),
            )
            .await
            .unwrap();
            assert_eq!(
                GoModEngine::get_file(
                    &ctx,
                    &repo,
                    &store,
                    &format!("/example.test/probe/@v/v1.0.0.{ext}")
                )
                .await
                .unwrap()
                .unwrap()
                .0,
                ext.as_bytes()
            );
        }
        assert_eq!(store.count(), 3);
        let (ctx, repo, store) = setup("PYPI", "ALLOW_ONCE").await;
        for filename in ["probe-1.0.0.tar.gz", "probe-1.0.0-py3-none-any.whl"] {
            PyPiEngine::upload_file(
                &ctx,
                &repo,
                &store,
                "probe",
                "1.0.0",
                filename,
                filename.as_bytes(),
            )
            .await
            .unwrap();
            assert_eq!(
                PyPiEngine::get_package_file(&ctx, &repo, &store, filename)
                    .await
                    .unwrap()
                    .unwrap()
                    .0,
                filename.as_bytes()
            );
        }
        assert_eq!(store.count(), 2);
    });
}

#[test]
fn invalid_metadata_rejected_before_writes() {
    common::run_with_large_stack(|| async {
        let (ctx, repo, store) = setup("CARGO", "ALLOW_ONCE").await;
        assert!(CargoEngine::upload_crate(
            &ctx,
            &repo,
            &store,
            "probe",
            "1.0.0",
            &serde_json::json!({"name":"other", "vers":"1.0.0"}),
            b"bad"
        )
        .await
        .is_err());
        assert_eq!(store.count(), 0);
        assert_empty_package_metadata(&ctx, &repo).await;
        let (ctx, repo, store) = setup("NPM", "ALLOW_ONCE").await;
        let mut doc = npm_document(b"original");
        doc.versions.get_mut("1.0.0").unwrap().name = "other".into();
        assert!(NpmEngine::publish_package(&ctx, &repo, &store, &doc)
            .await
            .is_err());
        assert_eq!(store.count(), 0);
        assert_empty_package_metadata(&ctx, &repo).await;
    });
}

#[test]
fn readonly_pypi_does_not_poll_upload_stream() {
    common::run_with_large_stack(|| async {
        let (ctx, repo, store) = setup("PYPI", "READ_ONLY").await;
        let stream =
            futures_util::stream::poll_fn(|_| -> std::task::Poll<Option<std::io::Result<Bytes>>> {
                panic!("READ_ONLY upload must not consume the stream");
            });
        let error = PyPiEngine::upload_file_from_stream(
            &ctx,
            &repo,
            &store,
            "probe",
            "1.0.0",
            "probe.tar.gz",
            Box::pin(stream),
        )
        .await
        .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<PublicationError>(),
            Some(PublicationError::ReadOnly)
        ));
        assert_eq!(store.count(), 0);
    });
}

fn http_upload_request(
    format: &str,
    repo: &str,
    data: &[u8],
) -> axum::http::Request<axum::body::Body> {
    use axum::{
        body::Body,
        http::{header, Method, Request},
    };
    let base = format!("/repository/{repo}");
    let (method, uri, content_type, body) = match format {
        "CARGO" => {
            let metadata =
                serde_json::to_vec(&serde_json::json!({"name":"probe", "vers":"1.0.0"})).unwrap();
            let mut body = (metadata.len() as u32).to_le_bytes().to_vec();
            body.extend_from_slice(&metadata);
            body.extend_from_slice(&(data.len() as u32).to_le_bytes());
            body.extend_from_slice(data);
            (
                Method::PUT,
                format!("{base}/api/v1/crates/new"),
                "application/octet-stream".to_owned(),
                body,
            )
        }
        "NPM" => (
            Method::PUT,
            format!("{base}/npm/@test%2Fprobe"),
            "application/json".into(),
            serde_json::to_vec(&npm_document(data)).unwrap(),
        ),
        "GOMOD" => (
            Method::PUT,
            format!("{base}/gomod/example.test/probe/@v/v1.0.0.zip"),
            "application/zip".into(),
            data.to_vec(),
        ),
        "NUGET" => (
            Method::PUT,
            format!("{base}/v3/package?id=Probe&version=1.0.0"),
            "application/octet-stream".into(),
            data.to_vec(),
        ),
        "PYPI" => {
            let boundary = "publication-policy-boundary";
            let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nprobe\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"version\"\r\n\r\n1.0.0\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"content\"; filename=\"probe-1.0.0.tar.gz\"\r\nContent-Type: application/gzip\r\n\r\n").into_bytes();
            body.extend_from_slice(data);
            body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
            (
                Method::POST,
                format!("{base}/pypi/upload"),
                format!("multipart/form-data; boundary={boundary}"),
                body,
            )
        }
        _ => panic!("Unsupported test format"),
    };
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body))
        .unwrap()
}

#[test]
fn five_protocols_return_conflict_and_forbidden_over_http() {
    common::run_with_large_stack(|| async {
        use axum::http::StatusCode;
        use std::sync::Arc;
        use teaql_registry::api::{build_app, AppState};
        use tower::ServiceExt;
        for format in ["CARGO", "NPM", "GOMOD", "PYPI", "NUGET"] {
            let (ctx, repo, store) = setup(format, "ALLOW_ONCE").await;
            let ctx = Arc::new(ctx);
            let store = Arc::new(store);
            let app = build_app(AppState::new(ctx.clone(), store.clone()));
            let first = app
                .clone()
                .oneshot(http_upload_request(format, &repo.name(), b"original"))
                .await
                .unwrap();
            assert!(
                first.status().is_success(),
                "{format} first publish: {}",
                first.status()
            );
            let writes = store.count();
            let duplicate = app
                .oneshot(http_upload_request(format, &repo.name(), b"replacement"))
                .await
                .unwrap();
            assert_eq!(duplicate.status(), StatusCode::CONFLICT, "{format}");
            let body = axum::body::to_bytes(duplicate.into_body(), 65536)
                .await
                .unwrap();
            assert!(String::from_utf8_lossy(&body).contains("ALLOW_ONCE"));
            assert_eq!(store.count(), writes, "{format} denied upload wrote blob");
            assert_eq!(
                download(format, &ctx, &repo, &*store).await,
                &b"original"[..]
            );

            let (ctx, repo, store) = setup(format, "READ_ONLY").await;
            let ctx = Arc::new(ctx);
            let store = Arc::new(store);
            let app = build_app(AppState::new(ctx.clone(), store.clone()));
            let response = app
                .oneshot(http_upload_request(format, &repo.name(), b"original"))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{format}");
            assert_eq!(store.count(), 0);
            assert_empty_package_metadata(&ctx, &repo).await;
        }
    });
}

#[test]
fn npm_document_preflights_all_attachments() {
    common::run_with_large_stack(|| async {
        let (ctx, repo, store) = setup("NPM", "ALLOW_ONCE").await;
        NpmEngine::publish_package(&ctx, &repo, &store, &npm_document(b"original"))
            .await
            .unwrap();
        let writes = store.count();
        let mut doc = npm_document(b"replacement");
        let mut detail = doc.versions["1.0.0"].clone();
        detail.version = "2.0.0".into();
        detail.dist.tarball = "https://example.test/probe-2.0.0.tgz".into();
        doc.versions.insert("2.0.0".into(), detail);
        doc.attachments.insert(
            "probe-2.0.0.tgz".into(),
            NpmAttachment {
                content_type: None,
                data: base64::engine::general_purpose::STANDARD.encode(b"new-version"),
                length: None,
            },
        );
        assert!(matches!(
            NpmEngine::publish_package(&ctx, &repo, &store, &doc)
                .await
                .unwrap_err()
                .downcast_ref::<PublicationError>(),
            Some(PublicationError::AlreadyExists(_))
        ));
        assert_eq!(
            store.count(),
            writes,
            "Existing attachment must reject the whole document before any write"
        );
        assert!(
            NpmEngine::get_tarball(&ctx, &repo, &store, "/@test/probe/-/probe-2.0.0.tgz")
                .await
                .unwrap()
                .is_none()
        );
        doc.attachments.remove("@test/probe-1.0.0.tgz");
        NpmEngine::publish_package(&ctx, &repo, &store, &doc)
            .await
            .unwrap();
        assert_eq!(store.count(), writes + 1);
    });
}
