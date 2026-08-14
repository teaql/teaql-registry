use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

use super::repository_content::AppState;
use crate::security::password::hash_password;
use crate::services::SecurityService;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserXO {
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
    pub status: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub user_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email_address: String,
    pub password: String,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleXO {
    pub id: String,
    pub name: String,
    pub description: String,
    pub read_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegeXO {
    pub id: String,
    pub name: String,
    pub description: String,
    pub r#type: String,
    pub permission: String,
    pub read_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnonymousConfigXO {
    pub enabled: bool,
    pub user_id: String,
    pub realm_name: String,
}

pub async fn list_users(State(state): State<AppState>) -> Response {
    match SecurityService::list_users(&state.runtime).await {
        Ok(users) => {
            let items: Vec<UserXO> = users
                .into_iter()
                .map(|u| UserXO {
                    user_id: u.username().to_string(),
                    first_name: u.first_name().to_string(),
                    last_name: u.last_name().to_string(),
                    email_address: u.email().to_string(),
                    status: "active".to_string(),
                    roles: vec!["nx-admin".to_string()],
                })
                .collect();
            Json(items).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Response {
    let password_hash = hash_password(&payload.password);
    match SecurityService::create_user(
        &state.runtime,
        &payload.user_id,
        payload.first_name.as_deref().unwrap_or(""),
        payload.last_name.as_deref().unwrap_or(""),
        &payload.email_address,
        &password_hash,
    )
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn list_roles(State(state): State<AppState>) -> Response {
    match SecurityService::list_roles(&state.runtime).await {
        Ok(roles) => {
            let items: Vec<RoleXO> = roles
                .into_iter()
                .map(|r| RoleXO {
                    id: r.role_id().to_string(),
                    name: r.name().to_string(),
                    description: r.description().to_string(),
                    read_only: r.read_only(),
                })
                .collect();
            Json(items).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn list_privileges(State(state): State<AppState>) -> Response {
    match SecurityService::list_privileges(&state.runtime).await {
        Ok(privs) => {
            let items: Vec<PrivilegeXO> = privs
                .into_iter()
                .map(|p| PrivilegeXO {
                    id: p.privilege_id().to_string(),
                    name: p.name().to_string(),
                    description: p.description().to_string(),
                    r#type: p.privilege_type().to_string(),
                    permission: p.permission_pattern().to_string(),
                    read_only: p.read_only(),
                })
                .collect();
            Json(items).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_anonymous_config() -> Response {
    Json(AnonymousConfigXO {
        enabled: true,
        user_id: "anonymous".to_string(),
        realm_name: "NexusAuthorizingRealm".to_string(),
    })
    .into_response()
}

pub async fn update_anonymous_config(
    Json(_config): Json<AnonymousConfigXO>,
) -> Response {
    StatusCode::NO_CONTENT.into_response()
}
