use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::Log;

#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
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
pub struct PaginatedLogsResponse {
    pub logs: Vec<LogResponse>,
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
    pub filters: Option<Value>,
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
