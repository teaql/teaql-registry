use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::AppState;
use crate::security::{PersonalAccessToken, RequestContext, TokenService};

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub description: String,
    pub scopes: Vec<String>,
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateTokenResponse {
    pub token: String,
    pub pat: PersonalAccessToken,
}

pub async fn handle_list_tokens(
    State(_state): State<AppState>,
    Extension(request): Extension<Arc<RequestContext>>,
) -> Response {
    match TokenService::list_user_tokens(
        &request.runtime,
        request.tenant_id,
        request.user_id,
        &request.username,
    )
    .await
    {
        Ok(tokens) => Json(tokens).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

pub async fn handle_create_token(
    State(_state): State<AppState>,
    Extension(request): Extension<Arc<RequestContext>>,
    Json(req): Json<CreateTokenRequest>,
) -> Response {
    if req.scopes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "at least one token scope is required",
        )
            .into_response();
    }
    let mut scopes = req.scopes;
    scopes.sort();
    scopes.dedup();
    for scope in &scopes {
        if !TokenService::ALLOWED_SCOPES.contains(&scope.as_str()) {
            return (
                StatusCode::BAD_REQUEST,
                format!("unsupported token scope: {scope}"),
            )
                .into_response();
        }
        if !request.has_privilege(scope) {
            return (
                StatusCode::FORBIDDEN,
                format!("cannot grant token scope not held by caller: {scope}"),
            )
                .into_response();
        }
    }
    if req.expires_in_days.is_some_and(|days| days < 0) {
        return (StatusCode::BAD_REQUEST, "expiresInDays cannot be negative").into_response();
    }
    let result = TokenService::create_token(
        &request.runtime,
        request.tenant_id,
        request.user_id,
        &request.username,
        &req.description,
        scopes,
        req.expires_in_days,
    )
    .await;

    let (secret, pat) = match result {
        Ok(value) => value,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
        }
    };

    (
        StatusCode::CREATED,
        Json(CreateTokenResponse { token: secret, pat }),
    )
        .into_response()
}

pub async fn handle_revoke_token(
    State(_state): State<AppState>,
    Extension(request): Extension<Arc<RequestContext>>,
    Path(token_id): Path<String>,
) -> Response {
    match TokenService::revoke_token(
        &request.runtime,
        request.tenant_id,
        request.user_id,
        &token_id,
    )
    .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Token not found").into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}
