use crate::services::SaveAuditedExt;
use anyhow::{anyhow, Result};
use std::sync::{Arc, LazyLock};
use teaql_core::{Entity, SmartList};
use teaql_registry_core::{ServiceLog, ServiceRuntime, Q};
use tokio::sync::Semaphore;

const MAX_PENDING_LOG_WRITES: usize = 256;
static LOG_WRITE_SLOTS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(MAX_PENDING_LOG_WRITES)));

pub struct ServiceLogService;

impl ServiceLogService {
    /// Write a service log entry to the database.
    /// This is the core method called by all handlers.
    /// Errors are intentionally swallowed at call sites to avoid
    /// blocking artifact operations due to logging failures.
    #[allow(clippy::too_many_arguments)]
    pub async fn write_log(
        ctx: &ServiceRuntime,
        tenant_id: u64,
        log_type: &str,
        operator_id: i64,
        operator_name: &str,
        client_ip: &str,
        action: &str,
        repo_name: &str,
        artifact_path: &str,
        format_name: &str,
        content_size: i64,
        status: &str,
        error_message: &str,
    ) -> Result<()> {
        let event_time = teaql_core::time::Timestamp::now();

        let mut entry = Q::service_logs()
            .comment("what: create a structured registry service log entry")
            .purpose("why: retain auditable package operation evidence")
            .new_entity(ctx);

        entry.update_tenant_id(tenant_id);
        entry.update_event_time(event_time);
        entry.update_log_type(log_type.to_string());
        entry.update_operator_id(operator_id);
        entry.update_operator_name(operator_name.to_string());
        entry.update_client_ip(client_ip.to_string());
        entry.update_action(action.to_string());
        entry.update_repository_name(repo_name.to_string());
        entry.update_artifact_path(artifact_path.to_string());
        entry.update_format_name(format_name.to_string());
        entry.update_content_size(content_size);
        entry.update_status(status.to_string());
        entry.update_error_message(error_message.to_string());

        entry
            .audit_as("Recording service log entry")
            .save_with(ctx)
            .await
            .map_err(|e| anyhow!("Failed to write service log: {}", e))?;

        Ok(())
    }

    /// Enqueue a bounded background write. Registry traffic never waits for the
    /// audit database, and overload drops logs explicitly instead of spawning
    /// an unbounded number of tasks.
    #[allow(clippy::too_many_arguments)]
    pub async fn log_event(
        ctx: &Arc<ServiceRuntime>,
        tenant_id: u64,
        log_type: &str,
        operator_id: i64,
        operator_name: &str,
        client_ip: &str,
        action: &str,
        repo_name: &str,
        artifact_path: &str,
        format_name: &str,
        content_size: i64,
        status: &str,
        error_message: &str,
    ) {
        let Ok(slot) = LOG_WRITE_SLOTS.clone().try_acquire_owned() else {
            tracing::warn!(
                max_pending = MAX_PENDING_LOG_WRITES,
                "dropping registry service log because the async writer is saturated"
            );
            return;
        };
        let ctx = ctx.clone();
        let values = [
            log_type,
            operator_name,
            client_ip,
            action,
            repo_name,
            artifact_path,
            format_name,
            status,
            error_message,
        ]
        .map(str::to_string);
        tokio::spawn(async move {
            let _slot = slot;
            if let Err(error) = Self::write_log(
                &ctx,
                tenant_id,
                &values[0],
                operator_id,
                &values[1],
                &values[2],
                &values[3],
                &values[4],
                &values[5],
                &values[6],
                content_size,
                &values[7],
                &values[8],
            )
            .await
            {
                tracing::warn!(%error, "failed to write registry service log");
            }
        });
    }

    /// Query service logs with optional filters and pagination.
    pub async fn query_logs(
        ctx: &ServiceRuntime,
        tenant_id: Option<u64>,
        log_type: Option<&str>,
        operator_name: Option<&str>,
        action: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Result<SmartList<ServiceLog>> {
        let mut query = Q::service_logs_minimal().select_self_fields();

        if let Some(tid) = tenant_id {
            query = query.filter_by_tenant(tid);
        }
        if let Some(lt) = log_type {
            query = query.with_log_type_is(lt);
        }
        if let Some(u) = operator_name {
            query = query.with_operator_name_is(u);
        }
        if let Some(a) = action {
            query = query.with_action_is(a);
        }

        let rows = query
            .order_by_id_desc()
            .offset(offset as u64, limit as u64)
            .comment("what: query registry service logs with bounded pagination")
            .purpose("why: render auditable package operation history")
            .execute_for_list(ctx)
            .await
            .map_err(|e| anyhow!("Failed to query service logs: {}", e))?;

        Ok(rows)
    }
}
