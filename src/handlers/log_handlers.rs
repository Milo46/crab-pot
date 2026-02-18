use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    dto::{
        common::DeletedResponse, CreateLogRequest, CursorLogsResponse, LogEvent, LogResponse,
        LogsResponse, QueryLogsRequest,
    },
    error::WithRequestId,
    middleware::RequestId,
    models::query_params::LogQueryParams,
    AppError, AppResult, AppState, SchemaNameVersion,
};

pub async fn create_log(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateLogRequest>,
) -> AppResult<(StatusCode, HeaderMap, Json<LogResponse>)> {
    let validated_payload = payload.validate_and_transform().with_req_id(&request_id)?;

    let log = state
        .log_service
        .create_log(validated_payload.schema_id, validated_payload.log_data)
        .await
        .with_req_id(&request_id)?;

    let _ = state
        .log_broadcast
        .send(LogEvent::created_from(log.clone()));

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        format!("/logs/{}", log.id).parse().map_err(|e| {
            AppError::internal_error(format!("Failed to create Location header: {}", e))
        })?,
    );

    Ok((StatusCode::CREATED, headers, Json(LogResponse::from(log))))
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

pub async fn delete_log(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<DeletedResponse<LogResponse>>> {
    let deleted_log = state
        .log_service
        .delete_log(id)
        .await
        .with_req_id(&request_id)?;

    let _ = state
        .log_broadcast
        .send(LogEvent::deleted_from(deleted_log.clone()));

    Ok(Json(DeletedResponse {
        deleted: true,
        data: LogResponse::from(deleted_log),
    }))
}

async fn get_logs_internal(
    state: AppState,
    schema_id: Uuid,
    params: QueryLogsRequest,
    request_id: RequestId,
) -> AppResult<Json<LogsResponse>> {
    let filters = LogQueryParams {
        date_begin: params.date_begin,
        date_end: params.date_end,
        json_filters: params.filters,
    };

    let (logs, cursor_metadata) = state
        .log_service
        .get_cursor_logs(schema_id, params.cursor, params.limit, filters)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(LogsResponse::Cursor(CursorLogsResponse::new(
        schema_id,
        logs,
        cursor_metadata,
    ))))
}

pub async fn get_logs(
    State(state): State<AppState>,
    Path(schema_id): Path<Uuid>,
    Query(params): Query<QueryLogsRequest>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<LogsResponse>> {
    get_logs_internal(state, schema_id, params, request_id).await
}

pub async fn get_logs_query(
    State(state): State<AppState>,
    Path(schema_id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<QueryLogsRequest>,
) -> AppResult<Json<LogsResponse>> {
    get_logs_internal(state, schema_id, payload, request_id).await
}

async fn get_logs_with_schema_resolve_internal(
    state: AppState,
    schema_ref: SchemaNameVersion,
    params: QueryLogsRequest,
    request_id: RequestId,
) -> AppResult<Json<LogsResponse>> {
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
) -> AppResult<Json<LogsResponse>> {
    let schema_ref = SchemaNameVersion::with_version(schema_name, schema_version);
    get_logs_with_schema_resolve_internal(state, schema_ref, params, request_id).await
}

pub async fn get_logs_by_schema_name_and_version_query(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<QueryLogsRequest>,
) -> AppResult<Json<LogsResponse>> {
    let schema_ref = SchemaNameVersion::with_version(schema_name, schema_version);
    get_logs_with_schema_resolve_internal(state, schema_ref, payload, request_id).await
}

pub async fn get_logs_by_schema_name_latest(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Query(params): Query<QueryLogsRequest>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<LogsResponse>> {
    let schema_ref = SchemaNameVersion::latest(schema_name);
    get_logs_with_schema_resolve_internal(state, schema_ref, params, request_id).await
}

pub async fn get_logs_by_schema_name_latest_query(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<QueryLogsRequest>,
) -> AppResult<Json<LogsResponse>> {
    let schema_ref = SchemaNameVersion::latest(schema_name);
    get_logs_with_schema_resolve_internal(state, schema_ref, payload, request_id).await
}

pub async fn get_initial_cursor(
    State(state): State<AppState>,
    Path(schema_id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<serde_json::Value>> {
    let cursor = state
        .log_service
        .get_initial_cursor(schema_id)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(serde_json::json!({
        "schema_id": schema_id,
        "initial_cursor": cursor
    })))
}
