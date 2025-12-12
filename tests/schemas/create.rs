use log_server::{ErrorResponse, Schema};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::{fixtures::valid_schema_payload, test_app::{setup_test_app, TestApp}};

async fn post_schema_with_payload(
    app: &TestApp,
    payload: serde_json::Value
) -> reqwest::Response {
    app.client
        .post(format!("{}/schemas", app.address))
        .header("X-Api-Key", "secret-key")
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request")
}

async fn post_schema(app: &TestApp, name: &str) -> reqwest::Response {
    post_schema_with_payload(app, valid_schema_payload(name)).await
}

#[tokio::test]
async fn creates_schema_with_valid_data() {
    let app = setup_test_app().await;
    let response = post_schema(&app, "test-schema").await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let schema: Schema = response.json().await.unwrap();
    assert_eq!(schema.name, "test-schema");
    assert_eq!(schema.version, "1.0.0");
    let uuid_str = schema.id.to_string();
    assert!(
        Uuid::parse_str(&uuid_str).is_ok(),
        "Schema ID should be a valid UUID"
    );
    assert!(schema.created_at.timestamp() > 0);
}

#[tokio::test]
async fn returns_201_with_location_header() {
    let app = setup_test_app().await;
    let response = post_schema(&app, "location-test").await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let location = response
        .headers()
        .get("Location")
        .expect("Location header should be present");

    assert!(location.to_str().unwrap().contains("/schemas/"));
}

#[tokio::test]
async fn rejects_duplicate_schema_name() {
    let app = setup_test_app().await;
    let _ = post_schema(&app, "duplicate").await;
    let response = post_schema(&app, "duplicate").await;

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let error: ErrorResponse = response.json().await.unwrap();
    assert!(error.message.contains("already exists"));
}

#[tokio::test]
async fn rejects_missing_required_fields() {
    let app = setup_test_app().await;

    let invalid_payload = json!({
        "version": "1.0.0",
    });

    let response = post_schema_with_payload(&app, invalid_payload).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let error_text = response.text().await.unwrap();
    assert!(error_text.contains("missing field") || error_text.contains("name"));
}

