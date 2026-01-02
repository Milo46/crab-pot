use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};

use crate::{
    dto::{
        log_dto::QueryParams, CreateLogRequest, LogEvent, LogResponse, PaginatedLogsResponse,
        QueryLogsRequest,
    },
    error::WithRequestId,
    middleware::RequestId,
    services::schema_service::SchemaNameVersion,
    AppError, AppResult, AppState,
};

pub async fn get_logs_by_name(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Query(params): Query<QueryLogsRequest>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    get_logs_internal(state, schema_name, None, params, request_id).await
}

pub async fn get_logs_by_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Query(params): Query<QueryLogsRequest>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    get_logs_internal(state, schema_name, Some(schema_version), params, request_id).await
}

async fn get_logs_internal(
    state: AppState,
    schema_name: String,
    schema_version: Option<String>,
    params: QueryLogsRequest,
    request_id: RequestId,
) -> AppResult<Json<PaginatedLogsResponse>> {
    if schema_name.trim().is_empty() {
        return Err(AppError::bad_request("Schema name cannot be empty")).with_req_id(&request_id);
    }

    if let Some(ref version) = schema_version {
        if version.trim().is_empty() {
            return Err(AppError::bad_request("Schema version cannot be empty"))
                .with_req_id(&request_id);
        }
    }

    let query_params: QueryParams = params.into();
    let schema_ref = SchemaNameVersion::new(schema_name.clone(), schema_version.clone());

    let schema = state
        .schema_service
        .resolve_schema(&schema_ref)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    let response = state
        .log_service
        .get_paginated_logs(schema.id, query_params)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    Ok(Json(response))
}

pub async fn query_logs_by_name(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<QueryLogsRequest>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    query_logs_internal(state, schema_name, None, payload, request_id).await
}

pub async fn query_logs_by_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<QueryLogsRequest>,
) -> AppResult<Json<PaginatedLogsResponse>> {
    query_logs_internal(
        state,
        schema_name,
        Some(schema_version),
        payload,
        request_id,
    )
    .await
}

async fn query_logs_internal(
    state: AppState,
    schema_name: String,
    schema_version: Option<String>,
    payload: QueryLogsRequest,
    request_id: RequestId,
) -> AppResult<Json<PaginatedLogsResponse>> {
    if schema_name.trim().is_empty() {
        return Err(
            AppError::bad_request("Schema name cannot be empty").with_request_id(&request_id)
        );
    }

    if let Some(ref version) = schema_version {
        if version.trim().is_empty() {
            return Err(AppError::bad_request("Schema version cannot be empty")
                .with_request_id(&request_id));
        }
    }

    let query_params: QueryParams = payload.into();
    let schema_ref = SchemaNameVersion::new(schema_name.clone(), schema_version.clone());

    let schema = state
        .schema_service
        .resolve_schema(&schema_ref)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    let response = state
        .log_service
        .get_paginated_logs(schema.id, query_params)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    Ok(Json(response))
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
        .map_err(|e| e.with_request_id(&request_id))?;

    Ok(Json(LogResponse::from(log)))
}

pub async fn create_log(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateLogRequest>,
) -> AppResult<(StatusCode, Json<LogResponse>)> {
    if payload.schema_id.is_nil() {
        return Err(AppError::bad_request("Schema ID cannot be empty").with_request_id(&request_id));
    }

    if !payload.log_data.is_object() {
        return Err(
            AppError::bad_request("Log Data must be a JSON object").with_request_id(&request_id)
        );
    }

    let log = state
        .log_service
        .create_log(payload.schema_id, payload.log_data)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

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
    let log = state.log_service.get_log_by_id(id).await.ok();

    let deleted = state
        .log_service
        .delete_log(id)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    if deleted {
        if let Some(log) = log {
            let _ = state.log_broadcast.send(LogEvent::deleted_from(log));
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(
            AppError::not_found(format!("Log with id '{}' not found", id))
                .with_request_id(&request_id),
        )
    }
}
