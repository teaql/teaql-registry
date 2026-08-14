use base64::Engine;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub roles: Vec<String>,
    pub privileges: HashSet<String>,
    pub is_anonymous: bool,
}

pub fn parse_basic_auth(auth_header: &str) -> Option<(String, String)> {
    if !auth_header.starts_with("Basic ") {
        return None;
    }
    let encoded = auth_header.trim_start_matches("Basic ").trim();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    let creds_str = String::from_utf8(decoded).ok()?;
    let (username, password) = creds_str.split_once(':')?;
    Some((username.to_string(), password.to_string()))
}
