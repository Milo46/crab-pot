use reqwest::StatusCode;
use log_server::{ErrorResponse, Log, Schema};
use uuid::Uuid;
use serde_json::json;

use crate::common::{TestContext, valid_schema_payload, valid_log_payload};

#[tokio::test]
async fn creates_log_with_valid_data() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("log-create-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    let response = ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema.id))
        .send()
        .await
        .expect("Failed to send create log request");

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let log: Log = response.json().await.unwrap();
    assert_eq!(log.schema_id, schema.id);
    assert_eq!(log.log_data["message"], "Test log message");
    assert!(log.id > 0);
    assert!(log.created_at.timestamp() > 0);
}

#[tokio::test]
async fn rejects_nonexistent_schema_id() {
    let ctx = TestContext::new().await;
    
    let nonexistent_id = Uuid::new_v4();
    let log_payload = json!({
        "schema_id": nonexistent_id,
        "log_data": {
            "message": "Test message"
        }
    });

    let response = ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&log_payload)
        .send()
        .await
        .expect("Failed to send create log request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
    assert!(error.message.contains("Schema"));
}

#[tokio::test]
async fn rejects_nil_schema_id() {
    let ctx = TestContext::new().await;
    
    let log_payload = json!({
        "schema_id": Uuid::nil(),
        "log_data": {
            "message": "Test message"
        }
    });

    let response = ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&log_payload)
        .send()
        .await
        .expect("Failed to send create log request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
}

#[tokio::test]
async fn rejects_missing_required_fields() {
    let ctx = TestContext::new().await;
    
    let invalid_payload = json!({
        "log_data": {
            "message": "Test message"
        }
    });

    let response = ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&invalid_payload)
        .send()
        .await
        .expect("Failed to send create log request");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn validates_log_data_against_schema() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("validation-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    let invalid_log_payload = json!({
        "schema_id": schema.id,
        "log_data": {
            "other_field": "value"
        }
    });

    let response = ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&invalid_log_payload)
        .send()
        .await
        .expect("Failed to send create log request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "VALIDATION_FAILED");
}

#[tokio::test]
async fn accepts_additional_properties() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("additional-props-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    let log_payload = json!({
        "schema_id": schema.id,
        "log_data": {
            "message": "Required field",
            "timestamp": "2023-01-01T00:00:00Z",
            "level": "INFO",
            "extra_data": {
                "nested": "value"
            }
        }
    });

    let response = ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&log_payload)
        .send()
        .await
        .expect("Failed to send create log request");

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let log: Log = response.json().await.unwrap();
    assert_eq!(log.log_data["message"], "Required field");
    assert_eq!(log.log_data["level"], "INFO");
    assert_eq!(log.log_data["extra_data"]["nested"], "value");
}
