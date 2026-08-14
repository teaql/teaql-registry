use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn status_ok() -> Response {
    StatusCode::OK.into_response()
}

pub async fn status_writable() -> Response {
    StatusCode::OK.into_response()
}
