use anyhow::{anyhow, Result};
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use super::group::GroupEngine;
use super::hosted::HostedEngine;
use super::proxy::ProxyEngine;
use crate::blobstore::{BlobStore, ByteStream};

pub struct RepositoryDispatcher;

impl RepositoryDispatcher {
    pub async fn get(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>> {
        if !repo.online() {
            return Err(anyhow!("Repository is offline: {}", repo.name()));
        }

        let recipe = repo.recipe_name();
        if recipe.ends_with("-hosted") || recipe == "hosted" {
            HostedEngine::handle_get(ctx, repo, blobstore, path).await
        } else if recipe.ends_with("-proxy") || recipe == "proxy" {
            ProxyEngine::handle_get(ctx, repo, blobstore, path).await
        } else if recipe.ends_with("-group") || recipe == "group" {
            GroupEngine::handle_get(ctx, repo, blobstore, path).await
        } else {
            HostedEngine::handle_get(ctx, repo, blobstore, path).await
        }
    }

    pub async fn get_stream(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
    ) -> Result<Option<(ByteStream, String, i64)>> {
        if !repo.online() {
            return Err(anyhow!("Repository is offline: {}", repo.name()));
        }
        let recipe = repo.recipe_name();
        if recipe.ends_with("-hosted") || recipe == "hosted" {
            HostedEngine::handle_get_stream(ctx, repo, blobstore, path).await
        } else {
            // Proxy and group engines still need their cache/routing logic;
            // adapt their bounded result without changing protocol semantics.
            Ok(Self::get(ctx, repo, blobstore, path)
                .await?
                .map(|(bytes, content_type)| {
                    let size = bytes.len() as i64;
                    let stream = Box::pin(futures_util::stream::once(async move { Ok(bytes) }));
                    (stream as ByteStream, content_type, size)
                }))
        }
    }

    pub async fn put(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<()> {
        if !repo.online() {
            return Err(anyhow!("Repository is offline: {}", repo.name()));
        }

        let recipe = repo.recipe_name();
        if recipe.ends_with("-hosted") || recipe == "hosted" {
            HostedEngine::handle_put(ctx, repo, blobstore, path, data, content_type).await
        } else {
            Err(anyhow!(
                "Repository {} with recipe {} does not support direct PUT",
                repo.name(),
                recipe
            ))
        }
    }

    pub async fn put_stream(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        path: &str,
        stream: ByteStream,
        content_type: &str,
    ) -> Result<i64> {
        if !repo.online() {
            return Err(anyhow!("Repository is offline: {}", repo.name()));
        }
        let recipe = repo.recipe_name();
        if recipe.ends_with("-hosted") || recipe == "hosted" {
            HostedEngine::handle_put_stream(ctx, repo, blobstore, path, stream, content_type).await
        } else {
            Err(anyhow!(
                "Repository {} with recipe {} does not support direct PUT",
                repo.name(),
                recipe
            ))
        }
    }
}
