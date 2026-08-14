use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

use super::repository_content::AppState;
use crate::services::BlobStoreService;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobStoreItemXO {
    pub name: String,
    pub r#type: String,
    pub blob_count: i64,
    pub total_size_in_bytes: i64,
    pub available_space_in_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateS3BlobStoreRequest {
    pub name: String,
    pub bucket: Option<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileBlobStoreRequest {
    pub name: String,
    pub path: Option<String>,
}

pub async fn list_blobstores(State(state): State<AppState>) -> Response {
    match BlobStoreService::list(&state.runtime).await {
        Ok(stores) => {
            let items: Vec<BlobStoreItemXO> = stores
                .into_iter()
                .map(|s| BlobStoreItemXO {
                    name: s.name().to_string(),
                    r#type: "S3".to_string(),
                    blob_count: s.blob_count(),
                    total_size_in_bytes: s.total_size(),
                    available_space_in_bytes: 1099511627776, // 1 TB default S3 quota
                })
                .collect();
            Json(items).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_s3_blobstore(
    State(state): State<AppState>,
    Json(payload): Json<CreateS3BlobStoreRequest>,
) -> Response {
    let bucket = payload.bucket.unwrap_or_else(|| "teaql-blobs".to_string());
    let path = format!("s3://{}/{}", bucket, payload.prefix.unwrap_or(payload.name.clone()));
    match BlobStoreService::create(&state.runtime, &payload.name, &path, true).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn create_file_blobstore(
    State(state): State<AppState>,
    Json(payload): Json<CreateFileBlobStoreRequest>,
) -> Response {
    let path = payload
        .path
        .unwrap_or_else(|| format!("s3://teaql-blobs/{}", payload.name));
    match BlobStoreService::create(&state.runtime, &payload.name, &path, true).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}
