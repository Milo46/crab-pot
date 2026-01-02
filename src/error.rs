use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

use crate::middleware::RequestId;

#[derive(Debug, Clone)]
pub struct AppError {
    kind: AppErrorKind,
    request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AppErrorKind {
    NotFound(String),              // Resource not found (404)
    ValidationError(String),       // Validation error (400)
    Conflict(String),              // Conflict with existing resource (409)
    DatabaseError(String),         // Database operation failed (500)
    InternalError(String),         // Internal server error (500)
    BadRequest(String),            // Bad request (400)
    SchemaValidationError(String), // Schema validation failed (422)
}

impl AppError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::NotFound(msg.into()),
            request_id: None,
        }
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::ValidationError(msg.into()),
            request_id: None,
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::Conflict(msg.into()),
            request_id: None,
        }
    }

    pub fn database_error(msg: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::DatabaseError(msg.into()),
            request_id: None,
        }
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::InternalError(msg.into()),
            request_id: None,
        }
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::BadRequest(msg.into()),
            request_id: None,
        }
    }

    pub fn schema_validation_error(msg: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::SchemaValidationError(msg.into()),
            request_id: None,
        }
    }

    pub fn context(mut self, context: impl Into<String>) -> Self {
        let context = context.into();
        self.kind = match self.kind {
            AppErrorKind::NotFound(msg) => AppErrorKind::NotFound(format!("{}: {}", context, msg)),
            AppErrorKind::ValidationError(msg) => {
                AppErrorKind::ValidationError(format!("{}: {}", context, msg))
            }
            AppErrorKind::Conflict(msg) => AppErrorKind::Conflict(format!("{}: {}", context, msg)),
            AppErrorKind::DatabaseError(msg) => {
                AppErrorKind::DatabaseError(format!("{}: {}", context, msg))
            }
            AppErrorKind::InternalError(msg) => {
                AppErrorKind::InternalError(format!("{}: {}", context, msg))
            }
            AppErrorKind::BadRequest(msg) => {
                AppErrorKind::BadRequest(format!("{}: {}", context, msg))
            }
            AppErrorKind::SchemaValidationError(msg) => {
                AppErrorKind::SchemaValidationError(format!("{}: {}", context, msg))
            }
        };
        self
    }

    pub fn with_request_id(mut self, request_id: &RequestId) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    fn error_type(&self) -> &str {
        match self.kind {
            AppErrorKind::NotFound(_) => "NOT_FOUND",
            AppErrorKind::ValidationError(_) => "VALIDATION_ERROR",
            AppErrorKind::Conflict(_) => "CONFLICT",
            AppErrorKind::DatabaseError(_) => "DATABASE_ERROR",
            AppErrorKind::InternalError(_) => "INTERNAL_ERROR",
            AppErrorKind::BadRequest(_) => "BAD_REQUEST",
            AppErrorKind::SchemaValidationError(_) => "SCHEMA_VALIDATION_ERROR",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self.kind {
            AppErrorKind::NotFound(_) => StatusCode::NOT_FOUND,
            AppErrorKind::ValidationError(_) | AppErrorKind::BadRequest(_) => {
                StatusCode::BAD_REQUEST
            }
            AppErrorKind::Conflict(_) => StatusCode::CONFLICT,
            AppErrorKind::DatabaseError(_) | AppErrorKind::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppErrorKind::SchemaValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    fn user_message(&self) -> String {
        match &self.kind {
            // For internal errors, don't expose details
            AppErrorKind::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                "A database error occurred".to_string()
            }
            AppErrorKind::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                "An internal error occurred".to_string()
            }
            _ => self.to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AppErrorKind::NotFound(msg) => write!(f, "{}", msg),
            AppErrorKind::ValidationError(msg) => write!(f, "{}", msg),
            AppErrorKind::Conflict(msg) => write!(f, "{}", msg),
            AppErrorKind::DatabaseError(msg) => write!(f, "{}", msg),
            AppErrorKind::InternalError(msg) => write!(f, "{}", msg),
            AppErrorKind::BadRequest(msg) => write!(f, "{}", msg),
            AppErrorKind::SchemaValidationError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_type = self.error_type();
        let message = self.user_message();

        let mut body = json!({
            "error": error_type,
            "message": message,
        });

        if let Some(request_id) = self.request_id {
            body["request_id"] = json!(request_id);
        }

        (status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::internal_error(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::not_found("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => {
                            return AppError::conflict(
                                "A resource with these attributes already exists",
                            )
                        }
                        "23503" => {
                            let message = db_err.message();

                            if message.contains("fk_logs_schema_id") {
                                return AppError::not_found("Schema not found");
                            } else if message.contains("schema") {
                                return AppError::not_found("Referenced schema does not exist");
                            } else {
                                return AppError::bad_request("Referenced resource does not exist");
                            }
                        }
                        _ => {}
                    }
                }
                AppError::database_error(db_err.to_string())
            }
            _ => AppError::database_error(err.to_string()),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub trait WithRequestId<T> {
    fn with_req_id(self, request_id: &RequestId) -> AppResult<T>;
}

impl<T> WithRequestId<T> for AppResult<T> {
    fn with_req_id(self, request_id: &RequestId) -> AppResult<T> {
        self.map_err(|e| e.with_request_id(request_id))
    }
}
