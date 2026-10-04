//! Publication preflight shared by immutable hosted package formats.
//!
//! The lock spans the existence check and every publication write. It is
//! process-wide (including fresh request contexts), NOT a distributed lock.
use anyhow::Result;
use axum::http::StatusCode;
use std::sync::OnceLock;
use teaql_registry_core::{ContentRepository, RepositoryConfiguration, ServiceRuntime};
use tokio::sync::{Mutex, MutexGuard};

use crate::services::{AssetService, RepositoryService};

#[derive(Debug, thiserror::Error)]
pub enum PublicationError {
    #[error("Repository is READ_ONLY; publication is disabled")]
    ReadOnly,
    #[error("Repository is ALLOW_ONCE; artifact already exists: {0}")]
    AlreadyExists(String),
}

pub fn publication_error_status(error: &anyhow::Error) -> StatusCode {
    match error.downcast_ref::<PublicationError>() {
        Some(PublicationError::ReadOnly) => StatusCode::FORBIDDEN,
        Some(PublicationError::AlreadyExists(_)) => StatusCode::CONFLICT,
        None => StatusCode::BAD_REQUEST,
    }
}

pub(crate) struct PublicationGuard {
    pub content_repository: ContentRepository,
    _lock: MutexGuard<'static, ()>,
}

impl PublicationGuard {
    pub async fn begin(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        format: &str,
        paths: &[String],
    ) -> Result<Self> {
        if repo.write_policy_is_read_only() {
            return Err(PublicationError::ReadOnly.into());
        }
        // Bounded stripes avoid a lock-map entry for every untrusted artifact.
        // Repository scope also serializes first content-repository creation.
        static LOCKS: OnceLock<[Mutex<()>; 256]> = OnceLock::new();
        let locks = LOCKS.get_or_init(|| std::array::from_fn(|_| Mutex::new(())));
        let lock = locks[(repo.id() % 256) as usize].lock().await;
        let content_repository =
            RepositoryService::ensure_content_repository(ctx, repo.id(), format).await?;
        if repo.write_policy_is_allow_once() {
            for path in paths {
                if AssetService::find_by_path(ctx, content_repository.id(), path)
                    .await?
                    .is_some()
                {
                    return Err(PublicationError::AlreadyExists(path.clone()).into());
                }
            }
        }
        Ok(Self {
            content_repository,
            _lock: lock,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_publication_statuses_survive_error_context() {
        let readonly = anyhow::Error::new(PublicationError::ReadOnly).context("upload failed");
        assert_eq!(publication_error_status(&readonly), StatusCode::FORBIDDEN);
        let duplicate = anyhow::Error::new(PublicationError::AlreadyExists("/a".into()));
        assert_eq!(publication_error_status(&duplicate), StatusCode::CONFLICT);
        assert_eq!(
            publication_error_status(&anyhow::anyhow!("invalid data")),
            StatusCode::BAD_REQUEST
        );
    }
}
