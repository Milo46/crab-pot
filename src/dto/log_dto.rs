use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::{dto::cursor::CursorMetadata, AppError, AppResult, Log};

fn validate_string_not_empty(string: &String) -> Result<(), validator::ValidationError> {
    if string.trim().is_empty() {
        return Err(validator::ValidationError::new("string_empty"));
    }
    Ok(())
}

fn validate_log_data_is_object(value: &Value) -> Result<(), validator::ValidationError> {
    if !value.is_object() {
        return Err(validator::ValidationError::new("log_data_must_be_object"));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLogRequest {
    #[validate(custom(
        function = "validate_string_not_empty",
        message = "Schema ID cannot be empty"
    ))]
    pub schema_id: String,
    #[validate(custom(
        function = "validate_log_data_is_object",
        message = "Log data must be a JSON object"
    ))]
    pub log_data: Value,
}

impl CreateLogRequest {
    pub fn validate_and_transform(self) -> AppResult<CreateLogRequestValidated> {
        self.validate()
            .map_err(|e| AppError::bad_request(format!("Validation failed: {}", e)))?;

        let schema_id = Uuid::parse_str(&self.schema_id)
            .map_err(|e| AppError::bad_request(format!("Invalid UUID: {}", e)))?;

        Ok(CreateLogRequestValidated {
            schema_id,
            log_data: self.log_data,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLogRequestValidated {
    pub schema_id: Uuid,
    pub log_data: Value,
}

#[derive(Debug, Serialize)]
pub struct PaginationMetadata {
    pub page: i32,
    pub limit: i32,
    pub total: i64,
    pub total_pages: i32,
}

#[derive(Debug, Serialize)]
pub struct TimeWindowMetadata {
    pub date_begin: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedLogsResponse {
    pub schema_id: Uuid,
    pub logs: Vec<LogResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timewindow: Option<TimeWindowMetadata>,
    pub pagination: PaginationMetadata,
}

#[derive(Debug, Serialize)]
pub struct CursorLogsResponse {
    pub schema_id: Uuid,
    pub logs: Vec<LogResponse>,
    pub cursor: CursorMetadata<i32>,
}

impl CursorLogsResponse {
    pub fn new(schema_id: Uuid, logs: Vec<Log>, cursor: CursorMetadata<i32>) -> Self {
        Self {
            schema_id,
            logs: logs.into_iter().map(LogResponse::from).collect(),
            cursor,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LogsResponse {
    Paginated(PaginatedLogsResponse),
    Cursor(CursorLogsResponse),
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub id: i32,
    pub log_data: Value,
    pub schema_id: Uuid,
    pub created_at: String,
}

impl From<Log> for LogResponse {
    fn from(log: Log) -> Self {
        LogResponse {
            id: log.id,
            log_data: log.log_data,
            schema_id: log.schema_id,
            created_at: log.created_at.to_rfc3339(),
        }
    }
}

fn default_limit() -> i32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct QueryLogsRequest {
    pub date_begin: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
    pub filters: Option<Value>,
    pub cursor: Option<i32>,
    #[serde(default = "default_limit")]
    pub limit: i32,
}

// impl From<QueryLogsRequest> for QueryParams {
//     fn from(req: QueryLogsRequest) -> Self {
//         Self {
//             limit: req.limit,
//             date_begin: req.date_begin,
//             date_end: req.date_end,
//             filters: req.filters,
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "lowercase")]
pub enum LogEvent {
    Created {
        id: i32,
        schema_id: Uuid,
        log_data: Value,
        created_at: String,
    },
    Deleted {
        id: i32,
        schema_id: Uuid,
    },
}

impl LogEvent {
    pub fn created_from(log: Log) -> Self {
        LogEvent::Created {
            id: log.id,
            schema_id: log.schema_id,
            log_data: log.log_data,
            created_at: log.created_at.to_rfc3339(),
        }
    }

    pub fn deleted_from(log: Log) -> Self {
        LogEvent::Deleted {
            id: log.id,
            schema_id: log.schema_id,
        }
    }

    pub fn schema_id(&self) -> Uuid {
        match self {
            LogEvent::Created { schema_id, .. } => *schema_id,
            LogEvent::Deleted { schema_id, .. } => *schema_id,
        }
    }
}
