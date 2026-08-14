use anyhow::{anyhow, Result};
use bytes::Bytes;
use teaql_registry_core::{RepositoryConfiguration, ServiceRuntime};

use super::group::GroupEngine;
use super::hosted::HostedEngine;
use super::proxy::ProxyEngine;
use crate::blobstore::BlobStore;

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
}
