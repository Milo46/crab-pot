use reqwest::StatusCode;
use log_server::{ErrorResponse, Log, Schema};

use crate::common::{TestContext, valid_schema_payload, valid_log_payload};

#[tokio::test]
async fn deletes_existing_log_successfully() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("delete-test"))
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
    
    let log: Log = log_response.json().await.unwrap();

    let delete_response = ctx.client
        .delete(&format!("{}/logs/{}", ctx.base_url, log.id))
        .send()
        .await
        .expect("Failed to delete log");

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    assert!(delete_response.text().await.unwrap().is_empty());
}

#[tokio::test]
async fn returns_404_for_nonexistent_log() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .delete(&format!("{}/logs/{}", ctx.base_url, 99999))
        .send()
        .await
        .expect("Failed to send delete request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}

#[tokio::test]
async fn rejects_invalid_log_id_format() {
    let ctx = TestContext::new().await;

    let response = ctx.client
        .delete(&format!("{}/logs/invalid", ctx.base_url))
        .send()
        .await
        .expect("Failed to send delete request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn log_not_accessible_after_deletion() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("access-after-delete"))
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
    
    let log: Log = log_response.json().await.unwrap();

    let get_response = ctx.client
        .get(&format!("{}/logs/{}", ctx.base_url, log.id))
        .send()
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let delete_response = ctx.client
        .delete(&format!("{}/logs/{}", ctx.base_url, log.id))
        .send()
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_after_delete = ctx.client
        .get(&format!("{}/logs/{}", ctx.base_url, log.id))
        .send()
        .await
        .unwrap();
    assert_eq!(get_after_delete.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn double_delete_returns_404() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("double-delete"))
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
    
    let log: Log = log_response.json().await.unwrap();

    let first_delete = ctx.client
        .delete(&format!("{}/logs/{}", ctx.base_url, log.id))
        .send()
        .await
        .unwrap();
    assert_eq!(first_delete.status(), StatusCode::NO_CONTENT);

    let second_delete = ctx.client
        .delete(&format!("{}/logs/{}", ctx.base_url, log.id))
        .send()
        .await
        .unwrap();
    assert_eq!(second_delete.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = second_delete.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}
