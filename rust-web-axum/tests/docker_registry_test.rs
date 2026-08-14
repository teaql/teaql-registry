use axum::http::{header, Method, Request, StatusCode};
use bytes::Bytes;
use nexus_repository_service_core::{service_runtime, ServiceRuntimeConfig};
use nexus_repository_service_core_workspace::{
    api::{build_app, AppState},
    blobstore::{BlobStore, S3BlobStore},
    format::docker::{
        compute_sha256_digest, DockerDescriptor, DockerManifestV2, DockerTagList,
        DOCKER_CONFIG_JSON_MEDIA_TYPE, DOCKER_LAYER_GZIP_MEDIA_TYPE, DOCKER_MANIFEST_V2_MEDIA_TYPE,
    },
    services::{BlobStoreService, RepositoryService},
};
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_docker_test_app() -> axum::Router {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("Runtime connect error"));
    runtime.ensure_schema().await.expect("Schema init error");

    let blobstore: Arc<dyn BlobStore> = Arc::new(S3BlobStore::from_env("docker-blobs"));
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
    if !repos.iter().any(|r| r.name() == "docker-hosted") {
        RepositoryService::create(
            &runtime,
            "docker-hosted",
            "docker-hosted",
            "HOSTED",
            "DOCKER",
            "ALLOW_WRITE",
            bs.id(),
            true,
            "",
        )
        .await
        .unwrap();
    }

    build_app(AppState { runtime, blobstore })
}

#[tokio::test(flavor = "multi_thread")]
async fn test_docker_v2_ping() {
    let app = setup_docker_test_app().await;

    let req = Request::builder()
        .uri("/v2/")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("docker-distribution-api-version")
            .unwrap(),
        "registry/2.0"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_docker_blob_and_manifest_lifecycle() {
    let app = setup_docker_test_app().await;
    let image_name = format!("testapp-{}", uuid::Uuid::new_v4().simple());

    // 1. Upload Layer Blob via Monolithic Upload (POST with ?digest=...)
    let layer_data = b"fake-tar-gzip-layer-binary-content-12345";
    let layer_digest = compute_sha256_digest(layer_data);

    let init_post_req = Request::builder()
        .method(Method::POST)
        .uri(format!("/v2/{}/blobs/uploads/?digest={}", image_name, layer_digest))
        .body(axum::body::Body::from(Bytes::from_static(layer_data)))
        .unwrap();
    let init_post_resp = app.clone().oneshot(init_post_req).await.unwrap();
    assert_eq!(init_post_resp.status(), StatusCode::CREATED);
    assert_eq!(
        init_post_resp.headers().get("docker-content-digest").unwrap(),
        layer_digest.as_str()
    );

    // 2. Upload Config Blob via Chunked Upload (POST -> PATCH -> PUT)
    let config_json = b"{\"architecture\":\"amd64\",\"os\":\"linux\",\"rootfs\":{\"type\":\"layers\"}}";
    let config_digest = compute_sha256_digest(config_json);

    // 2a. POST to init upload
    let post_req = Request::builder()
        .method(Method::POST)
        .uri(format!("/v2/{}/blobs/uploads/", image_name))
        .body(axum::body::Body::empty())
        .unwrap();
    let post_resp = app.clone().oneshot(post_req).await.unwrap();
    assert_eq!(post_resp.status(), StatusCode::ACCEPTED);
    let upload_location = post_resp
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap();

    // 2b. PATCH chunk
    let patch_req = Request::builder()
        .method(Method::PATCH)
        .uri(upload_location)
        .body(axum::body::Body::from(Bytes::from_static(config_json)))
        .unwrap();
    let patch_resp = app.clone().oneshot(patch_req).await.unwrap();
    assert_eq!(patch_resp.status(), StatusCode::ACCEPTED);

    // 2c. PUT to finish upload with digest
    let put_req = Request::builder()
        .method(Method::PUT)
        .uri(format!("{}?digest={}", upload_location, config_digest))
        .body(axum::body::Body::empty())
        .unwrap();
    let put_resp = app.clone().oneshot(put_req).await.unwrap();
    assert_eq!(put_resp.status(), StatusCode::CREATED);

    // 3. Verify HEAD & GET on Blobs
    let head_blob_req = Request::builder()
        .method(Method::HEAD)
        .uri(format!("/v2/{}/blobs/{}", image_name, layer_digest))
        .body(axum::body::Body::empty())
        .unwrap();
    let head_blob_resp = app.clone().oneshot(head_blob_req).await.unwrap();
    assert_eq!(head_blob_resp.status(), StatusCode::OK);

    let get_blob_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v2/{}/blobs/{}", image_name, layer_digest))
        .body(axum::body::Body::empty())
        .unwrap();
    let get_blob_resp = app.clone().oneshot(get_blob_req).await.unwrap();
    assert_eq!(get_blob_resp.status(), StatusCode::OK);
    let read_layer_bytes = axum::body::to_bytes(get_blob_resp.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(read_layer_bytes.as_ref(), layer_data);

    // 4. Create and Upload Docker Manifest v2
    let manifest = DockerManifestV2 {
        schema_version: 2,
        media_type: DOCKER_MANIFEST_V2_MEDIA_TYPE.to_string(),
        config: DockerDescriptor {
            media_type: DOCKER_CONFIG_JSON_MEDIA_TYPE.to_string(),
            size: config_json.len() as i64,
            digest: config_digest.clone(),
            urls: None,
        },
        layers: vec![DockerDescriptor {
            media_type: DOCKER_LAYER_GZIP_MEDIA_TYPE.to_string(),
            size: layer_data.len() as i64,
            digest: layer_digest.clone(),
            urls: None,
        }],
    };
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    let tag = "1.0.0";

    let put_manifest_req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/v2/{}/manifests/{}", image_name, tag))
        .header(header::CONTENT_TYPE, DOCKER_MANIFEST_V2_MEDIA_TYPE)
        .body(axum::body::Body::from(manifest_bytes.clone()))
        .unwrap();
    let put_manifest_resp = app.clone().oneshot(put_manifest_req).await.unwrap();
    assert_eq!(put_manifest_resp.status(), StatusCode::CREATED);
    let manifest_digest = put_manifest_resp
        .headers()
        .get("docker-content-digest")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // 5. GET Manifest by Tag and by Digest
    let get_manifest_tag_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v2/{}/manifests/{}", image_name, tag))
        .body(axum::body::Body::empty())
        .unwrap();
    let get_manifest_tag_resp = app.clone().oneshot(get_manifest_tag_req).await.unwrap();
    assert_eq!(get_manifest_tag_resp.status(), StatusCode::OK);

    let get_manifest_digest_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v2/{}/manifests/{}", image_name, manifest_digest))
        .body(axum::body::Body::empty())
        .unwrap();
    let get_manifest_digest_resp = app.clone().oneshot(get_manifest_digest_req).await.unwrap();
    assert_eq!(get_manifest_digest_resp.status(), StatusCode::OK);

    // 6. List Tags: GET /v2/<name>/tags/list
    let tags_req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v2/{}/tags/list", image_name))
        .body(axum::body::Body::empty())
        .unwrap();
    let tags_resp = app.clone().oneshot(tags_req).await.unwrap();
    assert_eq!(tags_resp.status(), StatusCode::OK);
    let tags_body = axum::body::to_bytes(tags_resp.into_body(), 1024 * 1024).await.unwrap();
    let tag_list: DockerTagList = serde_json::from_slice(&tags_body).unwrap();
    assert_eq!(tag_list.name, image_name);
    assert!(tag_list.tags.contains(&tag.to_string()));

    // 7. Verify Component & Asset in REST API
    let rest_comp_req = Request::builder()
        .uri("/service/rest/v1/components?repository=docker-hosted")
        .body(axum::body::Body::empty())
        .unwrap();
    let rest_comp_resp = app.oneshot(rest_comp_req).await.unwrap();
    assert_eq!(rest_comp_resp.status(), StatusCode::OK);
}
