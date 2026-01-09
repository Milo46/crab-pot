use axum::{
    http::StatusCode,
    middleware as axum_middleware,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub mod dto;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod services;

use crate::{
    handlers::{
        create_log, create_schema, delete_log, delete_schema, get_log_by_id, get_logs,
        get_logs_query, get_schema_by_id, get_schema_by_name_and_version, get_schemas,
        log_handlers::{
            get_logs_by_schema_name_and_version, get_logs_by_schema_name_and_version_query,
        },
        update_schema, ws_handler,
    },
    middleware::api_key_middleware,
};

pub use dto::{LogEvent, PaginatedLogsResponse, PaginationMetadata, SchemaResponse};
pub use error::{AppError, AppResult};
pub use middleware::request_id::{RequestIdLayer, RequestIdMakeSpan};
pub use models::{Log, QueryParams, Schema, SchemaNameVersion};
pub use repositories::{ApiKeyRepository, LogRepository, SchemaRepository};
pub use services::{ApiKeyService, LogService, SchemaService};

#[derive(Clone)]
pub struct AppState {
    pub schema_service: Arc<SchemaService>,
    pub log_service: Arc<LogService>,
    pub api_key_service: Arc<ApiKeyService>,
    pub log_broadcast: broadcast::Sender<LogEvent>,
}

impl AppState {
    pub fn new(
        schema_service: Arc<SchemaService>,
        log_service: Arc<LogService>,
        api_key_service: Arc<ApiKeyService>,
        log_broadcast: broadcast::Sender<LogEvent>,
    ) -> Self {
        Self {
            schema_service,
            log_service,
            api_key_service,
            log_broadcast,
        }
    }
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Health check endpoint called");
    Ok(Json(json!({
        "status": "healthy",
        "service": "log-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

pub fn create_app(app_state: AppState, _pool: PgPool) -> Router {
    let public_routes = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check));

    let schema_routes = Router::new()
        .route("/schemas", get(get_schemas))
        .route("/schemas", post(create_schema))
        .route("/schemas/{id}", get(get_schema_by_id))
        .route("/schemas/{id}", put(update_schema))
        .route("/schemas/{id}", delete(delete_schema))
        .route(
            "/schemas/by-name/{schema_name}/versions/{schema_version}",
            get(get_schema_by_name_and_version),
        );

    let log_routes = Router::new()
        .route("/logs", post(create_log))
        // .route("/logs/bulk", post(create_logs_bulk))
        .route("/logs/{id}", get(get_log_by_id))
        .route("/logs/{id}", delete(delete_log))
        .route("/logs/schemas/{schema_id}", get(get_logs))
        .route("/logs/schemas/{schema_id}", post(get_logs_query))
        // .route("/logs/by-schema-name/{name}", get(...))
        // .route("/logs/by-schema-name/{name}", post(...))
        .route(
            "/logs/by-schema-name/{name}/versions/{version}",
            get(get_logs_by_schema_name_and_version),
        )
        .route(
            "/logs/by-schema-name/{name}/versions/{version}",
            post(get_logs_by_schema_name_and_version_query),
        );

    let ws_routes = Router::new().route("/ws/logs", get(ws_handler));

    let protected_routes = Router::new()
        .merge(schema_routes)
        .merge(log_routes)
        .merge(ws_routes)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            api_key_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(axum_middleware::from_fn(RequestIdLayer::middleware))
                .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan))
                .layer(CorsLayer::permissive()),
        )
}

pub fn create_admin_app(app_state: AppState) -> Router {
    use crate::handlers::{
        create_api_key, delete_api_key, get_api_key_by_id, get_api_keys, rotate_api_key,
    };

    let admin_health_check = || async {
        Json(json!({
            "status": "healthy",
            "service": "log-server-admin",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    };

    Router::new()
        .route("/", get(admin_health_check))
        .route("/health", get(admin_health_check))
        .route("/api-keys", post(create_api_key))
        .route("/api-keys", get(get_api_keys))
        .route("/api-keys/{key_id}", get(get_api_key_by_id))
        .route("/api-keys/{key_id}", delete(delete_api_key))
        .route("/api-keys/{key_id}/rotate", post(rotate_api_key))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(axum_middleware::from_fn(RequestIdLayer::middleware))
                .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan))
                .layer(CorsLayer::permissive()),
        )
}
