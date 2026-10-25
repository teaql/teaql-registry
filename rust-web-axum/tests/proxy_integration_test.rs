//! Real PostgreSQL, HTTP upstream and public package routes. No external network.
mod common;

use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
    Router,
};
use sha1::Digest;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use teaql_registry::{
    api::{build_app, AppState},
    blobstore::{BlobStore, MemoryBlobStore},
    engine::RepositoryDispatcher,
    services::{BlobStoreService, RepositoryService, TenantService},
};
use teaql_registry_core::service_runtime;

#[derive(Clone, Default)]
struct Upstream {
    requests: Arc<Mutex<Vec<String>>>,
    assets: Arc<HashMap<String, (Vec<u8>, &'static str)>>,
}

async fn upstream(State(state): State<Upstream>, uri: Uri) -> axum::response::Response {
    let path = uri.path().replace("%2f", "/").replace("%2F", "/");
    state.requests.lock().unwrap().push(path.clone());
    if path == "/failure" {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    match state.assets.get(&path) {
        Some((body, content_type)) => {
            ([("content-type", *content_type)], body.clone()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn archive(name: &str, dependencies: serde_json::Value) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    let package = serde_json::to_vec(
        &serde_json::json!({"name": name, "version": "1.0.0", "dependencies": dependencies}),
    )
    .unwrap();
    let mut header = tar::Header::new_gnu();
    header.set_size(package.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, "package/package.json", package.as_slice())
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap()
}

#[test]
fn proxy_native_routes_cache_and_read_only_policy() {
    common::run_with_large_stack(run);
}

async fn run() {
    let runtime = Arc::new(service_runtime(common::runtime_config()).await.unwrap());
    runtime.ensure_schema().await.unwrap();
    if TenantService::find_tenant_by_code(&runtime, "default")
        .await
        .unwrap()
        .is_none()
    {
        TenantService::create_tenant_with_platform(&runtime, 1, "Default", "default", "Proxy test")
            .await
            .unwrap();
    }
    let store: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("proxy-test"));
    store.init().await.unwrap();
    let bs = BlobStoreService::create(
        &runtime,
        &format!("proxy-{}", uuid::Uuid::new_v4()),
        "/tmp/proxy-test",
        true,
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream_url = format!("http://{}", listener.local_addr().unwrap());
    let mut assets = HashMap::new();
    for (name, dependencies) in [
        (
            "proxy-fixture",
            serde_json::json!({"@proxy/child": "1.0.0"}),
        ),
        ("@proxy/child", serde_json::json!({})),
    ] {
        let basename = name.rsplit('/').next().unwrap();
        let tarball_path = format!("/{name}/-/{basename}-1.0.0.tgz");
        let metadata = serde_json::json!({"name": name, "dist-tags": {"latest": "1.0.0"}, "versions": {"1.0.0": {"name": name, "version": "1.0.0", "dependencies": dependencies, "engines": {"node": ">=14"}, "dist": {"tarball": format!("{upstream_url}{tarball_path}")}}}});
        assets.insert(
            format!("/{name}"),
            (serde_json::to_vec(&metadata).unwrap(), "application/json"),
        );
        assets.insert(
            tarball_path,
            (archive(name, dependencies), "application/gzip"),
        );
    }
    let maven_path = "/com/example/proxy/1.0/proxy-1.0.pom";
    let pom = b"<project><modelVersion>4.0.0</modelVersion><groupId>com.example</groupId><artifactId>proxy</artifactId><version>1.0</version><packaging>pom</packaging></project>";
    assets.insert(maven_path.into(), (pom.to_vec(), "application/xml"));
    assets.insert(
        format!("{maven_path}.sha1"),
        (
            hex::encode(sha1::Sha1::digest(pom)).into_bytes(),
            "text/plain",
        ),
    );
    let fixture = Upstream {
        assets: Arc::new(assets),
        ..Default::default()
    };
    let observed = fixture.requests.clone();
    let upstream_task = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(upstream).with_state(fixture),
        )
        .await
        .unwrap();
    });
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let npm_name = format!("npm-proxy-{suffix}");
    let maven_name = format!("maven-proxy-{suffix}");
    let npm_repo = RepositoryService::create(
        &runtime,
        &npm_name,
        "npm-proxy",
        "PROXY",
        "NPM",
        "READ_ONLY",
        bs.id(),
        true,
        &upstream_url,
    )
    .await
    .unwrap();
    let maven_repo = RepositoryService::create(
        &runtime,
        &maven_name,
        "maven2-proxy",
        "PROXY",
        "MAVEN2",
        "READ_ONLY",
        bs.id(),
        true,
        &upstream_url,
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = build_app(AppState::new(runtime.clone(), store.clone()));
    let registry_task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::new();
    let npm_url = format!("{base}/repository/{npm_name}/npm");
    let mut cached = Vec::new();
    for name in ["proxy-fixture", "@proxy/child"] {
        let metadata_url = format!("{npm_url}/{}", name.replace('/', "%2F"));
        let response = client.get(&metadata_url).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let metadata: serde_json::Value = response.json().await.unwrap();
        let version = &metadata["versions"]["1.0.0"];
        assert_eq!(version["engines"]["node"], ">=14");
        if name == "proxy-fixture" {
            assert_eq!(version["dependencies"]["@proxy/child"], "1.0.0");
        }
        let tarball_url = version["dist"]["tarball"].as_str().unwrap().to_string();
        assert!(tarball_url.starts_with(&npm_url));
        let tarball = client.get(&tarball_url).send().await.unwrap();
        assert_eq!(tarball.status(), StatusCode::OK);
        cached.push((tarball_url, tarball.bytes().await.unwrap()));
    }
    let maven_url = format!("{base}/repository/{maven_name}{maven_path}");
    let response = client.get(&maven_url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    cached.push((maven_url, response.bytes().await.unwrap()));
    let checksum_url = format!("{base}/repository/{maven_name}{maven_path}.sha1");
    let response = client.get(&checksum_url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    cached.push((checksum_url, response.bytes().await.unwrap()));
    let head = client.head(&cached.last().unwrap().0).send().await.unwrap();
    assert_eq!(head.status(), StatusCode::OK);
    assert!(RepositoryDispatcher::put(
        &runtime,
        &npm_repo,
        &store,
        "/upload",
        b"private",
        "text/plain"
    )
    .await
    .is_err());
    assert!(RepositoryDispatcher::put(
        &runtime,
        &maven_repo,
        &store,
        "/upload",
        b"private",
        "text/plain"
    )
    .await
    .is_err());
    let missing_url = format!("{base}/repository/{maven_name}/missing");
    for _ in 0..2 {
        assert_eq!(
            client.get(&missing_url).send().await.unwrap().status(),
            StatusCode::NOT_FOUND
        );
    }
    assert_eq!(
        observed
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.as_str() == "/missing")
            .count(),
        1
    );
    // An upstream failure must not become a false 404 or enter the negative cache.
    for _ in 0..2 {
        assert!(
            RepositoryDispatcher::get(&runtime, &maven_repo, &store, "/failure")
                .await
                .is_err()
        );
    }
    assert_eq!(
        observed
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.as_str() == "/failure")
            .count(),
        2
    );
    assert!(!observed.lock().unwrap().iter().any(|p| p == "/upload"));
    if std::env::var_os("TEAQL_TEST_PUBLIC_PROXY").is_some() {
        let public_npm = format!("npm-public-{suffix}");
        let public_maven = format!("maven-public-{suffix}");
        RepositoryService::create(
            &runtime,
            &public_npm,
            "npm-proxy",
            "PROXY",
            "NPM",
            "READ_ONLY",
            bs.id(),
            true,
            "https://registry.npmjs.org",
        )
        .await
        .unwrap();
        RepositoryService::create(
            &runtime,
            &public_maven,
            "maven2-proxy",
            "PROXY",
            "MAVEN2",
            "READ_ONLY",
            bs.id(),
            true,
            "https://repo.maven.apache.org/maven2",
        )
        .await
        .unwrap();
        let metadata: serde_json::Value = client
            .get(format!("{base}/repository/{public_npm}/npm/is-number"))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        let tarball = metadata["versions"]["7.0.0"]["dist"]["tarball"]
            .as_str()
            .unwrap();
        assert!(tarball.starts_with(&base));
        let bytes = client
            .get(tarball)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .bytes()
            .await
            .unwrap();
        assert!(bytes.starts_with(&[0x1f, 0x8b]));
        let pom = client.get(format!("{base}/repository/{public_maven}/org/apache/commons/commons-lang3/3.14.0/commons-lang3-3.14.0.pom")).send().await.unwrap().error_for_status().unwrap().text().await.unwrap();
        assert!(pom.contains("commons-lang3"));
        println!("Live npmjs tarball and Maven Central POM proxy downloads passed");
    }
    upstream_task.abort();
    let _ = upstream_task.await;
    for (url, expected) in cached {
        let response = client.get(url).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.bytes().await.unwrap(), expected);
    }
    let metadata: serde_json::Value = client
        .get(format!("{npm_url}/proxy-fixture"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        metadata["versions"]["1.0.0"]["dependencies"]["@proxy/child"],
        "1.0.0"
    );
    // Optional native-client verification is explicit and uses a fresh cache.
    if std::env::var_os("TEAQL_TEST_NATIVE_NPM").is_some() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().to_path_buf();
        std::fs::write(project.join("package.json"), "{\"private\":true}").unwrap();
        let registry = npm_url.clone();
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new("npm")
                .current_dir(&project)
                .args([
                    "install",
                    "--ignore-scripts",
                    "--no-audit",
                    "--no-fund",
                    "--registry",
                    &registry,
                    "--cache",
                ])
                .arg(project.join("cache"))
                .arg("proxy-fixture@1.0.0")
                .output()
                .unwrap()
        })
        .await
        .unwrap();
        assert!(
            output.status.success(),
            "npm install failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(dir
            .path()
            .join("node_modules/@proxy/child/package.json")
            .exists());
        println!("Native npm install with fresh cache and stopped upstream passed");
    }
    if std::env::var_os("TEAQL_TEST_NATIVE_MAVEN").is_some() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().to_path_buf();
        std::fs::write(project.join("pom.xml"), "<project><modelVersion>4.0.0</modelVersion><parent><groupId>com.example</groupId><artifactId>proxy</artifactId><version>1.0</version><relativePath/></parent><artifactId>consumer</artifactId><packaging>pom</packaging></project>").unwrap();
        std::fs::write(project.join("settings.xml"), format!("<settings><mirrors><mirror><id>proxy-only</id><mirrorOf>*</mirrorOf><url>{base}/repository/{maven_name}/</url></mirror></mirrors></settings>")).unwrap();
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new("mvn")
                .current_dir(&project)
                .args(["-B", "-s", "settings.xml"])
                .arg(format!(
                    "-Dmaven.repo.local={}",
                    project.join("m2").display()
                ))
                .arg("validate")
                .output()
                .unwrap()
        })
        .await
        .unwrap();
        assert!(
            output.status.success(),
            "Maven validate failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(dir
            .path()
            .join("m2/com/example/proxy/1.0/proxy-1.0.pom")
            .exists());
        println!("Native Maven parent resolution with fresh cache and stopped upstream passed");
    }
    registry_task.abort();
}
