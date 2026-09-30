use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use teaql_core::Entity;
use teaql_registry_core::{PersonalAccessToken as StoredToken, ServiceRuntime, Q};
use uuid::Uuid;

use crate::services::SaveAuditedExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccessToken {
    pub id: String,
    pub tenant_id: u64,
    pub user_id: u64,
    pub username: String,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub description: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPrincipal {
    pub tenant_id: u64,
    pub user_id: u64,
    pub username: String,
    pub scopes: Vec<String>,
}

pub struct TokenService;

impl TokenService {
    pub const ALLOWED_SCOPES: [&'static str; 4] = [
        "repository:read",
        "repository:write",
        "registry:admin",
        "platform:admin",
    ];

    #[allow(clippy::too_many_arguments)]
    pub async fn create_token(
        context: &ServiceRuntime,
        tenant_id: u64,
        user_id: u64,
        username: &str,
        description: &str,
        scopes: Vec<String>,
        expires_in_days: Option<i64>,
    ) -> Result<(String, PersonalAccessToken)> {
        let raw_secret = format!("tql_pat_{}", Uuid::new_v4().simple());
        let token_hash = hash_token(&raw_secret);
        let now = Utc::now();
        let expires_at = expires_in_days
            .filter(|days| *days > 0)
            .map(|days| now + chrono::Duration::days(days));
        let token_id = Uuid::new_v4().to_string();

        let mut stored = Q::personal_access_tokens()
            .comment("what: create a personal access token credential")
            .purpose("why: authorize a native registry client without a password")
            .new_entity(context);
        stored.update_tenant_id(tenant_id);
        stored.update_security_user_id(user_id);
        stored.update_username(username);
        stored.update_token_id(token_id.as_str());
        stored.update_token_hash(token_hash.as_str());
        stored.update_description(description);
        stored.update_scopes(serde_json::to_string(&scopes)?);
        stored.update_created_at(teaql_core::time::Timestamp::from(now.timestamp_millis()));
        stored.update_expires_at_epoch_millis(
            expires_at
                .map(|value| value.timestamp_millis())
                .unwrap_or(0),
        );
        stored.update_revoked(false);
        stored.update_revoked_at_epoch_millis(0_i64);

        let stored = stored
            .audit_as("Creating personal access token")
            .save_with(context)
            .await
            .map_err(|error| anyhow!("failed to persist access token: {error}"))?;

        Ok((raw_secret, to_api_token(&stored, username.to_string())?))
    }

    pub async fn validate_token(
        context: &ServiceRuntime,
        raw_token: &str,
        required_scope: &str,
    ) -> Result<Option<TokenPrincipal>> {
        let token_hash = hash_token(raw_token);
        let Some(stored) = Q::personal_access_tokens_minimal()
            .select_self_fields()
            .with_token_hash_is(token_hash)
            .which_are_not_revoked()
            .limit(1)
            .comment("what: resolve a presented personal access token")
            .purpose("why: authenticate a registry protocol request")
            .execute_for_one(context)
            .await
            .map_err(|error| anyhow!("failed to validate access token: {error}"))?
        else {
            return Ok(None);
        };

        let expires_at = stored.expires_at_epoch_millis();
        if expires_at > 0 && Utc::now().timestamp_millis() > expires_at {
            return Ok(None);
        }
        let scopes = parse_scopes(&stored.scopes())?;
        if !required_scope.is_empty() && !scopes.iter().any(|scope| scope == required_scope) {
            return Ok(None);
        }
        Ok(Some(TokenPrincipal {
            tenant_id: stored.tenant_id(),
            user_id: stored.security_user_id(),
            username: stored.username(),
            scopes,
        }))
    }

    pub async fn revoke_token(
        context: &ServiceRuntime,
        tenant_id: u64,
        user_id: u64,
        token_id: &str,
    ) -> Result<bool> {
        let Some(mut stored) = Q::personal_access_tokens()
            .with_token_id_is(token_id)
            .limit(1)
            .comment("what: load a personal access token for revocation")
            .purpose("why: revoke one credential owned by the authenticated user")
            .execute_for_one(context)
            .await
            .map_err(|error| anyhow!("failed to load access token: {error}"))?
        else {
            return Ok(false);
        };
        if stored.tenant_id() != tenant_id || stored.security_user_id() != user_id {
            return Ok(false);
        }
        stored.update_revoked(true);
        stored.update_revoked_at_epoch_millis(Utc::now().timestamp_millis());
        stored
            .audit_as("Revoking personal access token")
            .save_with(context)
            .await
            .map_err(|error| anyhow!("failed to revoke access token: {error}"))?;
        Ok(true)
    }

    pub async fn list_user_tokens(
        context: &ServiceRuntime,
        tenant_id: u64,
        user_id: u64,
        username: &str,
    ) -> Result<Vec<PersonalAccessToken>> {
        let rows = Q::personal_access_tokens_minimal()
            .select_self_fields()
            .with_tenant_matching(Q::tenants_minimal().with_id_is(tenant_id))
            .with_security_user_matching(Q::security_users_minimal().with_id_is(user_id))
            .order_by_id_desc()
            .limit(100)
            .comment("what: list personal access tokens owned by one user")
            .purpose("why: administer registry credentials without exposing their secrets")
            .execute_for_list(context)
            .await
            .map_err(|error| anyhow!("failed to list access tokens: {error}"))?;
        rows.iter()
            .map(|stored| to_api_token(stored, username.to_string()))
            .collect()
    }
}

fn hash_token(raw_token: &str) -> String {
    hex::encode(Sha256::digest(raw_token.as_bytes()))
}

fn parse_scopes(value: &str) -> Result<Vec<String>> {
    serde_json::from_str(value).map_err(|error| anyhow!("invalid persisted token scopes: {error}"))
}

fn to_api_token(stored: &StoredToken, username: String) -> Result<PersonalAccessToken> {
    let expires_at = match stored.expires_at_epoch_millis() {
        0 => None,
        millis => DateTime::from_timestamp_millis(millis),
    };
    Ok(PersonalAccessToken {
        id: stored.token_id(),
        tenant_id: stored.tenant_id(),
        user_id: stored.security_user_id(),
        username,
        token_hash: stored.token_hash(),
        description: stored.description(),
        scopes: parse_scopes(&stored.scopes())?,
        created_at: stored.created_at().to_datetime(),
        expires_at,
    })
}
