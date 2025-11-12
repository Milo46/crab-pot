use axum::{
    routing::{get, post, delete, put},
    Router,
};
use tokio::net::TcpListener;
use std::{env, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
};

mod models;
mod repositories;
mod services;
mod handlers;

use repositories::{SchemaRepository, LogRepository};
use services::{SchemaService, LogService};
use handlers::{
    get_schemas, get_schema_by_id, create_schema, update_schema, delete_schema,
    get_logs, get_log_by_id, create_log, delete_log,
};
use axum::{response::Json, http::StatusCode};
use serde_json::json;
use chrono;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::fmt::format::FmtSpan;    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tower_http=debug,log_server=debug,info".into())
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE) 
        .init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable is not set");
    
    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    let schema_repository = Arc::new(SchemaRepository::new(pool.clone()));
    let log_repository = Arc::new(LogRepository::new(pool.clone()));
    
    let schema_service = Arc::new(SchemaService::new(schema_repository.clone()));
    let log_service = Arc::new(LogService::new(log_repository, schema_repository));
    
    let app_state = AppState {
        schema_service,
        log_service,
    };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/schemas", get(get_schemas))
        .route("/schemas", post(create_schema))
        .route("/schemas/:id", get(get_schema_by_id))
        .route("/schemas/:id", put(update_schema))
        .route("/schemas/:id", delete(delete_schema))
        .route("/logs", post(create_log))
        .route("/logs/schema/:schema_id", get(get_logs))
        .route("/logs/:id", get(get_log_by_id))
        .route("/logs/:id", delete(delete_log))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // Optional: CORS support
        )
        .with_state(app_state);
    
    tracing::info!("🚀 Log Server starting up at http://0.0.0.0:8080");
    tracing::info!("✅ Database connected successfully!");
    tracing::info!("📊 Available endpoints:");
    tracing::info!("   GET    /                     - Health check");
    tracing::info!("   GET    /health               - Health check");
    tracing::info!("   GET    /schemas              - Get all schemas");
    tracing::info!("   POST   /schemas              - Create new schema");
    tracing::info!("   GET    /schemas/:id          - Get schema by ID");
    tracing::info!("   PUT    /schemas/:id          - Update schema");
    tracing::info!("   DELETE /schemas/:id          - Delete schema");
    tracing::info!("   POST   /logs                      - Create new log entry");
    tracing::info!("   GET    /logs/schema/:schema_id - Get logs by schema ID");
    tracing::info!("   GET    /logs/:id               - Get log by ID");
    tracing::info!("   DELETE /logs/:id               - Delete log");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}
