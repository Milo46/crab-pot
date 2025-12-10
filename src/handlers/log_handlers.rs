use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    dto::{
        CreateLogRequest, ErrorResponse, LogEvent, LogResponse, PaginatedLogsResponse,
        PaginationMetadata, QueryLogsRequest,
    },
    AppState,
};

pub async fn get_logs_default(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    get_logs(
        State(state),
        Path((schema_name, "1.0.0".to_string())),
        Query(params),
    )
    .await
}

pub async fn get_logs(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() || schema_version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name or version cannot be empty",
            )),
        ));
    }

    fn extract_i32(params: &HashMap<String, String>, key: &str, default: i32) -> i32 {
        params
            .get(key)
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(default)
    }

    let page: i32 = extract_i32(&params, "page", 1);
    let page_limit: i32 = extract_i32(&params, "limit", 10);

    let filters: Option<Value> = if let Some(filters_str) = params.get("filters") {
        match serde_json::from_str::<Value>(&filters_str) {
            Ok(Value::Object(map)) => Some(Value::Object(map)),
            Ok(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new(
                        "INVALID_FILTERS",
                        "Filters must be a JSON object",
                    )),
                ));
            }
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new(
                        "INVALID_FILTERS",
                        "Filters JSON is invalid",
                    )),
                ));
            }
        }
    } else {
        None
    };

    match state
        .log_service
        .get_logs_by_schema_name_and_id(
            &schema_name,
            &schema_version,
            filters.clone(),
            page,
            page_limit,
        )
        .await
    {
        Ok(logs) => {
            let total = state
                .log_service
                .count_logs_by_schema_name_and_id(&schema_name, &schema_version, filters)
                .await
                .unwrap_or(0);

            let log_responses: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();

            let total_pages = if page_limit > 0 {
                ((total as f64) / (page_limit as f64)).ceil() as i32
            } else {
                0
            };

            Ok(Json(PaginatedLogsResponse {
                logs: log_responses,
                pagination: PaginationMetadata {
                    page,
                    limit: page_limit,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse::new("NOT_FOUND", e.to_string())),
            ))
        }
    }
}

pub async fn query_logs(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Json(payload): Json<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() || schema_version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name or version cannot be empty",
            )),
        ));
    }

    match state
        .log_service
        .get_logs_by_schema_name_and_id(
            &schema_name,
            &schema_version,
            payload.filters.clone(),
            payload.page,
            payload.limit,
        )
        .await
    {
        Ok(logs) => {
            let total = state
                .log_service
                .count_logs_by_schema_name_and_id(&schema_name, &schema_version, payload.filters)
                .await
                .unwrap_or(0);

            let log_responses: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();

            let total_pages = if payload.limit > 0 {
                ((total as f64) / (payload.limit as f64)).ceil() as i32
            } else {
                0
            };

            Ok(Json(PaginatedLogsResponse {
                logs: log_responses,
                pagination: PaginationMetadata {
                    page: payload.page,
                    limit: payload.limit,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse::new("NOT_FOUND", e.to_string())),
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
