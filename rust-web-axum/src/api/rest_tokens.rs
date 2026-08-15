use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::security::{PersonalAccessToken, TokenService};

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub username: String,
    pub description: String,
    pub scopes: Vec<String>,
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
    pub token: String,
    pub pat: PersonalAccessToken,
}

pub async fn handle_list_tokens(State(_state): State<AppState>) -> Response {
    let tokens = TokenService::list_user_tokens("admin");
    Json(tokens).into_response()
}

pub async fn handle_create_token(
    State(_state): State<AppState>,
    Json(req): Json<CreateTokenRequest>,
) -> Response {
    let (secret, pat) = TokenService::create_token(
        &req.username,
        &req.description,
        req.scopes,
        req.expires_in_days,
    );

    (
        StatusCode::CREATED,
        Json(CreateTokenResponse {
            token: secret,
            pat,
        }),
    )
        .into_response()
}

pub async fn handle_revoke_token(
    State(_state): State<AppState>,
    Path(token_id): Path<String>,
) -> Response {
    if TokenService::revoke_token(&token_id) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (StatusCode::NOT_FOUND, "Token not found").into_response()
    }
}
