use crate::common::{create_valid_schema, TestApp};
use log_server::dto::SchemaResponse;
use reqwest::StatusCode;

#[tokio::test]
async fn test_rate_limit_enforcement() {
    let app = TestApp::spawn().await;

    let schema_response = create_valid_schema(&app, "test-schema").await;
    let schema: SchemaResponse = schema_response.json().await.unwrap();

    let limited_key = app.create_api_key_with_limits(2, 4).await;

    for i in 0..4 {
        let response = app
            .client
            .get(format!(
                "{}/logs/by-schema-name/{}/versions/{}",
                app.address, schema.name, schema.version
            ))
            .header("Authorization", format!("Bearer {}", limited_key))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i + 1
        );

        assert!(response.headers().get("x-ratelimit-limit").is_some());
        assert!(response.headers().get("x-ratelimit-remaining").is_some());
    }

    let response = app
        .client
        .get(format!(
            "{}/logs/by-schema-name/{}/versions/{}",
            app.address, schema.name, schema.version
        ))
        .header("Authorization", format!("Bearer {}", limited_key))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(response.headers().get("retry-after").is_some());
}

#[tokio::test]
async fn test_rate_limit_headers() {
    let app = TestApp::spawn().await;
    let schema_response = create_valid_schema(&app, "test-schema").await;
    let schema: SchemaResponse = schema_response.json().await.unwrap();

    let limited_key = app.create_api_key_with_limits(5, 10).await;

    let response = app
        .client
        .get(format!(
            "{}/logs/by-schema-name/{}/versions/{}",
            app.address, schema.name, schema.version
        ))
        .header("Authorization", format!("Bearer {}", limited_key))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert_eq!(
        headers.get("x-ratelimit-limit").unwrap().to_str().unwrap(),
        "10"
    );
    assert_eq!(
        headers
            .get("x-ratelimit-remaining")
            .unwrap()
            .to_str()
            .unwrap(),
        "9"
    );
    assert!(headers.get("x-ratelimit-reset").is_some());
}

#[tokio::test]
async fn test_rate_limit_per_key_isolation() {
    let app = TestApp::spawn().await;
    let schema_response = create_valid_schema(&app, "test-schema").await;
    let schema: SchemaResponse = schema_response.json().await.unwrap();

    let key1 = app.create_api_key_with_limits(2, 3).await;
    let key2 = app.create_api_key_with_limits(2, 3).await;

    for _ in 0..3 {
        let _ = app
            .client
            .get(format!(
                "{}/logs/by-schema-name/{}/versions/{}",
                app.address, schema.name, schema.version
            ))
            .header("Authorization", format!("Bearer {}", key1))
            .send()
            .await;
    }

    let response = app
        .client
        .get(format!(
            "{}/logs/by-schema-name/{}/versions/{}",
            app.address, schema.name, schema.version
        ))
        .header("Authorization", format!("Bearer {}", key1))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let response = app
        .client
        .get(format!(
            "{}/logs/by-schema-name/{}/versions/{}",
            app.address, schema.name, schema.version
        ))
        .header("Authorization", format!("Bearer {}", key2))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
