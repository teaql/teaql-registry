use axum::{
    extract::Path,
    http::{header, Response, StatusCode},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "static/"]
struct EmbeddedAssets;

pub async fn handle_index() -> Response<axum::body::Body> {
    serve_embedded_file("index.html")
}

pub async fn handle_assets(Path(path): Path<String>) -> Response<axum::body::Body> {
    let full_path = format!("assets/{}", path);
    serve_embedded_file(&full_path)
}

fn serve_embedded_file(path: &str) -> Response<axum::body::Body> {
    match EmbeddedAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // SPA fallback to index.html
            if let Some(index) = EmbeddedAssets::get("index.html") {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(axum::body::Body::from(index.data.into_owned()))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(axum::body::Body::from("Frontend not found"))
                    .unwrap()
            }
        }
    }
}
