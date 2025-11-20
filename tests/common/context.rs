use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

pub struct TestContext {
    pub client: Client,
    pub base_url: String,
}

impl TestContext {
    pub async fn new() -> Self {
        wait_for_service().await.expect("Service should be ready");

        let base_url = get_test_base_url();

        Self {
            client: Client::new(),
            base_url,
        }
    }
}

fn get_test_base_url() -> String {
    std::env::var("TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:8082".to_string())
}

async fn wait_for_service() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut retries = 30;
    let base_url = get_test_base_url();
    
    while retries > 0 {
        match client.get(&format!("{}/health", base_url)).send().await {
            Ok(response) if response.status().is_success() => {
                return Ok(());
            }
            _ => {
                retries -= 1;
                if retries > 0 {
                    println!("Service not ready, retrying in 1 second... ({} retries left)", retries);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
    
    Err("Service did not become ready in time".into())
}
