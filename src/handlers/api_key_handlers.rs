use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    Extension, Json,
};
use validator::Validate;

use crate::{
    dto::{
        api_key_dto::{ApiKeyResponse, ApiKeysResponse, CreateApiKeyRequest, CreateApiKeyResponse},
        common::DeletedResponse,
    },
    error::WithRequestId,
    middleware::RequestId,
    models::CreateApiKey,
    AppError, AppResult, AppState,
};

pub async fn create_api_key(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> AppResult<(StatusCode, HeaderMap, Json<CreateApiKeyResponse>)> {
    payload
        .validate()
        .map_err(|e| AppError::validation_error(format!("Validation failed: {}", e)))?;

    let created_api_key = state
        .api_key_service
        .create_api_key(CreateApiKey::from(payload))
        .await
        .with_req_id(&request_id)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        format!("/api-keys/{}", created_api_key.api_key.id)
            .parse()
            .map_err(|e| {
                AppError::internal_error(format!("Failed to create Location header: {}", e))
            })?,
    );

    Ok((
        StatusCode::CREATED,
        headers,
        Json(CreateApiKeyResponse::from(created_api_key)),
    ))
}

pub async fn get_api_keys(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<ApiKeysResponse>> {
    let api_keys = state
        .api_key_service
        .list_api_keys()
        .await
        .with_req_id(&request_id)?;
    Ok(Json(ApiKeysResponse::from(api_keys)))
}

pub async fn get_api_key_by_id(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(api_key_id): Path<i32>,
) -> AppResult<Json<ApiKeyResponse>> {
    let api_key = state
        .api_key_service
        .find_by_id(api_key_id)
        .await
        .with_req_id(&request_id)?;
    Ok(Json(ApiKeyResponse::from(api_key)))
}

pub async fn delete_api_key(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(api_key_id): Path<i32>,
) -> AppResult<Json<DeletedResponse<ApiKeyResponse>>> {
    let deleted_api_key = state
        .api_key_service
        .delete_api_key(api_key_id)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(DeletedResponse {
        deleted: true,
        data: ApiKeyResponse::from(deleted_api_key),
    }))
}

pub async fn rotate_api_key(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(api_key_id): Path<i32>,
) -> AppResult<Json<CreateApiKeyResponse>> {
    let rotated_key = state
        .api_key_service
        .rotate_api_key(api_key_id)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(CreateApiKeyResponse::from(rotated_key)))
}
