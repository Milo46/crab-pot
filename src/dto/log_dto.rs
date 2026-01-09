use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::{Log, QueryParams};

fn validate_uuid_not_nil(uuid: &Uuid) -> Result<(), validator::ValidationError> {
    if uuid.is_nil() {
        return Err(validator::ValidationError::new("uuid_nil"));
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
        function = "validate_uuid_not_nil",
        message = "Schema ID cannot be nil"
    ))]
    pub schema_id: Uuid,
    #[validate(custom(
        function = "validate_log_data_is_object",
        message = "Log data must be a JSON object"
    ))]
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
    pub logs: Vec<LogResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timewindow: Option<TimeWindowMetadata>,
    pub pagination: PaginationMetadata,
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub id: i32,
    pub schema_id: Uuid,
    pub log_data: Value,
    pub created_at: String,
}

impl From<Log> for LogResponse {
    fn from(log: Log) -> Self {
        LogResponse {
            id: log.id,
            schema_id: log.schema_id,
            log_data: log.log_data,
            created_at: log.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryLogsRequest {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub date_begin: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
    pub filters: Option<Value>,
}

impl From<QueryLogsRequest> for QueryParams {
    fn from(req: QueryLogsRequest) -> Self {
        Self {
            page: req.page,
            limit: req.limit,
            date_begin: req.date_begin,
            date_end: req.date_end,
            filters: req.filters,
        }
    }
}

fn default_page() -> i32 {
    1
}
fn default_limit() -> i32 {
    10
}

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
