#![recursion_limit = "256"]

mod common;

use axum::http::{header, Method, Request, StatusCode};
use base64::Engine;
use bytes::Bytes;
use sha2::Digest;
use std::collections::HashMap;
use std::sync::Arc;
use teaql_registry::{
    api::{build_app, AppState},
    blobstore::{BlobStore, MemoryBlobStore},
    format::npm::{NpmAttachment, NpmDist, NpmPackageDocument, NpmVersionDetail},
    services::{BlobStoreService, RepositoryService, TenantService},
};
use teaql_registry_core::service_runtime;
use tower::ServiceExt;

async fn setup_multiformat_test_app() -> axum::Router {
    let config = common::runtime_config();
    let runtime = Arc::new(
        service_runtime(config)
            .await
            .expect("Runtime connect error"),
    );
    runtime.ensure_schema().await.expect("Schema init error");
    if TenantService::find_tenant_by_code(&runtime, "default")
        .await
        .unwrap()
        .is_none()
    {
        TenantService::create_tenant_with_platform(
            &runtime,
            1,
            "Default Tenant",
            "default",
            "Test tenant",
        )
        .await
        .unwrap();
    }

    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("multi-blobs"));
    blobstore.init().await.expect("Blobstore init error");

    let bs_list = BlobStoreService::list(&runtime).await.unwrap();
    let bs = if let Some(b) = bs_list.into_iter().find(|b| b.name() == "default") {
        b
    } else {
        BlobStoreService::create(&runtime, "default", "/tmp/blobs/default", true)
            .await
            .unwrap()
    };

    let repos = RepositoryService::list(&runtime).await.unwrap();
    let format_repos = [
        ("npm-hosted", "NPM"),
        ("pypi-hosted", "PYPI"),
        ("gomod-hosted", "GOMOD"),
        ("cargo-hosted", "CARGO"),
        ("nuget-hosted", "NUGET"),
        ("swift-hosted", "SWIFT"),
        ("dart-hosted", "DART"),
        ("rubygems-hosted", "RUBYGEMS"),
        ("composer-hosted", "COMPOSER"),
        ("conan-hosted", "CONAN"),
        ("hex-hosted", "HEX"),
    ];

    for (name, fmt) in format_repos {
        if !repos.iter().any(|r| r.name() == name) {
            RepositoryService::create(
                &runtime,
                name,
                name,
                "HOSTED",
                fmt,
                "ALLOW_WRITE",
                bs.id(),
                true,
                "",
            )
            .await
            .unwrap();
        }
    }

    build_app(AppState::new(runtime, blobstore))
}

fn dart_package_archive(name: &str, version: &str) -> Vec<u8> {
    use flate2::{write::GzEncoder, Compression};

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = tar::Builder::new(&mut encoder);
        let pubspec = format!(
            "name: {name}\nversion: {version}\ndescription: TeaQL Dart registry probe\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\n"
        );
        let mut header = tar::Header::new_gnu();
        header.set_size(pubspec.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "pubspec.yaml", pubspec.as_bytes())
            .unwrap();
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn ruby_gem_archive(name: &str, version: &str) -> Vec<u8> {
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    let metadata = format!(
        "--- !ruby/object:Gem::Specification\nname: {name}\nversion: !ruby/object:Gem::Version\n  version: {version}\nplatform: ruby\n"
    );
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(metadata.as_bytes()).unwrap();
    let metadata_gz = encoder.finish().unwrap();
    let mut gem = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut gem);
        let mut header = tar::Header::new_gnu();
        header.set_size(metadata_gz.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "metadata.gz", &metadata_gz[..])
            .unwrap();
        builder.finish().unwrap();
    }
    gem
}

fn composer_package_archive(name: &str) -> Vec<u8> {
    use std::io::{Cursor, Write};
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "package/composer.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer
        .write_all(
            serde_json::json!({
                "name": name,
                "description": "TeaQL Composer registry probe",
                "type": "library",
                "autoload": { "psr-4": { "TeaQL\\Probe\\": "src/" } }
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap();
    writer.finish().unwrap().into_inner()
}

fn hex_package_archive(name: &str, version: &str) -> Vec<u8> {
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    let metadata = format!(
        "{{<<\"app\">>,<<\"{name}\">>}}.\n\
         {{<<\"build_tools\">>,[<<\"mix\">>]}}.\n\
         {{<<\"description\">>,<<\"TeaQL Hex registry probe\">>}}.\n\
         {{<<\"files\">>,[<<\"mix.exs\">>]}}.\n\
         {{<<\"licenses\">>,[<<\"Apache-2.0\">>]}}.\n\
         {{<<\"name\">>,<<\"{name}\">>}}.\n\
         {{<<\"requirements\">>,[]}}.\n\
         {{<<\"version\">>,<<\"{version}\">>}}.\n"
    );
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"hex package contents").unwrap();
    let contents = encoder.finish().unwrap();
    let mut inner = sha2::Sha256::new();
    inner.update(b"3");
    inner.update(metadata.as_bytes());
    inner.update(&contents);
    let checksum = hex::encode_upper(inner.finalize());
    let mut archive = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut archive);
        for (path, data) in [
            ("VERSION", b"3".as_slice()),
            ("CHECKSUM", checksum.as_bytes()),
            ("metadata.config", metadata.as_bytes()),
            ("contents.tar.gz", contents.as_slice()),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, path, data).unwrap();
        }
        builder.finish().unwrap();
    }
    archive
}

#[test]
fn test_dart_pub_registry_lifecycle() {
    common::run_with_large_stack(dart_pub_registry_lifecycle_body);
}

async fn dart_pub_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;
    let package = format!("teaql_probe_{}", uuid::Uuid::new_v4().simple());
    let archive = dart_package_archive(&package, "1.2.3");

    let new_upload = Request::builder()
        .uri("/repository/dart-hosted/dart/api/packages/versions/new")
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(new_upload).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let boundary = "teaql-dart-boundary";
    let mut upload_body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"package.tar.gz\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .into_bytes();
    upload_body.extend_from_slice(&archive);
    upload_body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let upload = Request::builder()
        .method(Method::POST)
        .uri("/repository/dart-hosted/dart/api/packages/versions/newUpload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::from(upload_body.clone()))
        .unwrap();
    let response = app.clone().oneshot(upload).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let finalize_url = response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_string();
    assert!(finalize_url.contains("newUpload/complete"));
    let finalize = Request::builder()
        .uri(
            finalize_url
                .strip_prefix("https://registry.example.test:7443")
                .unwrap(),
        )
        .body(axum::body::Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(finalize).await.unwrap().status(),
        StatusCode::OK
    );

    let duplicate = Request::builder()
        .method(Method::POST)
        .uri("/repository/dart-hosted/dart/api/packages/versions/newUpload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(axum::body::Body::from(upload_body))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(duplicate).await.unwrap().status(),
        StatusCode::CONFLICT
    );

    let versions = Request::builder()
        .uri(format!(
            "/repository/dart-hosted/dart/api/packages/{package}"
        ))
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(versions).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/vnd.pub.v2+json"
    );
    let json: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(json["latest"]["version"], "1.2.3");
    assert_eq!(json["latest"]["pubspec"]["name"], package);
    assert_eq!(
        json["latest"]["archive_sha256"],
        hex::encode(sha2::Sha256::digest(&archive))
    );
    assert!(json["latest"]["archive_url"]
        .as_str()
        .unwrap()
        .starts_with("https://registry.example.test:7443/"));

    let download = Request::builder()
        .uri(format!(
            "/repository/dart-hosted/dart/packages/{package}/versions/1.2.3.tar.gz"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .as_ref(),
        archive
    );
}

#[test]
fn test_rubygems_registry_lifecycle() {
    common::run_with_large_stack(rubygems_registry_lifecycle_body);
}

async fn rubygems_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;
    let gem_name = format!("teaql_probe_{}", uuid::Uuid::new_v4().simple());
    let gem = ruby_gem_archive(&gem_name, "1.2.3");
    let filename = format!("{gem_name}-1.2.3.gem");

    let push = Request::builder()
        .method(Method::POST)
        .uri("/repository/rubygems-hosted/rubygems/api/v1/gems")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(axum::body::Body::from(gem.clone()))
        .unwrap();
    let response = app.clone().oneshot(push).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let versions = Request::builder()
        .uri("/repository/rubygems-hosted/rubygems/versions")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(versions).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("repr-digest"));
    let etag = response.headers()[header::ETAG]
        .to_str()
        .unwrap()
        .to_string();
    let versions = String::from_utf8(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(versions.contains(&format!("{gem_name} 1.2.3 ")));

    let conditional = Request::builder()
        .uri("/repository/rubygems-hosted/rubygems/versions")
        .header(header::IF_NONE_MATCH, etag)
        .body(axum::body::Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(conditional).await.unwrap().status(),
        StatusCode::NOT_MODIFIED
    );

    let ranged = Request::builder()
        .uri("/repository/rubygems-hosted/rubygems/versions")
        .header(header::RANGE, "bytes=5-")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(ranged).await.unwrap();
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert!(response.headers().contains_key(header::CONTENT_RANGE));

    let info = Request::builder()
        .uri(format!(
            "/repository/rubygems-hosted/rubygems/info/{gem_name}"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(info).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let info = String::from_utf8(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(info.contains(&format!(
        "checksum:{}",
        hex::encode(sha2::Sha256::digest(&gem))
    )));

    let download = Request::builder()
        .uri(format!(
            "/repository/rubygems-hosted/rubygems/gems/{filename}"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .as_ref(),
        gem
    );
}

fn swift_package_archive(package_id: &str) -> Vec<u8> {
    use std::io::{Cursor, Write};
    use zip::{write::SimpleFileOptions, ZipWriter};

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            format!("{package_id}/Package.swift"),
            SimpleFileOptions::default(),
        )
        .unwrap();
    writer
        .write_all(
            br#"// swift-tools-version: 5.9
import PackageDescription
let package = Package(
    name: "TeaQLProbe",
    products: [.library(name: "TeaQLProbe", targets: ["TeaQLProbe"])],
    targets: [.target(name: "TeaQLProbe")]
)
"#,
        )
        .unwrap();
    writer
        .start_file(
            format!("{package_id}/Sources/TeaQLProbe/TeaQLProbe.swift"),
            SimpleFileOptions::default(),
        )
        .unwrap();
    writer
        .write_all(b"public let teaqlProbe = \"ok\"\n")
        .unwrap();
    writer.finish().unwrap().into_inner()
}

#[test]
fn test_swift_package_registry_lifecycle() {
    common::run_with_large_stack(swift_package_registry_lifecycle_body);
}

async fn swift_package_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;
    let package = format!("probe{}", uuid::Uuid::new_v4().simple());
    let package_id = format!("teaql.{package}");
    let archive = swift_package_archive(&package_id);
    let boundary = "teaql-swift-registry-boundary";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"source-archive\"; filename=\"{package_id}-1.2.3.zip\"\r\nContent-Type: application/zip\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(&archive);
    body.extend_from_slice(
        format!(
            "\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"metadata\"\r\nContent-Type: application/json\r\n\r\n{{\"description\":\"native Swift package\",\"repositoryURLs\":[\"https://github.com/teaql/{package}\"]}}\r\n--{boundary}--\r\n"
        )
        .as_bytes(),
    );

    let publish_uri = format!("/repository/swift-hosted/swift/teaql/{package}/1.2.3");
    let publish = Request::builder()
        .method(Method::PUT)
        .uri(&publish_uri)
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(axum::body::Body::from(body.clone()))
        .unwrap();
    let response = app.clone().oneshot(publish).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()["content-version"], "1");

    let duplicate = Request::builder()
        .method(Method::PUT)
        .uri(&publish_uri)
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(axum::body::Body::from(body))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(duplicate).await.unwrap().status(),
        StatusCode::CONFLICT
    );

    let list = Request::builder()
        .uri(format!("/repository/swift-hosted/swift/teaql/{package}"))
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(list).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list_json: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        list_json["releases"]["1.2.3"]["url"],
        format!("https://registry.example.test:7443/repository/swift-hosted/swift/teaql/{package}/1.2.3")
    );

    let metadata = Request::builder()
        .uri(&publish_uri)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(metadata).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let metadata_json: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(metadata_json["id"], package_id);
    assert_eq!(metadata_json["version"], "1.2.3");
    assert_eq!(
        metadata_json["resources"][0]["checksum"],
        hex::encode(sha2::Sha256::digest(&archive))
    );

    let manifest = Request::builder()
        .uri(format!("{publish_uri}/Package.swift"))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(manifest).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "text/x-swift");
    let manifest_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert!(String::from_utf8_lossy(&manifest_bytes).contains("TeaQLProbe"));

    let download = Request::builder()
        .uri(format!(
            "/repository/swift-hosted/swift/teaql/{package}/1.2.3.zip"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("digest"));
    let downloaded = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(downloaded.as_ref(), archive);

    let identifiers = Request::builder()
        .uri(format!(
            "/repository/swift-hosted/swift/identifiers?url=https%3A%2F%2Fgithub.com%2Fteaql%2F{package}"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(identifiers).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let identifiers_json: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(identifiers_json["identifiers"][0], package_id);
}

#[test]
fn test_npm_registry_lifecycle() {
    common::run_with_large_stack(npm_registry_lifecycle_body);
}

async fn npm_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;

    let package_name = format!("my-ui-lib-{}", uuid::Uuid::new_v4().simple());
    let tarball_filename = format!("{}-1.0.0.tgz", package_name);
    let fake_tgz_data = b"fake-tarball-gzip-binary-content-for-npm";
    let encoded_data = base64::engine::general_purpose::STANDARD.encode(fake_tgz_data);

    let mut versions = HashMap::new();
    versions.insert(
        "1.0.0".to_string(),
        NpmVersionDetail {
            name: package_name.clone(),
            version: "1.0.0".to_string(),
            description: Some("UI Library".to_string()),
            dist: NpmDist {
                shasum: "fake-sha1".to_string(),
                tarball: format!(
                    "http://localhost:8081/repository/npm-hosted/npm/{}/-/{}",
                    package_name, tarball_filename
                ),
                integrity: None,
            },
        },
    );

    let mut attachments = HashMap::new();
    attachments.insert(
        tarball_filename.clone(),
        NpmAttachment {
            content_type: Some("application/gzip".to_string()),
            data: encoded_data,
            length: Some(fake_tgz_data.len()),
        },
    );

    let mut dist_tags = HashMap::new();
    dist_tags.insert("latest".to_string(), "1.0.0".to_string());

    let publish_doc = NpmPackageDocument {
        id: package_name.clone(),
        name: package_name.clone(),
        description: Some("UI Library".to_string()),
        dist_tags,
        versions,
        attachments,
    };

    // 1. Publish NPM package: PUT /repository/npm-hosted/npm/:package_name
    let publish_json = serde_json::to_vec(&publish_doc).unwrap();
    let put_req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/repository/npm-hosted/npm/{}", package_name))
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(publish_json))
        .unwrap();
    let put_resp = app.clone().oneshot(put_req).await.unwrap();
    assert_eq!(put_resp.status(), StatusCode::CREATED);

    // 2. Fetch package document: GET /repository/npm-hosted/npm/:package_name
    let get_doc_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/npm-hosted/npm/{}", package_name))
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::empty())
        .unwrap();
    let get_doc_resp = app.clone().oneshot(get_doc_req).await.unwrap();
    assert_eq!(get_doc_resp.status(), StatusCode::OK);
    let doc_bytes = axum::body::to_bytes(get_doc_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let fetched_doc: NpmPackageDocument = serde_json::from_slice(&doc_bytes).unwrap();
    assert_eq!(fetched_doc.name, package_name);
    assert_eq!(fetched_doc.dist_tags.get("latest").unwrap(), "1.0.0");
    assert_eq!(
        fetched_doc.versions["1.0.0"].dist.tarball,
        format!(
            "https://registry.example.test:7443/repository/npm-hosted/npm/{}/-/{}",
            package_name, tarball_filename
        )
    );

    // 3. Download tarball: GET /repository/npm-hosted/npm/:package_name/-/:tarball
    let get_tgz_req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/repository/npm-hosted/npm/{}/-/{}",
            package_name, tarball_filename
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let get_tgz_resp = app.clone().oneshot(get_tgz_req).await.unwrap();
    assert_eq!(get_tgz_resp.status(), StatusCode::OK);
    let tgz_bytes = axum::body::to_bytes(get_tgz_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(tgz_bytes.as_ref(), fake_tgz_data);
}

#[test]
fn test_scoped_npm_registry_lifecycle() {
    common::run_with_large_stack(scoped_npm_registry_lifecycle_body);
}

async fn scoped_npm_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;
    let name = format!("scoped-{}", uuid::Uuid::new_v4().simple());
    let package_name = format!("@teaql/{name}");
    let encoded_name = package_name.replace('/', "%2F");
    let attachment_name = format!("@teaql/{name}-1.0.0.tgz");
    let canonical_filename = format!("{name}-1.0.0.tgz");
    let archive = b"scoped-npm-tarball";
    let mut versions = HashMap::new();
    versions.insert(
        "1.0.0".to_string(),
        NpmVersionDetail {
            name: package_name.clone(),
            version: "1.0.0".to_string(),
            description: None,
            dist: NpmDist {
                shasum: String::new(),
                tarball: format!(
                    "http://localhost:8081/repository/npm-hosted/npm/{encoded_name}/-/{canonical_filename}"
                ),
                integrity: None,
            },
        },
    );
    let mut attachments = HashMap::new();
    attachments.insert(
        attachment_name,
        NpmAttachment {
            content_type: Some("application/gzip".to_string()),
            data: base64::engine::general_purpose::STANDARD.encode(archive),
            length: Some(archive.len()),
        },
    );
    let document = NpmPackageDocument {
        id: package_name.clone(),
        name: package_name.clone(),
        description: None,
        dist_tags: HashMap::from([("latest".to_string(), "1.0.0".to_string())]),
        versions,
        attachments,
    };
    let publish = Request::builder()
        .method(Method::PUT)
        .uri(format!("/repository/npm-hosted/npm/{encoded_name}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            serde_json::to_vec(&document).unwrap(),
        ))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(publish).await.unwrap().status(),
        StatusCode::CREATED
    );

    let metadata = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/npm-hosted/npm/{encoded_name}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(metadata).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let returned: NpmPackageDocument = serde_json::from_slice(&bytes).unwrap();
    assert!(returned.versions["1.0.0"]
        .dist
        .tarball
        .ends_with(&format!("/{encoded_name}/-/{canonical_filename}")));

    let download = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/repository/npm-hosted/npm/{encoded_name}/-/{canonical_filename}"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(bytes.as_ref(), archive);
}

#[test]
fn test_pypi_registry_lifecycle() {
    common::run_with_large_stack(pypi_registry_lifecycle_body);
}

async fn pypi_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;

    let proj_name = format!("flask-util-{}", uuid::Uuid::new_v4().simple());
    let filename = format!("{}-2.0.0-py3-none-any.whl", proj_name);
    // Keep the distribution above Axum's 2 MiB buffered extractor default so
    // this lifecycle test protects the streaming multipart implementation.
    let whl_content = vec![0x50_u8; 3 * 1024 * 1024];

    // Multipart upload payload
    let boundary = "------------------------Boundary123456789";
    let mut body = format!(
        "--{0}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\n{1}\r\n\
         --{0}\r\nContent-Disposition: form-data; name=\"version\"\r\n\r\n2.0.0\r\n\
         --{0}\r\nContent-Disposition: form-data; name=\"content\"; filename=\"{2}\"\r\nContent-Type: application/x-wheel+zip\r\n\r\n",
        boundary, proj_name, filename
    )
    .into_bytes();
    body.extend_from_slice(&whl_content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    // 1. Upload distribution: POST /repository/pypi-hosted/pypi/upload
    let post_req = Request::builder()
        .method(Method::POST)
        .uri("/repository/pypi-hosted/pypi/upload")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(axum::body::Body::from(body))
        .unwrap();
    let post_resp = app.clone().oneshot(post_req).await.unwrap();
    assert_eq!(post_resp.status(), StatusCode::OK);

    // 2. Simple Index Root: GET /repository/pypi-hosted/simple/
    let root_req = Request::builder()
        .method(Method::GET)
        .uri("/repository/pypi-hosted/simple/")
        .body(axum::body::Body::empty())
        .unwrap();
    let root_resp = app.clone().oneshot(root_req).await.unwrap();
    assert_eq!(root_resp.status(), StatusCode::OK);
    let root_html = String::from_utf8(
        axum::body::to_bytes(root_resp.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(root_html.contains(&format!("{}/", proj_name)));

    // 3. Simple Package Index: GET /repository/pypi-hosted/simple/:project/
    let pkg_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/pypi-hosted/simple/{}/", proj_name))
        .body(axum::body::Body::empty())
        .unwrap();
    let pkg_resp = app.clone().oneshot(pkg_req).await.unwrap();
    assert_eq!(pkg_resp.status(), StatusCode::OK);
    let pkg_html = String::from_utf8(
        axum::body::to_bytes(pkg_resp.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(pkg_html.contains(&filename));

    // 4. Download distribution: GET /repository/pypi-hosted/packages/:filename
    let dl_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/pypi-hosted/packages/{}", filename))
        .body(axum::body::Body::empty())
        .unwrap();
    let dl_resp = app.clone().oneshot(dl_req).await.unwrap();
    assert_eq!(dl_resp.status(), StatusCode::OK);
    let dl_bytes = axum::body::to_bytes(dl_resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(dl_bytes.as_ref(), whl_content.as_slice());
}

#[test]
fn test_gomod_registry_lifecycle() {
    common::run_with_large_stack(gomod_registry_lifecycle_body);
}

async fn gomod_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;

    let module = format!("github.com/example/lib-{}", uuid::Uuid::new_v4().simple());
    let mod_content = b"module github.com/example/lib\n\ngo 1.22\n";
    let zip_content = b"fake-go-module-zip-binary";
    let info_content = br#"{"Version":"v1.0.0","Time":"2026-09-28T10:00:00Z"}"#;

    // 1. Upload all Go proxy artifacts, including the authoritative .info metadata.
    let put_mod_req = Request::builder()
        .method(Method::PUT)
        .uri(format!(
            "/repository/gomod-hosted/gomod/{}/@v/v1.0.0.mod",
            module
        ))
        .body(axum::body::Body::from(Bytes::from_static(mod_content)))
        .unwrap();
    let put_mod_resp = app.clone().oneshot(put_mod_req).await.unwrap();
    assert_eq!(put_mod_resp.status(), StatusCode::CREATED);

    let put_zip_req = Request::builder()
        .method(Method::PUT)
        .uri(format!(
            "/repository/gomod-hosted/gomod/{}/@v/v1.0.0.zip",
            module
        ))
        .body(axum::body::Body::from(Bytes::from_static(zip_content)))
        .unwrap();
    let put_zip_resp = app.clone().oneshot(put_zip_req).await.unwrap();
    assert_eq!(put_zip_resp.status(), StatusCode::CREATED);

    let put_info_req = Request::builder()
        .method(Method::PUT)
        .uri(format!(
            "/repository/gomod-hosted/gomod/{}/@v/v1.0.0.info",
            module
        ))
        .body(axum::body::Body::from(Bytes::from_static(info_content)))
        .unwrap();
    let put_info_resp = app.clone().oneshot(put_info_req).await.unwrap();
    assert_eq!(put_info_resp.status(), StatusCode::CREATED);

    // 2. Query @v/list
    let list_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/gomod-hosted/gomod/{}/@v/list", module))
        .body(axum::body::Body::empty())
        .unwrap();
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_txt = String::from_utf8(
        axum::body::to_bytes(list_resp.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(list_txt.contains("v1.0.0"));

    // 3. Query @v/v1.0.0.info
    let info_req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/repository/gomod-hosted/gomod/{}/@v/v1.0.0.info",
            module
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let info_resp = app.clone().oneshot(info_req).await.unwrap();
    assert_eq!(info_resp.status(), StatusCode::OK);
    assert_eq!(
        info_resp.headers()[header::CONTENT_TYPE],
        "application/json"
    );
    let info_bytes = axum::body::to_bytes(info_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(info_bytes.as_ref(), info_content);

    // 4. Download @v/v1.0.0.mod & zip
    let dl_mod_req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/repository/gomod-hosted/gomod/{}/@v/v1.0.0.mod",
            module
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let dl_mod_resp = app.clone().oneshot(dl_mod_req).await.unwrap();
    assert_eq!(dl_mod_resp.status(), StatusCode::OK);
    let dl_mod_bytes = axum::body::to_bytes(dl_mod_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(dl_mod_bytes.as_ref(), mod_content);
}

#[test]
fn test_cargo_registry_lifecycle() {
    common::run_with_large_stack(cargo_registry_lifecycle_body);
}

async fn cargo_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;

    // 1. Check config.json
    let cfg_req = Request::builder()
        .method(Method::GET)
        .uri("/repository/cargo-hosted/cargo/index/config.json")
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::empty())
        .unwrap();
    let cfg_resp = app.clone().oneshot(cfg_req).await.unwrap();
    assert_eq!(cfg_resp.status(), StatusCode::OK);
    let cfg_body = axum::body::to_bytes(cfg_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let cfg: serde_json::Value = serde_json::from_slice(&cfg_body).unwrap();
    assert_eq!(
        cfg["dl"],
        "https://registry.example.test:7443/repository/cargo-hosted/api/v1/crates/{crate}/{version}/download"
    );
    assert_eq!(cfg["auth-required"], true);

    // 2. Publish crate via PUT /repository/cargo-hosted/api/v1/crates/new
    let crate_name = format!("cr-{}", uuid::Uuid::new_v4().simple());
    let json_meta = serde_json::json!({
        "name": crate_name,
        "vers": "0.1.0",
        "deps": [{
            "name": "serde",
            "version_req": "^1",
            "features": ["derive"],
            "optional": false,
            "default_features": true,
            "target": null,
            "kind": "normal",
            "registry": "https://github.com/rust-lang/crates.io-index",
            "explicit_name_in_toml": null
        }],
        "features": {"serde": ["serde/derive"]},
        "authors": ["Nexus Author"],
        "description": "Sample crate"
    });
    let json_bytes = serde_json::to_vec(&json_meta).unwrap();
    let crate_tarball = b"fake-cargo-crate-tarball-bytes";

    let mut payload = Vec::new();
    payload.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    payload.extend_from_slice(&json_bytes);
    payload.extend_from_slice(&(crate_tarball.len() as u32).to_le_bytes());
    payload.extend_from_slice(crate_tarball);

    let pub_req = Request::builder()
        .method(Method::PUT)
        .uri("/repository/cargo-hosted/api/v1/crates/new")
        .body(axum::body::Body::from(payload))
        .unwrap();
    let pub_resp = app.clone().oneshot(pub_req).await.unwrap();
    assert_eq!(pub_resp.status(), StatusCode::OK);

    // 3. Check sparse index
    let idx_req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/repository/cargo-hosted/cargo/index/cr/{}",
            crate_name
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let idx_resp = app.clone().oneshot(idx_req).await.unwrap();
    assert_eq!(idx_resp.status(), StatusCode::OK);
    let idx_body = axum::body::to_bytes(idx_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let index_record: serde_json::Value = serde_json::from_slice(&idx_body).unwrap();
    assert_eq!(
        index_record["cksum"],
        hex::encode(sha2::Sha256::digest(crate_tarball))
    );
    assert_eq!(index_record["deps"][0]["name"], "serde");
    assert_eq!(index_record["deps"][0]["req"], "^1");
    assert_eq!(index_record["deps"][0]["features"][0], "derive");
    assert_eq!(index_record["features"]["serde"][0], "serde/derive");
    assert_eq!(index_record["v"], 2);

    // 4. Download crate
    let dl_req = Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/repository/cargo-hosted/api/v1/crates/{}/0.1.0/download",
            crate_name
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let dl_resp = app.clone().oneshot(dl_req).await.unwrap();
    assert_eq!(dl_resp.status(), StatusCode::OK);
    let dl_bytes = axum::body::to_bytes(dl_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(dl_bytes.as_ref(), crate_tarball);
}

#[test]
fn test_nuget_registry_lifecycle() {
    common::run_with_large_stack(nuget_registry_lifecycle_body);
}

async fn nuget_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;

    // 1. Service Index: GET /repository/nuget-hosted/v3/index.json
    let index_req = Request::builder()
        .method(Method::GET)
        .uri("/repository/nuget-hosted/v3/index.json")
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::empty())
        .unwrap();
    let index_resp = app.clone().oneshot(index_req).await.unwrap();
    assert_eq!(index_resp.status(), StatusCode::OK);
    let index_body = axum::body::to_bytes(index_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let index: serde_json::Value = serde_json::from_slice(&index_body).unwrap();
    assert!(index["resources"]
        .as_array()
        .unwrap()
        .iter()
        .all(|resource| {
            resource["@id"]
                .as_str()
                .unwrap()
                .starts_with("https://registry.example.test:7443/repository/nuget-hosted/")
        }));

    // 2. Push NuGet package: PUT /repository/nuget-hosted/v3/package
    let fake_nupkg = b"fake-nuget-package-zip-content";
    let push_req = Request::builder()
        .method(Method::PUT)
        .uri("/repository/nuget-hosted/v3/package?id=sample-package&version=1.0.0")
        .body(axum::body::Body::from(Bytes::from_static(fake_nupkg)))
        .unwrap();
    let push_resp = app.clone().oneshot(push_req).await.unwrap();
    assert_eq!(push_resp.status(), StatusCode::CREATED);

    // 3. Check package versions: GET /repository/nuget-hosted/v3/flatcontainer/sample-package/index.json
    let ver_req = Request::builder()
        .method(Method::GET)
        .uri("/repository/nuget-hosted/v3/flatcontainer/sample-package/index.json")
        .body(axum::body::Body::empty())
        .unwrap();
    let ver_resp = app.clone().oneshot(ver_req).await.unwrap();
    assert_eq!(ver_resp.status(), StatusCode::OK);

    // 4. Download nupkg: GET /repository/nuget-hosted/v3/flatcontainer/sample-package/1.0.0/sample-package.1.0.0.nupkg
    let dl_req = Request::builder()
        .method(Method::GET)
        .uri("/repository/nuget-hosted/v3/flatcontainer/sample-package/1.0.0/sample-package.1.0.0.nupkg")
        .body(axum::body::Body::empty())
        .unwrap();
    let dl_resp = app.clone().oneshot(dl_req).await.unwrap();
    assert_eq!(dl_resp.status(), StatusCode::OK);
    let dl_bytes = axum::body::to_bytes(dl_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(dl_bytes.as_ref(), fake_nupkg);
}

#[test]
fn test_nuget_native_multipart_upload_strips_envelope() {
    common::run_with_large_stack(nuget_native_multipart_upload_strips_envelope_body);
}

async fn nuget_native_multipart_upload_strips_envelope_body() {
    use std::io::{Cursor, Write};
    use zip::{write::SimpleFileOptions, ZipWriter};

    let app = setup_multiformat_test_app().await;
    let package_id = format!("TeaQL.Native.{}", uuid::Uuid::new_v4().simple());
    let package_version = "1.2.3";
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(format!("{package_id}.nuspec"), SimpleFileOptions::default())
        .unwrap();
    writer
        .write_all(
            format!(
                r#"<package><metadata><id>{package_id}</id><version>{package_version}</version></metadata></package>"#
            )
            .as_bytes(),
        )
        .unwrap();
    let nupkg = writer.finish().unwrap().into_inner();
    let boundary = "teaql-nuget-native-boundary";
    let mut multipart = format!(
        "--{boundary}\r\nContent-Type: application/octet-stream\r\nContent-Disposition: form-data; name=\"package\"; filename=\"{package_id}.{package_version}.nupkg\"\r\n\r\n"
    )
    .into_bytes();
    multipart.extend_from_slice(&nupkg);
    multipart.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let push = Request::builder()
        .method(Method::PUT)
        .uri("/repository/nuget-hosted/v3/package/")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(axum::body::Body::from(multipart))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(push).await.unwrap().status(),
        StatusCode::CREATED
    );

    let id = package_id.to_ascii_lowercase();
    let download = Request::builder()
        .uri(format!(
            "/repository/nuget-hosted/v3/flatcontainer/{id}/{package_version}/{id}.{package_version}.nupkg"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let downloaded = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(downloaded.as_ref(), nupkg.as_slice());
}

#[test]
fn test_composer_registry_lifecycle() {
    common::run_with_large_stack(composer_registry_lifecycle_body);
}

async fn composer_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;
    let package = format!("teaql/probe-{}", uuid::Uuid::new_v4().simple());
    let (_, package_name) = package.split_once('/').unwrap();
    let archive = composer_package_archive(&package);
    let dist_path =
        format!("/repository/composer-hosted/composer/dist/teaql/{package_name}/1.2.3.zip");
    let publish = Request::builder()
        .method(Method::PUT)
        .uri(&dist_path)
        .body(axum::body::Body::from(archive.clone()))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(publish).await.unwrap().status(),
        StatusCode::CREATED
    );

    let duplicate = Request::builder()
        .method(Method::PUT)
        .uri(&dist_path)
        .body(axum::body::Body::from(archive.clone()))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(duplicate).await.unwrap().status(),
        StatusCode::CONFLICT
    );

    let development_dist_path =
        format!("/repository/composer-hosted/composer/dist/teaql/{package_name}/dev-main.zip");
    let publish_development = Request::builder()
        .method(Method::PUT)
        .uri(&development_dist_path)
        .body(axum::body::Body::from(archive.clone()))
        .unwrap();
    assert_eq!(
        app.clone()
            .oneshot(publish_development)
            .await
            .unwrap()
            .status(),
        StatusCode::CREATED
    );

    let root = Request::builder()
        .uri("/repository/composer-hosted/composer/packages.json")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(root).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let root: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        root["metadata-url"],
        "/repository/composer-hosted/composer/p2/%package%.json"
    );
    assert!(root["available-packages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|name| name == &package));

    let metadata = Request::builder()
        .uri(format!(
            "/repository/composer-hosted/composer/p2/{package}.json"
        ))
        .header(header::HOST, "registry.example.test:7443")
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(metadata).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let metadata: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(metadata["packages"][&package][0]["version"], "1.2.3");
    assert_eq!(metadata["packages"][&package].as_array().unwrap().len(), 1);
    assert_eq!(metadata["packages"][&package][0]["dist"]["type"], "zip");
    assert!(metadata["packages"][&package][0]["dist"]["url"]
        .as_str()
        .unwrap()
        .starts_with("https://registry.example.test:7443/"));

    let development_metadata = Request::builder()
        .uri(format!(
            "/repository/composer-hosted/composer/p2/{package}~dev.json"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(development_metadata).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let development_metadata: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        development_metadata["packages"][&package][0]["version"],
        "dev-main"
    );

    let download = Request::builder()
        .uri(&dist_path)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .as_ref(),
        archive
    );
}

#[test]
fn test_conan_v2_registry_lifecycle() {
    common::run_with_large_stack(conan_v2_registry_lifecycle_body);
}

async fn conan_v2_registry_lifecycle_body() {
    let app = setup_multiformat_test_app().await;
    let package = format!("probe{}", uuid::Uuid::new_v4().simple());
    let recipe_revision = "0123456789abcdef";
    let base = format!(
        "/repository/conan-hosted/conan/v2/conans/{package}/1.2.3/_/_/revisions/{recipe_revision}"
    );

    let ping = Request::builder()
        .uri("/repository/conan-hosted/conan/v1/ping")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(ping).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["x-conan-server-capabilities"],
        "revisions"
    );

    for (file, contents) in [
        (
            "conanfile.py",
            format!("from conan import ConanFile\nclass Probe(ConanFile):\n    name = \"{package}\"\n    version = \"1.2.3\"\n"),
        ),
        ("conanmanifest.txt", "manifest".to_string()),
    ] {
        let request = Request::builder()
            .method(Method::PUT)
            .uri(format!("{base}/files/{file}"))
            .body(axum::body::Body::from(contents))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::CREATED
        );
    }

    let revisions = Request::builder()
        .uri(format!(
            "/repository/conan-hosted/conan/v2/conans/{package}/1.2.3/_/_/revisions"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(revisions).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let revisions: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(revisions["revisions"][0]["revision"], recipe_revision);

    let snapshot = Request::builder()
        .uri(format!("{base}/files"))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(snapshot).await.unwrap();
    let snapshot: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(snapshot["files"].get("conanfile.py").is_some());

    let package_id = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
    let package_revision = "fedcba9876543210";
    let package_base = format!("{base}/packages/{package_id}/revisions/{package_revision}/files");
    for (file, contents) in [
        ("conaninfo.txt", "[settings]\nos=Linux\n"),
        ("conanmanifest.txt", "manifest"),
        ("conan_package.tgz", "binary-package"),
    ] {
        let request = Request::builder()
            .method(Method::PUT)
            .uri(format!("{package_base}/{file}"))
            .body(axum::body::Body::from(contents))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::CREATED
        );
    }

    let latest = Request::builder()
        .uri(format!("{base}/packages/{package_id}/latest"))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(latest).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let latest: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(latest["revision"], package_revision);

    let package_search = Request::builder()
        .uri(format!("{base}/search"))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(package_search).await.unwrap();
    let package_search: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(package_search[package_id]["content"]
        .as_str()
        .unwrap()
        .contains("os=Linux"));

    let download = Request::builder()
        .uri(format!("{package_base}/conan_package.tgz"))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .as_ref(),
        b"binary-package"
    );
}

#[derive(Clone, PartialEq, prost::Message)]
struct TestHexSigned {
    #[prost(bytes = "vec", required, tag = "1")]
    payload: Vec<u8>,
    #[prost(bytes = "vec", optional, tag = "2")]
    signature: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct TestHexPackageRegistry {
    #[prost(message, repeated, tag = "1")]
    releases: Vec<TestHexRelease>,
    #[prost(string, required, tag = "2")]
    name: String,
    #[prost(string, required, tag = "3")]
    repository: String,
}

#[derive(Clone, PartialEq, prost::Message)]
struct TestHexRelease {
    #[prost(string, required, tag = "1")]
    version: String,
    #[prost(bytes = "vec", required, tag = "2")]
    inner_checksum: Vec<u8>,
    #[prost(bytes = "vec", optional, tag = "5")]
    outer_checksum: Option<Vec<u8>>,
}

#[test]
fn test_hex_registry_v2_lifecycle_and_signature() {
    common::run_with_large_stack(hex_registry_v2_lifecycle_and_signature_body);
}

async fn hex_registry_v2_lifecycle_and_signature_body() {
    use flate2::read::GzDecoder;
    use prost::Message;
    use rsa::{
        pkcs1v15::{Signature, VerifyingKey},
        pkcs8::DecodePublicKey,
        signature::Verifier,
        RsaPublicKey,
    };
    use sha2::Sha512;
    use std::io::Read;

    let app = setup_multiformat_test_app().await;
    let package = format!("probe_{}", uuid::Uuid::new_v4().simple());
    let archive = hex_package_archive(&package, "1.2.3");
    let publish = Request::builder()
        .method(Method::POST)
        .uri("/repository/hex-hosted/hex/api/publish")
        .body(axum::body::Body::from(archive.clone()))
        .unwrap();
    let response = app.clone().oneshot(publish).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let public_key = Request::builder()
        .uri("/repository/hex-hosted/hex/repo/public_key")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(public_key).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let public_key = String::from_utf8(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    let registry = Request::builder()
        .uri(format!(
            "/repository/hex-hosted/hex/repo/packages/{package}"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.clone().oneshot(registry).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    // Hex registry files are gzip containers, not HTTP transfer-encoded bodies.
    // Declaring Content-Encoding would make HTTP clients decompress them before
    // the Hex client gets a chance to decode the registry container itself.
    assert!(response.headers().get(header::CONTENT_ENCODING).is_none());
    let etag = response.headers()[header::ETAG]
        .to_str()
        .unwrap()
        .to_string();
    let compressed = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let mut decoder = GzDecoder::new(compressed.as_ref());
    let mut signed_bytes = Vec::new();
    decoder.read_to_end(&mut signed_bytes).unwrap();
    let signed = TestHexSigned::decode(signed_bytes.as_slice()).unwrap();
    let signature = Signature::try_from(signed.signature.as_deref().unwrap()).unwrap();
    let public_key = RsaPublicKey::from_public_key_pem(&public_key).unwrap();
    let verifying_key = VerifyingKey::<Sha512>::new(public_key);
    verifying_key.verify(&signed.payload, &signature).unwrap();
    let registry = TestHexPackageRegistry::decode(signed.payload.as_slice()).unwrap();
    assert_eq!(registry.name, package);
    assert_eq!(registry.repository, "hex-hosted");
    assert_eq!(registry.releases[0].version, "1.2.3");
    assert_eq!(
        registry.releases[0].outer_checksum.as_deref(),
        Some(sha2::Sha256::digest(&archive).as_slice())
    );

    for path in ["names", "versions"] {
        let request = Request::builder()
            .uri(format!("/repository/hex-hosted/hex/repo/{path}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get(header::CONTENT_ENCODING).is_none());
        let compressed = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let mut decoder = GzDecoder::new(compressed.as_ref());
        let mut signed_bytes = Vec::new();
        decoder.read_to_end(&mut signed_bytes).unwrap();
        let signed = TestHexSigned::decode(signed_bytes.as_slice()).unwrap();
        let signature = Signature::try_from(signed.signature.as_deref().unwrap()).unwrap();
        verifying_key.verify(&signed.payload, &signature).unwrap();
    }

    let conditional = Request::builder()
        .uri(format!(
            "/repository/hex-hosted/hex/repo/packages/{package}"
        ))
        .header(header::IF_NONE_MATCH, etag)
        .body(axum::body::Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(conditional).await.unwrap().status(),
        StatusCode::NOT_MODIFIED
    );

    let download = Request::builder()
        .uri(format!(
            "/repository/hex-hosted/hex/repo/tarballs/{package}-1.2.3.tar"
        ))
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(download).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .as_ref(),
        archive
    );
}
