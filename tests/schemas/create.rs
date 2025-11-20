use reqwest::StatusCode;
use log_server::{ErrorResponse, Schema};
use uuid::Uuid;
use serde_json::json;

use crate::common::{TestContext, valid_schema_payload, TEST_SCHEMA_NAME, TEST_SCHEMA_VERSION};

#[tokio::test]
async fn creates_schema_with_valid_data() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload(TEST_SCHEMA_NAME))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);

    let schema: Schema = response.json().await.unwrap();
    assert_eq!(schema.name, TEST_SCHEMA_NAME);
    assert_eq!(schema.version, TEST_SCHEMA_VERSION);
    let uuid_str = schema.id.to_string();
    assert!(Uuid::parse_str(&uuid_str).is_ok(), "Schema ID should be a valid UUID");
    assert!(schema.created_at.timestamp() > 0);
}

#[tokio::test]
async fn returns_201_with_location_header() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("location-test"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);

    let location = response.headers()
        .get("Location")
        .expect("Location header should be present");

    assert!(location.to_str().unwrap().contains("/schemas/"));
}

#[tokio::test]
async fn rejects_duplicate_schema_name() {
    let ctx = TestContext::new().await;

    ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("duplicate"))
        .send()
        .await
        .unwrap();

    let response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("duplicate"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let error: ErrorResponse = response.json().await.unwrap();
    assert!(error.message.contains("already exists"));
}

#[tokio::test]
async fn rejects_missing_required_fields() {
    let ctx = TestContext::new().await;

    let invalid_payload = json!({
        "version": "1.0.0",
    });

    let response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&invalid_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    
    let error_text = response.text().await.unwrap();
    assert!(error_text.contains("missing field") || error_text.contains("name"));
}
