use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::{
    dto::{
        log_dto::QueryParams, CreateLogRequest, ErrorResponse, LogEvent, LogResponse,
        PaginatedLogsResponse, QueryLogsRequest,
    },
    services::schema_service::SchemaNameVersion,
    AppState,
};

pub async fn get_logs_by_name(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Query(params): Query<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    get_logs_internal(state, schema_name, None, params).await
}

pub async fn get_logs_by_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Query(params): Query<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    get_logs_internal(state, schema_name, Some(schema_version), params).await
}

async fn get_logs_internal(
    state: AppState,
    schema_name: String,
    schema_version: Option<String>,
    params: QueryLogsRequest,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name cannot be empty",
            )),
        ));
    }

    if let Some(ref version) = schema_version {
        if version.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "INVALID_INPUT",
                    "Schema version cannot be empty",
                )),
            ));
        }
    }

    let query_params: QueryParams = params.into();
    let schema_ref = SchemaNameVersion::new(schema_name.clone(), schema_version.clone());

    let schema = match state.schema_service.resolve_schema(&schema_ref).await {
        Ok(schema) => schema,
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            return Err((
                status_code,
                Json(ErrorResponse::new("SCHEMA_NOT_FOUND", e.to_string())),
            ));
        }
    };

    match state
        .log_service
        .get_paginated_logs(schema.id, query_params)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse::new("FETCH_FAILED", e.to_string())),
            ))
        }
    }
}

pub async fn query_logs_by_name(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Json(payload): Json<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    query_logs_internal(state, schema_name, None, payload).await
}

pub async fn query_logs_by_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Json(payload): Json<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    query_logs_internal(state, schema_name, Some(schema_version), payload).await
}

async fn query_logs_internal(
    state: AppState,
    schema_name: String,
    schema_version: Option<String>,
    payload: QueryLogsRequest,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name cannot be empty",
            )),
        ));
    }

    if let Some(ref version) = schema_version {
        if version.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "INVALID_INPUT",
                    "Schema version cannot be empty",
                )),
            ));
        }
    }

    let query_params: QueryParams = payload.into();
    let schema_ref = SchemaNameVersion::new(schema_name.clone(), schema_version.clone());

    let schema = match state.schema_service.resolve_schema(&schema_ref).await {
        Ok(schema) => schema,
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            return Err((
                status_code,
                Json(ErrorResponse::new("SCHEMA_NOT_FOUND", e.to_string())),
            ));
        }
    };

    match state
        .log_service
        .get_paginated_logs(schema.id, query_params)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse::new("FETCH_FAILED", e.to_string())),
            ))
        }
    }
}

pub async fn get_log_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<LogResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.log_service.get_log_by_id(id).await {
        Ok(Some(log)) => Ok(Json(LogResponse::from(log))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "NOT_FOUND",
                format!("Log with id '{}' not found", id),
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("FETCH_FAILED", e.to_string())),
        )),
    }
}

pub async fn create_log(
    State(state): State<AppState>,
    Json(payload): Json<CreateLogRequest>,
) -> Result<(StatusCode, Json<LogResponse>), (StatusCode, Json<ErrorResponse>)> {
    if payload.schema_id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
            )),
        ));
    }

    if !payload.log_data.is_object() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Log data must be a JSON object",
            )),
        ));
    }

    match state
        .log_service
        .create_log(payload.schema_id, payload.log_data)
        .await
    {
        Ok(log) => {
            let _ = state
                .log_broadcast
                .send(LogEvent::created_from(log.clone()));
            Ok((StatusCode::CREATED, Json(LogResponse::from(log))))
        }
        Err(e) => {
            let (status_code, error) = if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "NOT_FOUND")
            } else if e.to_string().contains("validation")
                || e.to_string().contains("Required field")
            {
                (StatusCode::BAD_REQUEST, "VALIDATION_FAILED")
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR")
            };

            Err((status_code, Json(ErrorResponse::new(error, e.to_string()))))
        }
    }
}

pub async fn delete_log(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let log = state.log_service.get_log_by_id(id).await;
    match state.log_service.delete_log(id).await {
        Ok(true) => {
            if let Ok(Some(log)) = log {
                let _ = state.log_broadcast.send(LogEvent::deleted_from(log));
            }
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "NOT_FOUND",
                format!("Log with id '{}' not found", id),
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("DELETION_FAILED", e.to_string())),
        )),
    }
}
