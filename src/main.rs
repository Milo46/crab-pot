use log_server::{
    create_app, AppState, LogRepository, LogService, SchemaRepository, SchemaService,
};
use std::net::SocketAddr;
use std::{env, sync::Arc};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::fmt::format::FmtSpan;
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tower_http=debug,log_server=debug,info".into()),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");

    let pool = sqlx::postgres::PgPool::connect(&database_url).await?;
    tracing::info!("✅ Database connected successfully!");

    let schema_repository = Arc::new(SchemaRepository::new(pool.clone()));
    let log_repository = Arc::new(LogRepository::new(pool.clone()));

    let schema_service = Arc::new(SchemaService::new(
        schema_repository.clone(),
        log_repository.clone(),
    ));
    let log_service = Arc::new(LogService::new(log_repository.clone()));

    let (log_broadcast_tx, _) = broadcast::channel(100);

    let app_state = AppState {
        schema_service,
        log_service,
        log_broadcast: log_broadcast_tx,
    };

    let app = create_app(app_state);

    let span = tracing::info_span!("API Endpoints");
    {
        let _enter = span.enter();
        tracing::info!("📊 Available endpoints:");
        tracing::info!("Schemas:");
        tracing::info!("  GET              /");
        tracing::info!("  GET              /health");
        tracing::info!("  GET, POST        /schemas");
        tracing::info!("  GET, PUT, DELETE /schemas/id/{{uuid}}");
        tracing::info!(
            "  GET              /schemas/name/{{schema_name}}/version/{{schema_version}}"
        );
        tracing::info!("");
        tracing::info!("Logs:");
        tracing::info!("  POST             /logs");
        tracing::info!("  GET              /logs/schema/{{schema_name}}");
        tracing::info!("  POST             /logs/schema/{{schema_name}}/query");
        tracing::info!(
            "  GET              /logs/schema/{{schema_name}}/version/{{schema_version}}"
        );
        tracing::info!(
            "  POST             /logs/schema/{{schema_name}}/version/{{schema_version}}/query"
        );
        tracing::info!("  GET, DELETE      /logs/{{id}}");
        tracing::info!("");
        tracing::info!("WebSocket:");
        tracing::info!("  GET              /ws/logs");
    }

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    tracing::info!("🚀 Log Server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
