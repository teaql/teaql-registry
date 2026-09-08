use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use uuid::Uuid;

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

static TOKEN_STORE: LazyLock<Arc<RwLock<HashMap<String, PersonalAccessToken>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub struct TokenService;

impl TokenService {
    pub const ALLOWED_SCOPES: [&'static str; 4] = [
        "repository:read",
        "repository:write",
        "registry:admin",
        "platform:admin",
    ];

    pub fn create_token(
        tenant_id: u64,
        user_id: u64,
        username: &str,
        description: &str,
        scopes: Vec<String>,
        expires_in_days: Option<i64>,
    ) -> (String, PersonalAccessToken) {
        let raw_secret = format!("tql_pat_{}", Uuid::new_v4().simple());
        let token_hash = hex::encode(Sha256::digest(raw_secret.as_bytes()));

        let now = Utc::now();
        let expires_at = expires_in_days
            .filter(|days| *days > 0)
            .map(|days| now + chrono::Duration::days(days));

        let token_id = Uuid::new_v4().to_string();
        let pat = PersonalAccessToken {
            id: token_id,
            tenant_id,
            user_id,
            username: username.to_string(),
            token_hash: token_hash.clone(),
            description: description.to_string(),
            scopes,
            created_at: now,
            expires_at,
        };

        {
            let mut store = TOKEN_STORE.write().expect("lock poisoned");
            store.insert(token_hash, pat.clone());
        }

        (raw_secret, pat)
    }

    pub fn validate_token(raw_token: &str, required_scope: &str) -> Option<TokenPrincipal> {
        let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
        let store = TOKEN_STORE.read().expect("lock poisoned");

        if let Some(pat) = store.get(&token_hash) {
            // Check expiration
            if let Some(expires_at) = pat.expires_at {
                if Utc::now() > expires_at {
                    return None;
                }
            }

            // Check scopes
            if required_scope.is_empty()
                || pat
                    .scopes
                    .iter()
                    .any(|scope| scope == "*" || scope == "admin" || scope == required_scope)
            {
                return Some(TokenPrincipal {
                    tenant_id: pat.tenant_id,
                    user_id: pat.user_id,
                    username: pat.username.clone(),
                    scopes: pat.scopes.clone(),
                });
            }
        }

        None
    }

    pub fn revoke_token(tenant_id: u64, user_id: u64, token_id: &str) -> bool {
        let mut store = TOKEN_STORE.write().expect("lock poisoned");
        if let Some(key) = store.iter().find_map(|(k, v)| {
            if v.id == token_id && v.tenant_id == tenant_id && v.user_id == user_id {
                Some(k.clone())
            } else {
                None
            }
        }) {
            store.remove(&key);
            true
        } else {
            false
        }
    }

    pub fn list_user_tokens(tenant_id: u64, user_id: u64) -> Vec<PersonalAccessToken> {
        let store = TOKEN_STORE.read().expect("lock poisoned");
        store
            .values()
            .filter(|pat| pat.tenant_id == tenant_id && pat.user_id == user_id)
            .cloned()
            .collect()
    }
}
