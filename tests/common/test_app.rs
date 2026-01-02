use log_server::{
    create_app, ApiKeyRepository, ApiKeyService, AppState, LogRepository, LogService,
    SchemaRepository, SchemaService,
};
use reqwest::{Client, Method, RequestBuilder};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use testcontainers_modules::{
    postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync},
};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_postgres::NoTls;

pub struct TestApp {
    pub address: String,
    pub client: Client,
    #[allow(unused)]
    pub db_pool: Pool<Postgres>,
    api_key: String,
    _container: ContainerAsync<postgres::Postgres>,
}

impl TestApp {
    pub fn auth(&self) -> AuthClient<'_> {
        AuthClient { app: self }
    }
}

pub struct AuthClient<'a> {
    app: &'a TestApp,
}

impl<'a> AuthClient<'a> {
    fn url(&self, path: impl AsRef<str>) -> String {
        format!("{}{}", self.app.address, path.as_ref())
    }

    pub fn request(&self, method: Method, path: impl AsRef<str>) -> RequestBuilder {
        self.app
            .client
            .request(method, self.url(path))
            .header("Authorization", format!("Bearer {}", self.app.api_key))
    }

    pub fn get(&self, path: impl AsRef<str>) -> reqwest::RequestBuilder {
        self.request(Method::GET, path)
    }

    pub fn post(&self, path: impl AsRef<str>) -> reqwest::RequestBuilder {
        self.request(Method::POST, path)
    }

    pub fn put(&self, path: impl AsRef<str>) -> reqwest::RequestBuilder {
        self.request(Method::PUT, path)
    }

    pub fn delete(&self, path: impl AsRef<str>) -> reqwest::RequestBuilder {
        self.request(Method::DELETE, path)
    }
}

pub async fn setup_test_app() -> TestApp {
    let container = postgres::Postgres::default().start().await.unwrap();
    let host = container.get_host().await.unwrap().to_string();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let dsn = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .expect("Failed to connect");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("conncetion error: {}", e);
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
        .expect("Failed to conncet to DB");

    let schema_repo = Arc::new(SchemaRepository::new(pool.clone()));
    let log_repo = Arc::new(LogRepository::new(pool.clone()));
    let api_key_repo = Arc::new(ApiKeyRepository::new(pool.clone()));

    let schema_service = Arc::new(SchemaService::new(schema_repo.clone(), log_repo.clone()));
    let log_service = Arc::new(LogService::new(log_repo.clone()));
    let api_key_service = Arc::new(ApiKeyService::new(api_key_repo.clone()));

    let create_api_key_request = log_server::models::CreateApiKey::new("Test API Key");
    let test_api_key = api_key_service
        .create_api_key(create_api_key_request)
        .await
        .expect("Failed to create test API key");

    let (tx, _) = broadcast::channel(16);

    let app_state = AppState {
        schema_service,
        log_service,
        api_key_service,
        log_broadcast: tx,
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

    TestApp {
        address: address_str,
        client,
        db_pool: pool,
        api_key: test_api_key.plain_key,
        _container: container,
    }
}
