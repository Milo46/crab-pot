use std::sync::Arc;

pub mod models;
pub mod repositories;
pub mod services;
pub mod handlers;

// Re-export key types for external use
pub use models::{Log, Schema};
pub use services::{SchemaService, LogService};
pub use handlers::{
    get_schemas, get_schema_by_id, create_schema, update_schema, delete_schema,
    get_logs, get_log_by_id, create_log, delete_log, ErrorResponse
};

#[derive(Clone)]
pub struct AppState {
    pub schema_service: Arc<SchemaService>,
    pub log_service: Arc<LogService>,
}
