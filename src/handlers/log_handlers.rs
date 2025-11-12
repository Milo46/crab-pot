use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AppState;

// Request/Response DTOs for API layer
#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    pub schema_id: String,
    pub log_data: Value,
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub id: i32,
    pub schema_id: String,
    pub log_data: Value,
    pub created_at: String,
}

use super::ErrorResponse;

// Handler functions - only responsible for HTTP concerns
pub async fn get_logs(
    State(state): State<AppState>,
    Path(schema_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // Input validation
    if schema_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema ID cannot be empty".to_string(),
            }),
        ));
    }

    match state.log_service.get_logs_by_schema_id(&schema_id).await {
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
                Json(ErrorResponse {
                    error: "FETCH_FAILED".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    }
}

pub async fn get_log_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<LogResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.log_service.get_log_by_id(&id).await {
        Ok(Some(log)) => Ok(Json(LogResponse {
            id: log.id,
            schema_id: log.schema_id,
            log_data: log.log_data,
            created_at: log.created_at.to_rfc3339(),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Log with id '{}' not found", id),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "FETCH_FAILED".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

pub async fn create_log(
    State(state): State<AppState>,
    Json(payload): Json<CreateLogRequest>,
) -> Result<Json<LogResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Input validation
    if payload.schema_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema ID cannot be empty".to_string(),
            }),
        ));
    }

    if !payload.log_data.is_object() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Log data must be a JSON object".to_string(),
            }),
        ));
    }

    match state.log_service
        .create_log(payload.schema_id, payload.log_data)
        .await
    {
        Ok(log) => Ok(Json(LogResponse {
            id: log.id,
            schema_id: log.schema_id,
            log_data: log.log_data,
            created_at: log.created_at.to_rfc3339(),
        })),
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("validation") || e.to_string().contains("Required field") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status_code,
                Json(ErrorResponse {
                    error: "CREATION_FAILED".to_string(),
                    message: e.to_string(),
                }),
            ))
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
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Log with id '{}' not found", id),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "DELETION_FAILED".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
