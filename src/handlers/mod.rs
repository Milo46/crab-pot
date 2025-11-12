use serde::Serialize;

pub mod schema_handlers;
pub mod log_handlers;

// Shared error response type
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// Re-export specific functions to avoid conflicts
pub use schema_handlers::{
    get_schemas, get_schema_by_id, create_schema, update_schema, delete_schema
};
pub use log_handlers::{
    get_logs, get_log_by_id, create_log, delete_log
};
