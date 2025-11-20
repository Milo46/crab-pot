use reqwest::StatusCode;
use log_server::{ErrorResponse, Log, Schema};
use serde_json::{json, Value};

use crate::common::{TestContext, valid_schema_payload, valid_log_payload};

#[tokio::test]
async fn retrieves_log_by_id() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("read-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    let log_response = ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema.id))
        .send()
        .await
        .expect("Failed to create log");
    
    let created_log: Log = log_response.json().await.unwrap();

    let response = ctx.client
        .get(&format!("{}/logs/{}", ctx.base_url, created_log.id))
        .send()
        .await
        .expect("Failed to retrieve log");

    assert_eq!(response.status(), StatusCode::OK);
    
    let retrieved_log: Log = response.json().await.unwrap();
    assert_eq!(retrieved_log.id, created_log.id);
    assert_eq!(retrieved_log.schema_id, schema.id);
    assert_eq!(retrieved_log.log_data["message"], "Test log message");
}

#[tokio::test]
async fn returns_404_for_nonexistent_log() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .get(&format!("{}/logs/{}", ctx.base_url, 99999))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}

#[tokio::test]
async fn rejects_invalid_log_id_format() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .get(&format!("{}/logs/invalid", ctx.base_url))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gets_logs_by_schema_name() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("logs-by-name"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    for i in 1..=3 {
        let log_payload = json!({
            "schema_id": schema.id,
            "log_data": {
                "message": format!("Log message {}", i)
            }
        });

        ctx.client
            .post(&format!("{}/logs", ctx.base_url))
            .json(&log_payload)
            .send()
            .await
            .expect("Failed to create log");
    }

    let response = ctx.client
        .get(&format!("{}/logs/schema/{}", ctx.base_url, "logs-by-name"))
        .send()
        .await
        .expect("Failed to get logs");

    assert_eq!(response.status(), StatusCode::OK);
    
    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 3);
}

#[tokio::test]
async fn gets_logs_by_schema_name_and_version() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("logs-by-name-version"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema.id))
        .send()
        .await
        .expect("Failed to create log");

    let response = ctx.client
        .get(&format!("{}/logs/schema/{}/version/{}", ctx.base_url, "logs-by-name-version", "1.0.0"))
        .send()
        .await
        .expect("Failed to get logs");

    assert_eq!(response.status(), StatusCode::OK);
    
    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 1);
}

#[tokio::test]
async fn filters_logs_with_query_parameters() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&json!({
            "name": "filter-test",
            "version": "1.0.0",
            "schema_definition": {
                "type": "object",
                "properties": {
                    "message": { "type": "string" },
                    "level": { "type": "string" }
                },
                "required": [ "message" ]
            }
        }))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    for level in ["INFO", "ERROR", "INFO"] {
        let log_payload = json!({
            "schema_id": schema.id,
            "log_data": {
                "message": format!("{} log message", level),
                "level": level
            }
        });

        ctx.client
            .post(&format!("{}/logs", ctx.base_url))
            .json(&log_payload)
            .send()
            .await
            .expect("Failed to create log");
    }

    let response = ctx.client
        .get(&format!("{}/logs/schema/filter-test?level=ERROR", ctx.base_url))
        .send()
        .await
        .expect("Failed to get filtered logs");

    assert_eq!(response.status(), StatusCode::OK);
    
    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0]["log_data"]["level"], "ERROR");
}

#[tokio::test]
async fn returns_404_for_nonexistent_schema_name() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .get(&format!("{}/logs/schema/nonexistent-schema", ctx.base_url))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}

#[tokio::test]
async fn rejects_empty_schema_name() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .get(&format!("{}/logs/schema/", ctx.base_url))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
