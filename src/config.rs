use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub main_api_addr: SocketAddr,
    pub admin_api_addr: SocketAddr,
    pub broadcast_channel_size: usize,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL environment variable is not set"),

            main_api_addr: std::env::var("MAIN_API_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
                .parse()?,

            admin_api_addr: std::env::var("ADMIN_API_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
                .parse()?,

            broadcast_channel_size: std::env::var("BROADCAST_CHANNEL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),

            rust_log: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "tower_http=debug,log_server=debug,info".to_string()),
        })
    }
}
