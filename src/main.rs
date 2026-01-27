use log_server::{
    create_admin_app, create_app, ApiKeyRepository, ApiKeyService, AppState, LogRepository,
    LogService, SchemaRepository, SchemaService,
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
    let api_key_repository = Arc::new(ApiKeyRepository::new(pool.clone()));

    let schema_service = Arc::new(SchemaService::new(
        schema_repository.clone(),
        log_repository.clone(),
    ));
    let log_service = Arc::new(LogService::new(
        log_repository.clone(),
        schema_repository.clone(),
    ));
    let api_key_service = Arc::new(ApiKeyService::new(api_key_repository.clone()));

    let (log_broadcast_tx, _) = broadcast::channel(100);

    let app_state = AppState {
        schema_service,
        log_service,
        api_key_service,
        log_broadcast: log_broadcast_tx,
    };

    let app = create_app(app_state.clone(), pool);
    let admin_app = create_admin_app(app_state.clone());

    tracing::info!("📊 Main API endpoints:");

    tracing::info!("Schemas:");
    tracing::info!("  GET               /");
    tracing::info!("  GET               /health");
    tracing::info!("  GET, POST         /schemas");
    tracing::info!("  GET, PUT, DELETE  /schemas/{{id}}");
    tracing::info!("  GET               /schemas/by-name/{{name}}/versions/{{version}}");
    tracing::info!("Logs:");
    tracing::info!("  POST         /logs");
    tracing::info!("  GET, DELETE  /logs/{{id}}");
    tracing::info!("  GET, POST    /logs/schemas/{{schema_id}}");
    tracing::info!("  GET, POST    /logs/by-schema-name/{{name}}/versions/{{version}}");
    tracing::info!("WebSocket:");
    tracing::info!("  GET  /ws/logs");

    let main_addr: SocketAddr = env::var("MAIN_API_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;

    let admin_addr: SocketAddr = env::var("ADMIN_API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
        .parse()?;

    tracing::info!("");
    tracing::info!("🚀 Main API Server running at http://{}", main_addr);
    tracing::info!("🔐 Admin API Server running at http://{}", admin_addr);
    tracing::info!("");
    tracing::warn!(
        "⚠️  SECURITY: Admin API is bound to {}. Ensure this is properly secured!",
        admin_addr
    );
    tracing::warn!("⚠️  For production, bind admin API to 127.0.0.1 and use SSH tunnel or VPN.");

    let main_listener = TcpListener::bind(main_addr).await?;
    let admin_listener = TcpListener::bind(admin_addr).await?;

    let admin_server = tokio::spawn(async move {
        tracing::info!("Starting Admin API server...");
        if let Err(e) = axum::serve(admin_listener, admin_app).await {
            tracing::error!("Admin API server error: {}", e);
        }
    });

    tracing::info!("Starting Main API server...");
    let main_result = axum::serve(
        main_listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await;

    let _ = tokio::join!(admin_server);

    main_result?;
    Ok(())
}
