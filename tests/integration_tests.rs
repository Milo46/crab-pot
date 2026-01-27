mod api_keys;
mod common;
mod logs;
mod schemas;

mod health {
    use crate::common::test_app::setup_test_app;
    use reqwest::StatusCode;

    #[tokio::test]
    async fn health_check_works() {
        let app = setup_test_app().await;

        let response = app
            .client
            .get(format!("{}/health", app.address))
            .header("X-Api-Key", "secret-key")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "log-server");
        assert!(body["timestamp"].is_string());
    }
}
