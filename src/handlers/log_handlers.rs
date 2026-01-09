use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::{CreateLogRequest, LogEvent, LogResponse, PaginatedLogsResponse, QueryLogsRequest},
    error::WithRequestId,
    middleware::RequestId,
    AppResult, AppState, QueryParams, SchemaNameVersion,
};

pub async fn get_logs(
    State(state): State<AppState>,
    Path(schema_id): Path<Uuid>,
    Query(params): Query<QueryLogsRequest>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    get_logs_internal(state, schema_id, params, request_id).await
}

pub async fn get_logs_query(
    State(state): State<AppState>,
    Path(schema_id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<QueryLogsRequest>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    get_logs_internal(state, schema_id, payload, request_id).await
}

async fn get_logs_internal(
    state: AppState,
    schema_id: Uuid,
    params: impl Into<QueryParams>,
    request_id: RequestId,
) -> AppResult<Json<PaginatedLogsResponse>> {
    let query = params.into();
    let response = state
        .log_service
        .get_paginated_logs(schema_id, query)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(response))
}

async fn get_logs_by_schema_name_and_version_internal(
    state: AppState,
    schema_name: String,
    schema_version: String,
    params: impl Into<QueryParams>,
    request_id: RequestId,
) -> AppResult<Json<PaginatedLogsResponse>> {
    let schema_ref = SchemaNameVersion::with_version(schema_name, schema_version);
    let schema = state
        .schema_service
        .resolve_schema(&schema_ref)
        .await
        .with_req_id(&request_id)?;

    get_logs_internal(state, schema.id, params, request_id).await
}

pub async fn get_logs_by_schema_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Query(params): Query<QueryLogsRequest>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    get_logs_by_schema_name_and_version_internal(
        state,
        schema_name,
        schema_version,
        params,
        request_id,
    )
    .await
}

pub async fn get_logs_by_schema_name_and_version_query(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<QueryLogsRequest>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    get_logs_by_schema_name_and_version_internal(
        state,
        schema_name,
        schema_version,
        payload,
        request_id,
    )
    .await
}

pub async fn get_log_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<LogResponse>> {
    let log = state
        .log_service
        .get_log_by_id(id)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(LogResponse::from(log)))
}

pub async fn create_log(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateLogRequest>,
) -> AppResult<(StatusCode, Json<LogResponse>)> {
    // Validate at the boundary
    payload
        .validate()
        .map_err(|e| crate::AppError::validation_error(format!("Validation failed: {}", e)))?;

    let log = state
        .log_service
        .create_log(payload.schema_id, payload.log_data)
        .await
        .with_req_id(&request_id)?;

    let _ = state
        .log_broadcast
        .send(LogEvent::created_from(log.clone()));

    Ok((StatusCode::CREATED, Json(LogResponse::from(log))))
}

pub async fn delete_log(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<StatusCode> {
    let deleted_log = state
        .log_service
        .delete_log(id)
        .await
        .with_req_id(&request_id)?;

    let _ = state
        .log_broadcast
        .send(LogEvent::deleted_from(deleted_log));

    Ok(StatusCode::NO_CONTENT)
}
