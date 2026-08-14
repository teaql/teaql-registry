use axum::http::{header, Method, Request, StatusCode};
use base64::Engine;
use bytes::Bytes;
use nexus_repository_service_core::{service_runtime, ServiceRuntimeConfig};
use nexus_repository_service_core_workspace::{
    api::{build_app, AppState},
    blobstore::{BlobStore, S3BlobStore},
    format::npm::{NpmAttachment, NpmDist, NpmPackageDocument, NpmVersionDetail},
    services::{BlobStoreService, RepositoryService},
};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_multiformat_test_app() -> axum::Router {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime connect error"));
    runtime.ensure_schema().await.expect("Schema init error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(S3BlobStore::from_env("multi-blobs"));
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

    build_app(AppState { runtime, blobstore })
}

#[tokio::test(flavor = "multi_thread")]
async fn test_npm_registry_lifecycle() {
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
                tarball: format!("http://localhost:8081/repository/npm-hosted/npm/{}/-/{}", package_name, tarball_filename),
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
        .body(axum::body::Body::empty())
        .unwrap();
    let get_doc_resp = app.clone().oneshot(get_doc_req).await.unwrap();
    assert_eq!(get_doc_resp.status(), StatusCode::OK);
    let doc_bytes = axum::body::to_bytes(get_doc_resp.into_body(), 1024 * 1024).await.unwrap();
    let fetched_doc: NpmPackageDocument = serde_json::from_slice(&doc_bytes).unwrap();
    assert_eq!(fetched_doc.name, package_name);
    assert_eq!(fetched_doc.dist_tags.get("latest").unwrap(), "1.0.0");

    // 3. Download tarball: GET /repository/npm-hosted/npm/:package_name/-/:tarball
    let get_tgz_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/npm-hosted/npm/{}/-/{}", package_name, tarball_filename))
        .body(axum::body::Body::empty())
        .unwrap();
    let get_tgz_resp = app.clone().oneshot(get_tgz_req).await.unwrap();
    assert_eq!(get_tgz_resp.status(), StatusCode::OK);
    let tgz_bytes = axum::body::to_bytes(get_tgz_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(tgz_bytes.as_ref(), fake_tgz_data);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_pypi_registry_lifecycle() {
    let app = setup_multiformat_test_app().await;

    let proj_name = format!("flask-util-{}", uuid::Uuid::new_v4().simple());
    let filename = format!("{}-2.0.0-py3-none-any.whl", proj_name);
    let whl_content = b"fake-python-wheel-zip-content";

    // Multipart upload payload
    let boundary = "------------------------Boundary123456789";
    let body_str = format!(
        "--{0}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\n{1}\r\n\
         --{0}\r\nContent-Disposition: form-data; name=\"version\"\r\n\r\n2.0.0\r\n\
         --{0}\r\nContent-Disposition: form-data; name=\"content\"; filename=\"{2}\"\r\nContent-Type: application/x-wheel+zip\r\n\r\n{3}\r\n\
         --{0}--\r\n",
        boundary, proj_name, filename, std::str::from_utf8(whl_content).unwrap()
    );

    // 1. Upload distribution: POST /repository/pypi-hosted/pypi/upload
    let post_req = Request::builder()
        .method(Method::POST)
        .uri("/repository/pypi-hosted/pypi/upload")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
        .body(axum::body::Body::from(body_str.into_bytes()))
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
    let root_html = String::from_utf8(axum::body::to_bytes(root_resp.into_body(), 1024 * 1024).await.unwrap().to_vec()).unwrap();
    assert!(root_html.contains(&format!("{}/", proj_name)));

    // 3. Simple Package Index: GET /repository/pypi-hosted/simple/:project/
    let pkg_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/pypi-hosted/simple/{}/", proj_name))
        .body(axum::body::Body::empty())
        .unwrap();
    let pkg_resp = app.clone().oneshot(pkg_req).await.unwrap();
    assert_eq!(pkg_resp.status(), StatusCode::OK);
    let pkg_html = String::from_utf8(axum::body::to_bytes(pkg_resp.into_body(), 1024 * 1024).await.unwrap().to_vec()).unwrap();
    assert!(pkg_html.contains(&filename));

    // 4. Download distribution: GET /repository/pypi-hosted/packages/:filename
    let dl_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/pypi-hosted/packages/{}", filename))
        .body(axum::body::Body::empty())
        .unwrap();
    let dl_resp = app.clone().oneshot(dl_req).await.unwrap();
    assert_eq!(dl_resp.status(), StatusCode::OK);
    let dl_bytes = axum::body::to_bytes(dl_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(dl_bytes.as_ref(), whl_content);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_gomod_registry_lifecycle() {
    let app = setup_multiformat_test_app().await;

    let module = format!("github.com/example/lib-{}", uuid::Uuid::new_v4().simple());
    let mod_content = b"module github.com/example/lib\n\ngo 1.22\n";
    let zip_content = b"fake-go-module-zip-binary";

    // 1. Upload go.mod and .zip
    let put_mod_req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/repository/gomod-hosted/gomod/{}/@v/v1.0.0.mod", module))
        .body(axum::body::Body::from(Bytes::from_static(mod_content)))
        .unwrap();
    let put_mod_resp = app.clone().oneshot(put_mod_req).await.unwrap();
    assert_eq!(put_mod_resp.status(), StatusCode::CREATED);

    let put_zip_req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/repository/gomod-hosted/gomod/{}/@v/v1.0.0.zip", module))
        .body(axum::body::Body::from(Bytes::from_static(zip_content)))
        .unwrap();
    let put_zip_resp = app.clone().oneshot(put_zip_req).await.unwrap();
    assert_eq!(put_zip_resp.status(), StatusCode::CREATED);

    // 2. Query @v/list
    let list_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/gomod-hosted/gomod/{}/@v/list", module))
        .body(axum::body::Body::empty())
        .unwrap();
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_txt = String::from_utf8(axum::body::to_bytes(list_resp.into_body(), 1024 * 1024).await.unwrap().to_vec()).unwrap();
    assert!(list_txt.contains("v1.0.0"));

    // 3. Query @v/v1.0.0.info
    let info_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/gomod-hosted/gomod/{}/@v/v1.0.0.info", module))
        .body(axum::body::Body::empty())
        .unwrap();
    let info_resp = app.clone().oneshot(info_req).await.unwrap();
    assert_eq!(info_resp.status(), StatusCode::OK);

    // 4. Download @v/v1.0.0.mod & zip
    let dl_mod_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/gomod-hosted/gomod/{}/@v/v1.0.0.mod", module))
        .body(axum::body::Body::empty())
        .unwrap();
    let dl_mod_resp = app.clone().oneshot(dl_mod_req).await.unwrap();
    assert_eq!(dl_mod_resp.status(), StatusCode::OK);
    let dl_mod_bytes = axum::body::to_bytes(dl_mod_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(dl_mod_bytes.as_ref(), mod_content);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_cargo_registry_lifecycle() {
    let app = setup_multiformat_test_app().await;

    // 1. Check config.json
    let cfg_req = Request::builder()
        .method(Method::GET)
        .uri("/repository/cargo-hosted/config.json")
        .body(axum::body::Body::empty())
        .unwrap();
    let cfg_resp = app.clone().oneshot(cfg_req).await.unwrap();
    assert_eq!(cfg_resp.status(), StatusCode::OK);

    // 2. Publish crate via PUT /repository/cargo-hosted/api/v1/crates/new
    let crate_name = format!("cr-{}", uuid::Uuid::new_v4().simple());
    let json_meta = serde_json::json!({
        "name": crate_name,
        "vers": "0.1.0",
        "deps": [],
        "features": {},
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
        .uri(format!("/repository/cargo-hosted/cargo/index/cr/{}", crate_name))
        .body(axum::body::Body::empty())
        .unwrap();
    let idx_resp = app.clone().oneshot(idx_req).await.unwrap();
    assert_eq!(idx_resp.status(), StatusCode::OK);

    // 4. Download crate
    let dl_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/repository/cargo-hosted/api/v1/crates/{}/0.1.0/download", crate_name))
        .body(axum::body::Body::empty())
        .unwrap();
    let dl_resp = app.clone().oneshot(dl_req).await.unwrap();
    assert_eq!(dl_resp.status(), StatusCode::OK);
    let dl_bytes = axum::body::to_bytes(dl_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(dl_bytes.as_ref(), crate_tarball);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_nuget_registry_lifecycle() {
    let app = setup_multiformat_test_app().await;

    // 1. Service Index: GET /repository/nuget-hosted/v3/index.json
    let index_req = Request::builder()
        .method(Method::GET)
        .uri("/repository/nuget-hosted/v3/index.json")
        .body(axum::body::Body::empty())
        .unwrap();
    let index_resp = app.clone().oneshot(index_req).await.unwrap();
    assert_eq!(index_resp.status(), StatusCode::OK);

    // 2. Push NuGet package: PUT /repository/nuget-hosted/v3/package
    let fake_nupkg = b"fake-nuget-package-zip-content";
    let push_req = Request::builder()
        .method(Method::PUT)
        .uri("/repository/nuget-hosted/v3/package")
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
    let dl_bytes = axum::body::to_bytes(dl_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(dl_bytes.as_ref(), fake_nupkg);
}
