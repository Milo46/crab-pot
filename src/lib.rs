use axum::{
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub mod dto;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

pub use dto::SchemaResponse;
pub use error::{AppError, AppResult};
pub use handlers::{
    create_log, create_schema, delete_log, delete_schema, get_log_by_id, get_logs,
    get_logs_default, get_schema_by_id, get_schema_by_name_and_version, get_schemas, update_schema,
};
pub use models::{Log, Schema};
pub use repositories::{LogRepository, SchemaRepository};
pub use services::{LogService, SchemaService};

#[derive(Clone)]
pub struct AppState {
    pub schema_service: Arc<SchemaService>,
    pub log_service: Arc<LogService>,
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Health check endpoint called");
    Ok(Json(json!({
        "status": "healthy",
        "service": "log-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

pub fn create_app(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/schemas", get(get_schemas))
        .route("/schemas", post(create_schema))
        .route("/schemas/{id}", get(get_schema_by_id))
        .route("/schemas/{id}", put(update_schema))
        .route("/schemas/{id}", delete(delete_schema))
        .route(
            "/schemas/{schema_name}/{schema_version}",
            get(get_schema_by_name_and_version),
        )
        .route("/logs", post(create_log))
        .route("/logs/schema/{schema_name}", get(get_logs_default))
        .route("/logs/schema/{schema_name}/{schema_version}", get(get_logs))
        .route("/logs/{id}", get(get_log_by_id))
        .route("/logs/{id}", delete(delete_log))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(app_state)
}
