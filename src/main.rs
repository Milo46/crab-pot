use tokio::net::TcpListener;
use std::{env, sync::Arc};
use std::net::SocketAddr;
use log_server::{create_app, AppState, SchemaRepository, LogRepository, SchemaService, LogService};

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
    
    let pool = sqlx::postgres::PgPool::connect(&database_url).await?;
    tracing::info!("✅ Database connected successfully!");
    
    let schema_repository = Arc::new(SchemaRepository::new(pool.clone()));
    let log_repository = Arc::new(LogRepository::new(pool.clone()));
    
    let schema_service = Arc::new(SchemaService::new(schema_repository.clone(), log_repository.clone()));
    let log_service = Arc::new(LogService::new(log_repository.clone(), schema_repository));
    
    let app_state = AppState {
        schema_service,
        log_service,
    };

    let app = create_app(app_state);
    
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

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    tracing::info!("🚀 Log Server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
