use axum::http::{header, Method, Request, StatusCode};
use nexus_repository_service_core::{service_runtime, ServiceRuntimeConfig};
use nexus_repository_service_core_workspace::{
    api::{build_app, AppState},
    blobstore::S3BlobStore,
    security::password::hash_password,
    services::{BlobStoreService, RepositoryService, SecurityService},
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_rest_test_app() -> axum::Router {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let runtime = Arc::new(service_runtime(config).await.expect("connect error"));
    runtime.ensure_schema().await.expect("schema error");

    let blobstore = Arc::new(S3BlobStore::from_env("rest-store"));
    blobstore.init().await.expect("init error");

    let bs_list = BlobStoreService::list(&runtime).await.unwrap();
    let bs = if let Some(b) = bs_list.into_iter().find(|b| b.name() == "default") {
        b
    } else {
        BlobStoreService::create(&runtime, "default", "/tmp/blobs/default", true)
            .await
            .unwrap()
    };

    let repos = RepositoryService::list(&runtime).await.unwrap();
    if !repos.iter().any(|r| r.name() == "maven-releases") {
        RepositoryService::create(
            &runtime,
            "maven-releases",
            "maven2-hosted",
            "HOSTED",
            "MAVEN2",
            "ALLOW_ONCE",
            bs.id(),
            true,
            "",
        )
        .await
        .unwrap();
    }

    let users = SecurityService::list_users(&runtime).await.unwrap();
    if !users.iter().any(|u| u.username() == "admin") {
        SecurityService::create_user(
            &runtime,
            "admin",
            "Administrator",
            "User",
            "admin@example.com",
            &hash_password("admin123"),
        )
        .await
        .unwrap();
    }

    build_app(AppState { runtime, blobstore })
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rest_status_endpoints() {
    let app = setup_rest_test_app().await;

    let req = Request::builder()
        .uri("/service/rest/v1/status")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req_writable = Request::builder()
        .uri("/service/rest/v1/status/writable")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp_writable = app.oneshot(req_writable).await.unwrap();
    assert_eq!(resp_writable.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rest_repositories_endpoints() {
    let app = setup_rest_test_app().await;

    // 1. List repositories
    let list_req = Request::builder()
        .uri("/service/rest/v1/repositories")
        .body(axum::body::Body::empty())
        .unwrap();
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);

    // 2. Get specific repository
    let get_req = Request::builder()
        .uri("/service/rest/v1/repositories/maven2/hosted/maven-releases")
        .body(axum::body::Body::empty())
        .unwrap();
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);

    // 3. Create raw repository
    let new_repo_name = format!("raw-rest-{}", uuid::Uuid::new_v4().simple());
    let create_body = json!({
        "name": new_repo_name,
        "online": true,
        "storage": {
            "blobStoreName": "default",
            "strictContentTypeValidation": false,
            "writePolicy": "allow_write"
        }
    });

    let create_req = Request::builder()
        .method(Method::POST)
        .uri("/service/rest/v1/repositories/raw/hosted")
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(create_body.to_string()))
        .unwrap();
    let create_resp = app.oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rest_blobstores_endpoints() {
    let app = setup_rest_test_app().await;

    // List blobstores
    let list_req = Request::builder()
        .uri("/service/rest/v1/blobstores")
        .body(axum::body::Body::empty())
        .unwrap();
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);

    // Create new file blobstore
    let bs_name = format!("bs-rest-{}", uuid::Uuid::new_v4().simple());
    let create_body = json!({
        "name": bs_name,
        "path": "/tmp/custom-blobs"
    });
    let create_req = Request::builder()
        .method(Method::POST)
        .uri("/service/rest/v1/blobstores/file")
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(create_body.to_string()))
        .unwrap();
    let create_resp = app.oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rest_security_endpoints() {
    let app = setup_rest_test_app().await;

    // 1. List users
    let req = Request::builder()
        .uri("/service/rest/v1/security/users")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 2. Create user
    let uname = format!("user-{}", uuid::Uuid::new_v4().simple());
    let user_body = json!({
        "userId": uname,
        "firstName": "John",
        "lastName": "Doe",
        "emailAddress": "john.doe@example.com",
        "password": "Password123!",
        "status": "active",
        "roles": ["nx-admin"]
    });
    let create_user_req = Request::builder()
        .method(Method::POST)
        .uri("/service/rest/v1/security/users")
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(user_body.to_string()))
        .unwrap();
    let create_user_resp = app.clone().oneshot(create_user_req).await.unwrap();
    assert_eq!(create_user_resp.status(), StatusCode::CREATED);

    // 3. List roles
    let roles_req = Request::builder()
        .uri("/service/rest/v1/security/roles")
        .body(axum::body::Body::empty())
        .unwrap();
    let roles_resp = app.clone().oneshot(roles_req).await.unwrap();
    assert_eq!(roles_resp.status(), StatusCode::OK);

    // 4. List privileges
    let privs_req = Request::builder()
        .uri("/service/rest/v1/security/privileges")
        .body(axum::body::Body::empty())
        .unwrap();
    let privs_resp = app.clone().oneshot(privs_req).await.unwrap();
    assert_eq!(privs_resp.status(), StatusCode::OK);

    // 5. Anonymous access configuration
    let anon_get = Request::builder()
        .uri("/service/rest/v1/security/anonymous")
        .body(axum::body::Body::empty())
        .unwrap();
    let anon_get_resp = app.clone().oneshot(anon_get).await.unwrap();
    assert_eq!(anon_get_resp.status(), StatusCode::OK);

    let anon_put = Request::builder()
        .method(Method::PUT)
        .uri("/service/rest/v1/security/anonymous")
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(
            json!({"enabled": true, "userId": "anonymous", "realmName": "NexusAuthorizingRealm"}).to_string(),
        ))
        .unwrap();
    let anon_put_resp = app.oneshot(anon_put).await.unwrap();
    assert_eq!(anon_put_resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rest_components_and_assets() {
    let app = setup_rest_test_app().await;

    // Components for maven-releases
    let comp_req = Request::builder()
        .uri("/service/rest/v1/components?repository=maven-releases")
        .body(axum::body::Body::empty())
        .unwrap();
    let comp_resp = app.clone().oneshot(comp_req).await.unwrap();
    assert_eq!(comp_resp.status(), StatusCode::OK);

    // Assets for maven-releases
    let asset_req = Request::builder()
        .uri("/service/rest/v1/assets?repository=maven-releases")
        .body(axum::body::Body::empty())
        .unwrap();
    let asset_resp = app.oneshot(asset_req).await.unwrap();
    assert_eq!(asset_resp.status(), StatusCode::OK);
}
