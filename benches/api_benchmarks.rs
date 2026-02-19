use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, Criterion, PlotConfiguration,
};
use log_server::{
    create_app, middleware::RateLimiter, ApiKeyRepository, ApiKeyService, AppState, LogRepository,
    LogService, SchemaRepository, SchemaResponse, SchemaService,
};
use reqwest::Client;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use testcontainers_modules::{
    postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync},
};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_postgres::NoTls;
use uuid::Uuid;

struct BenchmarkApp {
    address: String,
    client: Client,
    api_key: String,
    #[allow(unused)]
    db_pool: Pool<Postgres>,
    #[allow(unused)]
    _container: ContainerAsync<postgres::Postgres>,
}

async fn setup_benchmark_app() -> BenchmarkApp {
    let container = postgres::Postgres::default().start().await.unwrap();
    let host = container.get_host().await.unwrap().to_string();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let dsn = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .expect("Failed to connect");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let sql_files = vec![
        "./docker/db/01_extensions.sql",
        "./docker/db/02_api_keys.sql",
        "./docker/db/03_schemas.sql",
        "./docker/db/04_logs.sql",
        "./docker/db/05_functions.sql",
        "./docker/db/06_seed_data.sql",
    ];

    for sql_file in sql_files {
        let sql_content = std::fs::read_to_string(sql_file)
            .unwrap_or_else(|_| panic!("Failed to read {}", sql_file));
        client
            .batch_execute(&sql_content)
            .await
            .unwrap_or_else(|e| panic!("Failed to execute {}: {}", sql_file, e));
    }

    let pool = sqlx::postgres::PgPool::connect(&dsn)
        .await
        .expect("Failed to connect to DB");

    let schema_repo = Arc::new(SchemaRepository::new(pool.clone()));
    let log_repo = Arc::new(LogRepository::new(pool.clone()));
    let api_key_repo = Arc::new(ApiKeyRepository::new(pool.clone()));

    let schema_service = Arc::new(SchemaService::new(schema_repo.clone(), log_repo.clone()));
    let log_service = Arc::new(LogService::new(log_repo.clone(), schema_repo.clone()));
    let api_key_service = Arc::new(ApiKeyService::new(api_key_repo.clone()));

    let create_api_key_request = log_server::models::CreateApiKey::new("Benchmark API Key");
    let test_api_key = api_key_service
        .create_api_key(create_api_key_request)
        .await
        .expect("Failed to create test API key");

    let (tx, _) = broadcast::channel(16);

    let rate_limiter = Arc::new(RateLimiter::new());

    let app_state = AppState {
        schema_service,
        log_service,
        api_key_service,
        log_broadcast: tx,
        rate_limiter,
    };

    let app = create_app(app_state, pool.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    let address = listener.local_addr().unwrap();
    let address_str = format!("http://{}", address);

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("Failed to run server");
    });

    let client = Client::new();

    BenchmarkApp {
        address: address_str,
        client,
        db_pool: pool,
        api_key: test_api_key.plain_key,
        _container: container,
    }
}

async fn benchmark_health_endpoint(app: &BenchmarkApp) {
    let response = app
        .client
        .get(format!("{}/health", app.address))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_create_schema(app: &BenchmarkApp, iteration: u64) {
    let schema_payload = serde_json::json!({
        "name": format!("test_schema_{}", iteration),
        "version": "1.0.0",
        "schema_definition": {
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": ["message"]
        }
    });

    let response = app
        .client
        .post(format!("{}/schemas", app.address))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .json(&schema_payload)
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_list_schemas(app: &BenchmarkApp) {
    let response = app
        .client
        .get(format!("{}/schemas", app.address))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_create_log(app: &BenchmarkApp, schema_id: Uuid, iteration: u64) {
    let log_payload = serde_json::json!({
        "schema_id": schema_id,
        "log_data": {
            "message": format!("Sample message no. {}", iteration),
        }
    });

    let response = app
        .client
        .post(format!("{}/logs", app.address))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .json(&log_payload)
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_list_logs(app: &BenchmarkApp, schema_id: Uuid) {
    let response = app
        .client
        .get(format!("{}/logs/schemas/{}", app.address, schema_id))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_list_logs_cursor(app: &BenchmarkApp, schema_id: Uuid, cursor: i32) {
    let response = app
        .client
        .get(format!(
            "{}/logs/schemas/{}?cursor={}&limit=10",
            app.address, schema_id, cursor
        ))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_list_logs_filtered(app: &BenchmarkApp, schema_id: Uuid) {
    let response = app
        .client
        .get(format!(
            "{}/logs/schemas/{}?filters={{\"message\":\"Sample\"}}",
            app.address, schema_id
        ))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_get_log_by_id(app: &BenchmarkApp, log_id: i32) {
    let response = app
        .client
        .get(format!("{}/logs/{}", app.address, log_id))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_get_schema_by_id(app: &BenchmarkApp, schema_id: Uuid) {
    let response = app
        .client
        .get(format!("{}/schemas/{}", app.address, schema_id))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

async fn benchmark_get_initial_cursor(app: &BenchmarkApp, schema_id: Uuid) {
    let response = app
        .client
        .get(format!(
            "{}/logs/schemas/{}/cursor/initial",
            app.address, schema_id
        ))
        .header("Authorization", format!("Bearer {}", app.api_key))
        .send()
        .await
        .unwrap();

    black_box(response.status());
}

fn api_benchmarks(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let app = runtime.block_on(setup_benchmark_app());
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    {
        let mut health_group = c.benchmark_group("health_group");
        health_group.sample_size(100);
        health_group.measurement_time(std::time::Duration::from_secs(10));
        health_group.bench_function("health_check", |b| {
            b.to_async(&runtime)
                .iter(|| benchmark_health_endpoint(&app))
        });
        health_group.finish();
    }

    {
        let mut schemas_group = c.benchmark_group("schemas_endpoints");
        schemas_group.plot_config(plot_config);
        schemas_group.sample_size(100);
        schemas_group.measurement_time(std::time::Duration::from_secs(10));
        schemas_group.warm_up_time(std::time::Duration::from_secs(3));

        let mut schema_iteration = 0u64;
        schemas_group.bench_function("create_schema", |b| {
            b.to_async(&runtime).iter(|| {
                schema_iteration += 1;
                benchmark_create_schema(&app, schema_iteration)
            })
        });

        schemas_group.bench_function("list_schemas", |b| {
            b.to_async(&runtime).iter(|| benchmark_list_schemas(&app))
        });

        let sample_schema_id = runtime.block_on(async {
            let response = app
                .client
                .get(format!("{}/schemas", app.address))
                .header("Authorization", format!("Bearer {}", app.api_key))
                .send()
                .await
                .unwrap();
            let data: serde_json::Value = response.json().await.unwrap();
            let schemas = data["schemas"].as_array().unwrap();
            schemas.first().unwrap()["id"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap()
        });

        schemas_group.bench_function("get_schema_by_id", |b| {
            b.to_async(&runtime)
                .iter(|| benchmark_get_schema_by_id(&app, sample_schema_id))
        });

        schemas_group.finish();
    }

    {
        let mut logs_group = c.benchmark_group("logs_group");
        logs_group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
        logs_group.sample_size(100);
        logs_group.measurement_time(std::time::Duration::from_secs(10));
        logs_group.warm_up_time(std::time::Duration::from_secs(3));

        let create_schema_id = runtime.block_on(async {
            let schema_payload = serde_json::json!({
                "name": "bench_create_logs",
                "version": "1.0.0",
                "schema_definition": {
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    },
                    "required": ["message"]
                }
            });

            let response = app
                .client
                .post(format!("{}/schemas", app.address))
                .header("Authorization", format!("Bearer {}", app.api_key))
                .json(&schema_payload)
                .send()
                .await
                .unwrap();

            let schema: SchemaResponse = response.json().await.unwrap();
            schema.id
        });

        let mut log_iteration = 0u64;
        logs_group.bench_function("create_logs", |b| {
            b.to_async(&runtime).iter(|| {
                log_iteration += 1;
                benchmark_create_log(&app, create_schema_id, log_iteration)
            })
        });

        let (read_schema_id, sample_log_id, initial_cursor) = runtime.block_on(async {
            let schema_payload = serde_json::json!({
                "name": "bench_read_logs",
                "version": "1.0.0",
                "schema_definition": {
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    },
                    "required": ["message"]
                }
            });

            let response = app
                .client
                .post(format!("{}/schemas", app.address))
                .header("Authorization", format!("Bearer {}", app.api_key))
                .json(&schema_payload)
                .send()
                .await
                .unwrap();

            let schema: SchemaResponse = response.json().await.unwrap();
            let schema_id = schema.id;

            let mut log_id = 0;
            for i in 0..20 {
                let log_payload = serde_json::json!({
                    "schema_id": schema_id,
                    "log_data": {
                        "message": format!("Sample message no. {}", i),
                    }
                });

                let response = app
                    .client
                    .post(format!("{}/logs", app.address))
                    .header("Authorization", format!("Bearer {}", app.api_key))
                    .json(&log_payload)
                    .send()
                    .await
                    .unwrap();

                let log: serde_json::Value = response.json().await.unwrap();
                if i == 10 {
                    log_id = log["id"].as_i64().unwrap() as i32;
                }
            }

            let cursor_response = app
                .client
                .get(format!(
                    "{}/logs/schemas/{}/cursor/initial",
                    app.address, schema_id
                ))
                .header("Authorization", format!("Bearer {}", app.api_key))
                .send()
                .await
                .unwrap();

            let status = cursor_response.status();
            if !status.is_success() {
                let body = cursor_response.text().await.unwrap_or_default();
                panic!("Failed to get cursor: {} - {}", status, body);
            }

            let cursor_data: serde_json::Value = cursor_response.json().await.unwrap();
            let cursor = cursor_data["initial_cursor"].as_i64().unwrap() as i32;

            (schema_id, log_id, cursor)
        });

        logs_group.bench_function("list_logs_paginated", |b| {
            b.to_async(&runtime)
                .iter(|| benchmark_list_logs(&app, read_schema_id))
        });

        logs_group.bench_function("list_logs_cursor", |b| {
            b.to_async(&runtime)
                .iter(|| benchmark_list_logs_cursor(&app, read_schema_id, initial_cursor))
        });

        logs_group.bench_function("list_logs_filtered", |b| {
            b.to_async(&runtime)
                .iter(|| benchmark_list_logs_filtered(&app, read_schema_id))
        });

        logs_group.bench_function("get_log_by_id", |b| {
            b.to_async(&runtime)
                .iter(|| benchmark_get_log_by_id(&app, sample_log_id))
        });

        logs_group.bench_function("get_initial_cursor", |b| {
            b.to_async(&runtime)
                .iter(|| benchmark_get_initial_cursor(&app, read_schema_id))
        });

        logs_group.finish();
    }

    runtime.block_on(async move {
        std::mem::drop(app);
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05)
        .significance_level(0.05);
    targets = api_benchmarks
}
criterion_main!(benches);
