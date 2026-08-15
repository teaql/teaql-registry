use std::sync::Arc;
use teaql_registry::blobstore::{BlobStore, MemoryBlobStore};
use teaql_registry::context::RegistryContextExt;
use teaql_registry::engine::hosted::HostedEngine;
use teaql_registry::services::{ComponentService, RepositoryService};
use teaql_registry_core::{service_runtime, ServiceRuntimeConfig};

#[tokio::test(flavor = "multi_thread")]
async fn test_pure_memory_mode_single_latest_version_retention() {
    let config = ServiceRuntimeConfig {
        database_url: "postgresql://postgres:postgres@localhost:5432/nexus_db".to_string(),
        database_user: "postgres".to_string(),
        database_password: "postgres".to_string(),
    };
    let mut runtime = service_runtime(config).await.expect("Runtime error");
    runtime.ensure_schema().await.expect("Schema error");

    // 1. Initialize with MemoryBlobStore and enable memory_mode
    let blobstore: Arc<dyn BlobStore> = Arc::new(MemoryBlobStore::new("mem-store-test"));
    blobstore.init().await.expect("Init error");
    runtime.init_registry_context(blobstore.clone());
    runtime.set_memory_mode(true);
    assert!(runtime.is_memory_mode());

    let repo_name = format!("mem-maven-{}", uuid::Uuid::new_v4().simple());
    let repo = RepositoryService::create(
        &runtime,
        &repo_name,
        "maven2-hosted",
        "HOSTED",
        "MAVEN2",
        "ALLOW_WRITE",
        1,
        true,
        "",
    )
    .await
    .expect("Create repo failed");

    let content_repo = RepositoryService::ensure_content_repository(&runtime, repo.id(), "maven2")
        .await
        .expect("Ensure content repo failed");

    // 2. Publish version 1.0.0
    let pom_v1 = b"<project><groupId>com.teaql.ai</groupId><artifactId>model-runner</artifactId><version>1.0.0</version></project>";
    let jar_v1 = b"JAR_V1_BYTECODE_PAYLOAD";

    HostedEngine::handle_put(
        &runtime,
        &repo,
        blobstore.as_ref(),
        "/com/teaql/ai/model-runner/1.0.0/model-runner-1.0.0.pom",
        pom_v1,
        "application/xml",
    )
    .await
    .expect("Put v1 pom failed");

    HostedEngine::handle_put(
        &runtime,
        &repo,
        blobstore.as_ref(),
        "/com/teaql/ai/model-runner/1.0.0/model-runner-1.0.0.jar",
        jar_v1,
        "application/java-archive",
    )
    .await
    .expect("Put v1 jar failed");

    let comps_v1 = ComponentService::list_by_repository(&runtime, content_repo.id()).await.unwrap();
    assert_eq!(comps_v1.len(), 1);
    assert_eq!(comps_v1[0].version_name(), "1.0.0");

    // 3. Publish version 2.0.0 (in memory mode, version 1.0.0 MUST be evicted!)
    let pom_v2 = b"<project><groupId>com.teaql.ai</groupId><artifactId>model-runner</artifactId><version>2.0.0</version></project>";
    let jar_v2 = b"JAR_V2_BYTECODE_PAYLOAD_NEW";

    HostedEngine::handle_put(
        &runtime,
        &repo,
        blobstore.as_ref(),
        "/com/teaql/ai/model-runner/2.0.0/model-runner-2.0.0.pom",
        pom_v2,
        "application/xml",
    )
    .await
    .expect("Put v2 pom failed");

    HostedEngine::handle_put(
        &runtime,
        &repo,
        blobstore.as_ref(),
        "/com/teaql/ai/model-runner/2.0.0/model-runner-2.0.0.jar",
        jar_v2,
        "application/java-archive",
    )
    .await
    .expect("Put v2 jar failed");

    let comps_v2 = ComponentService::list_by_repository(&runtime, content_repo.id()).await.unwrap();
    // Strictly ONLY 1 version (the latest 2.0.0) must be present in memory mode!
    assert_eq!(comps_v2.len(), 1, "Only single latest version must be retained in memory mode");
    assert_eq!(comps_v2[0].version_name(), "2.0.0");

    // 4. Publish version 3.0.0 (version 2.0.0 must be evicted)
    let jar_v3 = b"JAR_V3_BYTECODE_PAYLOAD_LATEST";

    HostedEngine::handle_put(
        &runtime,
        &repo,
        blobstore.as_ref(),
        "/com/teaql/ai/model-runner/3.0.0/model-runner-3.0.0.jar",
        jar_v3,
        "application/java-archive",
    )
    .await
    .expect("Put v3 jar failed");

    let comps_v3 = ComponentService::list_by_repository(&runtime, content_repo.id()).await.unwrap();
    assert_eq!(comps_v3.len(), 1);
    assert_eq!(comps_v3[0].version_name(), "3.0.0");
}
