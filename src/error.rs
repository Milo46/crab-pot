use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    // Resource not found (404)
    NotFound(String),

    // Validation error (400)
    ValidationError(String),

    // Conflict with existing resource (409)
    Conflict(String),

    // Database operation failed (500)
    DatabaseError(String),

    // Internal server error (500)
    InternalError(String),

    // Bad request (400)
    BadRequest(String),

    // Schema validation failed (422)
    SchemaValidationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::SchemaValidationError(msg) => write!(f, "Schema validation error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NotFound", msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "ValidationError", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict", msg),
            AppError::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DatabaseError",
                    "A database error occurred".to_string(),
                )
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalError",
                    "An internal error occurred".to_string(),
                )
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BadRequest", msg),
            AppError::SchemaValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "SchemaValidationError",
                msg,
            ),
        };

        let body = Json(json!({
            "error": {
                "type": error_type,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        // PostgreSQL unique violation
                        return AppError::Conflict(
                            "A resource with these attributes already exists".to_string(),
                        );
                    }
                    if code == "23503" {
                        // PostgreSQL foreign key violation
                        return AppError::BadRequest(
                            "Referenced resource does not exist".to_string(),
                        );
                    }
                }
                AppError::DatabaseError(db_err.to_string())
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
