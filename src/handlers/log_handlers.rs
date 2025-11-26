use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{
    dto::{CreateLogRequest, ErrorResponse, LogResponse},
    AppState,
};

pub async fn get_logs_default(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() || schema_version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name or version cannot be empty",
            )),
        ));
    }

    let filters: Option<Value> = if params.is_empty() {
        None
    } else {
        let mut filter_obj = serde_json::Map::new();
        for (key, value) in params {
            let json_value = serde_json::from_str::<Value>(&value).unwrap_or(Value::String(value));
            filter_obj.insert(key, json_value);
        }
        Some(Value::Object(filter_obj))
    };

    match state
        .log_service
        .get_logs_by_schema_name_and_id(&schema_name, &schema_version, filters)
        .await
    {
        Ok(logs) => {
            let log_responses: Vec<LogResponse> = logs
                .into_iter()
                .map(|l| LogResponse {
                    id: l.id,
                    schema_id: l.schema_id,
                    log_data: l.log_data,
                    created_at: l.created_at.to_rfc3339(),
                })
                .collect();

            Ok(Json(json!({ "logs": log_responses })))
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
        Ok(Some(log)) => Ok(Json(LogResponse {
            id: log.id,
            schema_id: log.schema_id,
            log_data: log.log_data,
            created_at: log.created_at.to_rfc3339(),
        })),
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
        Ok(log) => Ok((
            StatusCode::CREATED,
            Json(LogResponse {
                id: log.id,
                schema_id: log.schema_id,
                log_data: log.log_data,
                created_at: log.created_at.to_rfc3339(),
            }),
        )),
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
    match state.log_service.delete_log(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
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
