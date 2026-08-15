use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

use crate::api::AppState;
use crate::webhook::{WebhookEventPayload, WebhookService};

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub target_url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

pub async fn handle_list_webhooks(State(_state): State<AppState>) -> Response {
    let list = WebhookService::list();
    Json(list).into_response()
}

pub async fn handle_create_webhook(
    State(_state): State<AppState>,
    Json(req): Json<CreateWebhookRequest>,
) -> Response {
    let sub = WebhookService::register(&req.target_url, req.events, req.secret);
    (StatusCode::CREATED, Json(sub)).into_response()
}

pub async fn handle_delete_webhook(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    if WebhookService::unregister(&id) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (StatusCode::NOT_FOUND, "Webhook not found").into_response()
    }
}

pub async fn handle_test_webhook(
    State(_state): State<AppState>,
    Json(payload): Json<WebhookEventPayload>,
) -> Response {
    WebhookService::dispatch(payload).await;
    StatusCode::ACCEPTED.into_response()
}
