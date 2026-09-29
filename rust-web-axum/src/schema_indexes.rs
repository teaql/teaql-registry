use anyhow::{bail, Result};
use deadpool_postgres::Pool;

/// Install Registry-owned constraints after the generated TeaQL schema exists.
pub async fn ensure_application_indexes(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;
    let duplicate_count: i64 = client
        .query_one(
            "SELECT count(*) FROM (\
               SELECT content_repository, path FROM asset_data \
               WHERE length(path) > 0 \
               GROUP BY content_repository, path HAVING count(*) > 1\
             ) duplicate_paths",
            &[],
        )
        .await?
        .get(0);
    if duplicate_count > 0 {
        bail!(
            "Registry schema has {duplicate_count} duplicate non-empty asset paths; \
             resolve them before enabling immutable Go module uploads"
        );
    }
    client
        .batch_execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS ux_personal_access_token_token_id \
             ON personal_access_token_data(token_id); \
             CREATE UNIQUE INDEX IF NOT EXISTS ux_personal_access_token_token_hash \
             ON personal_access_token_data(token_hash); \
             CREATE UNIQUE INDEX IF NOT EXISTS ux_asset_content_repository_path \
             ON asset_data(content_repository, path) WHERE length(path) > 0;",
        )
        .await?;
    Ok(())
}
