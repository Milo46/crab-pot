#[path = "common/mod.rs"]
mod common;

mod schemas;
mod logs;

mod health {
    use reqwest::StatusCode;
    use crate::common::TestContext;

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let ctx = TestContext::new().await;
        
        let response = ctx.client
            .get(&format!("{}/health", ctx.base_url))
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse JSON");
        
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "log-server");
        assert!(body["timestamp"].is_string());
    }

    #[tokio::test]
    async fn root_endpoint_returns_health() {
        let ctx = TestContext::new().await;
        
        let response = ctx.client
            .get(&ctx.base_url)
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse JSON");
        
        assert_eq!(body["status"], "healthy");
    }
}
