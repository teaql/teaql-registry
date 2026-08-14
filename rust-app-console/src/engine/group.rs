use anyhow::Result;
use bytes::Bytes;
use nexus_repository_service_core::{RepositoryConfiguration, ServiceRuntime};

use super::hosted::HostedEngine;
use super::proxy::ProxyEngine;
use crate::blobstore::S3BlobStore;
use crate::services::RepositoryService;

pub struct GroupEngine;

impl GroupEngine {
    pub async fn handle_get(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &S3BlobStore,
        path: &str,
    ) -> Result<Option<(Bytes, String)>> {
        // Find member repos by recipe conventions or configuration
        let all_repos = RepositoryService::list(ctx).await?;

        // If repo is maven-public, default members are maven-releases, maven-snapshots, maven-central
        let members: Vec<RepositoryConfiguration> = if repo.name() == "maven-public" {
            let names = ["maven-releases", "maven-snapshots", "maven-central"];
            all_repos
                .into_iter()
                .filter(|r| names.contains(&r.name().as_str()))
                .collect()
        } else {
            all_repos
                .into_iter()
                .filter(|r| {
                    r.name() != repo.name()
                        && r.online()
                        && r.repository_format_id() == repo.repository_format_id()
                })
                .collect()
        };

        for member in members {
            if member.recipe_name().contains("hosted") {
                if let Ok(Some(res)) = HostedEngine::handle_get(ctx, &member, blobstore, path).await {
                    return Ok(Some(res));
                }
            } else if member.recipe_name().contains("proxy") {
                if let Ok(Some(res)) = ProxyEngine::handle_get(ctx, &member, blobstore, path).await {
                    return Ok(Some(res));
                }
            }
        }

        Ok(None)
    }
}
