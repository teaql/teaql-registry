use anyhow::Result;
use tracing::info;

use crate::blobstore::BlobStore;
use crate::engine::hosted::HostedEngine;
use crate::services::{ComponentService, RepositoryService};
use teaql_registry_core::ServiceRuntime;

pub async fn seed_demo_artifacts(ctx: &ServiceRuntime, blobstore: &dyn BlobStore) -> Result<()> {
    // 1. Maven2 Demo Artifact
    if let Some(repo) = RepositoryService::find_by_name(ctx, "maven-releases").await? {
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example.teaql</groupId>
    <artifactId>teaql-sample-lib</artifactId>
    <version>1.0.0</version>
    <description>Sample Demo Java Library for TeaQL Registry</description>
</project>"#;
        let jar_content = b"PK\x03\x04\x14\x00\x00\x00\x08\x00Demo Java Bytecode in JAR Archive";

        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/com/example/teaql/teaql-sample-lib/1.0.0/teaql-sample-lib-1.0.0.pom",
            pom_content.as_bytes(),
            "application/xml",
        )
        .await?;

        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/com/example/teaql/teaql-sample-lib/1.0.0/teaql-sample-lib-1.0.0.jar",
            jar_content,
            "application/java-archive",
        )
        .await?;
        info!("Seeded Maven2 demo artifact: com.example.teaql:teaql-sample-lib:1.0.0");
    }

    // 2. NPM Demo Package
    if let Some(repo) = RepositoryService::find_by_name(ctx, "npm-hosted").await? {
        let npm_tarball = b"PK\x03\x04\x14\x00\x00\x00NPM sample tarball binary payload @teaql/sample-utils";
        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/npm/@teaql/sample-utils/-/sample-utils-1.0.0.tgz",
            npm_tarball,
            "application/gzip",
        )
        .await?;

        // Create NPM component entry
        if let Some(content_repo) = RepositoryService::get_content_repository(ctx, repo.id()).await? {
            let _ = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                "@teaql",
                "sample-utils",
                "1.0.0",
                "npm",
            )
            .await?;
        }
        info!("Seeded NPM demo package: @teaql/sample-utils@1.0.0");
    }

    // 3. PyPI Demo Package
    if let Some(repo) = RepositoryService::find_by_name(ctx, "pypi-hosted").await? {
        let pypi_wheel = b"PK\x03\x04\x14\x00\x00\x00Python Wheel Binary Payload teaql-client-1.0.0";
        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/packages/teaql_client-1.0.0-py3-none-any.whl",
            pypi_wheel,
            "application/x-wheel+zip",
        )
        .await?;

        if let Some(content_repo) = RepositoryService::get_content_repository(ctx, repo.id()).await? {
            let _ = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                "pypi",
                "teaql-client",
                "1.0.0",
                "pypi",
            )
            .await?;
        }
        info!("Seeded PyPI demo package: teaql-client==1.0.0");
    }

    // 4. Docker Demo Image
    if let Some(repo) = RepositoryService::find_by_name(ctx, "docker-hosted").await? {
        let docker_manifest = r#"{
   "schemaVersion": 2,
   "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
   "config": {
      "mediaType": "application/vnd.docker.container.image.v1+json",
      "size": 7023,
      "digest": "sha256:d826a7e0344d324b890887189196b27e69c10f607147b1981297e64a13e51f89"
   },
   "layers": [
      {
         "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
         "size": 32654,
         "digest": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
      }
   ]
}"#;
        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/v2/library/teaql-demo-service/manifests/1.0.0",
            docker_manifest.as_bytes(),
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .await?;

        if let Some(content_repo) = RepositoryService::get_content_repository(ctx, repo.id()).await? {
            let _ = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                "library",
                "teaql-demo-service",
                "1.0.0",
                "docker",
            )
            .await?;
        }
        info!("Seeded Docker demo image: teaql-demo-service:1.0.0");
    }

    // 5. Cargo Demo Crate
    if let Some(repo) = RepositoryService::find_by_name(ctx, "cargo-hosted").await? {
        let crate_bytes = b"PK\x03\x04\x14\x00\x00\x00Rust Crate Archive teaql-core-demo-0.1.0.crate";
        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/api/v1/crates/teaql-core-demo/0.1.0/download",
            crate_bytes,
            "application/gzip",
        )
        .await?;

        if let Some(content_repo) = RepositoryService::get_content_repository(ctx, repo.id()).await? {
            let _ = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                "cargo",
                "teaql-core-demo",
                "0.1.0",
                "cargo",
            )
            .await?;
        }
        info!("Seeded Cargo demo crate: teaql-core-demo@0.1.0");
    }

    // 6. Go Modules Demo Module
    if let Some(repo) = RepositoryService::find_by_name(ctx, "gomod-hosted").await? {
        let go_mod = "module github.com/teaql/sample-go-lib\n\ngo 1.22\n";
        let go_info = r#"{"Version":"v1.0.0","Time":"2026-08-15T00:00:00Z"}"#;
        let go_zip = b"PK\x03\x04\x14\x00\x00\x00Go Module Zip Archive github.com/teaql/sample-go-lib@v1.0.0";

        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/github.com/teaql/sample-go-lib/@v/v1.0.0.mod",
            go_mod.as_bytes(),
            "text/plain",
        )
        .await?;

        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/github.com/teaql/sample-go-lib/@v/v1.0.0.info",
            go_info.as_bytes(),
            "application/json",
        )
        .await?;

        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/github.com/teaql/sample-go-lib/@v/v1.0.0.zip",
            go_zip,
            "application/zip",
        )
        .await?;

        if let Some(content_repo) = RepositoryService::get_content_repository(ctx, repo.id()).await? {
            let _ = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                "github.com/teaql",
                "sample-go-lib",
                "1.0.0",
                "gomod",
            )
            .await?;
        }
        info!("Seeded Go Modules demo: github.com/teaql/sample-go-lib@v1.0.0");
    }

    // 7. NuGet Demo Package
    if let Some(repo) = RepositoryService::find_by_name(ctx, "nuget-hosted").await? {
        let nupkg_bytes = b"PK\x03\x04\x14\x00\x00\x00NuGet Package Archive TeaQL.SDK.DotNet.1.0.0.nupkg";
        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/v3/flatcontainer/teaql.sdk.dotnet/1.0.0/teaql.sdk.dotnet.1.0.0.nupkg",
            nupkg_bytes,
            "application/octet-stream",
        )
        .await?;

        if let Some(content_repo) = RepositoryService::get_content_repository(ctx, repo.id()).await? {
            let _ = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                "nuget",
                "TeaQL.SDK.DotNet",
                "1.0.0",
                "nuget",
            )
            .await?;
        }
        info!("Seeded NuGet demo package: TeaQL.SDK.DotNet@1.0.0");
    }

    // 8. Raw Generic Demo Asset
    if let Some(repo) = RepositoryService::find_by_name(ctx, "raw-hosted").await? {
        let raw_tarball = b"PK\x03\x04\x14\x00\x00\x00TeaQL CLI binary distribution tarball v1.0.0";
        HostedEngine::handle_put(
            ctx,
            &repo,
            blobstore,
            "/dist/v1.0.0/teaql-cli-linux-amd64.tar.gz",
            raw_tarball,
            "application/gzip",
        )
        .await?;

        if let Some(content_repo) = RepositoryService::get_content_repository(ctx, repo.id()).await? {
            let _ = ComponentService::find_or_create(
                ctx,
                content_repo.id(),
                "raw",
                "teaql-cli-linux-amd64",
                "1.0.0",
                "raw",
            )
            .await?;
        }
        info!("Seeded Raw demo asset: /dist/v1.0.0/teaql-cli-linux-amd64.tar.gz");
    }

    Ok(())
}
