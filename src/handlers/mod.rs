use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod log_handlers;
pub mod schema_handlers;

// Shared error response type
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<HashMap<String, Vec<String>>>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            field_errors: None,
        }
    }

    pub fn with_field_errors(
        error: impl Into<String>,
        message: impl Into<String>,
        field_errors: HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            field_errors: Some(field_errors),
        }
    }
}

// Re-export specific functions to avoid conflicts
pub use log_handlers::{create_log, delete_log, get_log_by_id, get_logs, get_logs_default};
pub use schema_handlers::{
    create_schema, delete_schema, get_schema_by_id, get_schema_by_name_and_version, get_schemas,
    update_schema, SchemaResponse,
};
