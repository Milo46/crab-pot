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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum LogAction {
    Create { schema_id: Uuid, log_data: Value },
    Delete { id: i32 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "response_type", rename_all = "lowercase")]
pub enum LogActionResponse {
    Success {
        #[serde(flatten)]
        event: LogEvent,
    },
    Error {
        action: String,
        message: String,
    },
}

impl LogActionResponse {
    pub fn success(event: LogEvent) -> Self {
        LogActionResponse::Success { event }
    }

    pub fn error(action: impl Into<String>, message: impl Into<String>) -> Self {
        LogActionResponse::Error {
            action: action.into(),
            message: message.into(),
        }
    }
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
