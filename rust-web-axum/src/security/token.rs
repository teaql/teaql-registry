use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccessToken {
    pub id: String,
    pub username: String,
    pub token_hash: String,
    pub description: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

static TOKEN_STORE: LazyLock<Arc<RwLock<HashMap<String, PersonalAccessToken>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub struct TokenService;

impl TokenService {
    pub fn create_token(
        username: &str,
        description: &str,
        scopes: Vec<String>,
        expires_in_days: Option<i64>,
    ) -> (String, PersonalAccessToken) {
        let raw_secret = format!("tql_pat_{}", Uuid::new_v4().simple());
        let token_hash = hex::encode(Sha256::digest(raw_secret.as_bytes()));

        let now = Utc::now();
        let expires_at = expires_in_days.map(|days| now + chrono::Duration::days(days));

        let token_id = Uuid::new_v4().to_string();
        let pat = PersonalAccessToken {
            id: token_id,
            username: username.to_string(),
            token_hash: token_hash.clone(),
            description: description.to_string(),
            scopes,
            created_at: now,
            expires_at,
        };

        {
            let mut store = TOKEN_STORE.write().unwrap();
            store.insert(token_hash, pat.clone());
        }

        (raw_secret, pat)
    }

    pub fn validate_token(raw_token: &str, required_scope: &str) -> Option<String> {
        let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
        let store = TOKEN_STORE.read().unwrap();

        if let Some(pat) = store.get(&token_hash) {
            // Check expiration
            if let Some(expires_at) = pat.expires_at {
                if Utc::now() > expires_at {
                    return None;
                }
            }

            // Check scopes
            if pat.scopes.iter().any(|s| s == "admin" || s == required_scope) {
                return Some(pat.username.clone());
            }
        }

        None
    }

    pub fn revoke_token(token_id: &str) -> bool {
        let mut store = TOKEN_STORE.write().unwrap();
        if let Some(key) = store.iter().find_map(|(k, v)| if v.id == token_id { Some(k.clone()) } else { None }) {
            store.remove(&key);
            true
        } else {
            false
        }
    }

    pub fn list_user_tokens(username: &str) -> Vec<PersonalAccessToken> {
        let store = TOKEN_STORE.read().unwrap();
        store
            .values()
            .filter(|pat| pat.username == username)
            .cloned()
            .collect()
    }
}
