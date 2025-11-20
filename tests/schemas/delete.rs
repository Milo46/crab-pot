use reqwest::StatusCode;
use log_server::{ErrorResponse, Schema};
use uuid::Uuid;

use crate::common::{TestContext, valid_schema_payload};

#[tokio::test]
async fn deletes_existing_schema_successfully() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("delete-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    let delete_response = ctx.client
        .delete(&format!("{}/schemas/{}", ctx.base_url, schema.id))
        .send()
        .await
        .expect("Failed to send delete request");

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    assert!(delete_response.text().await.unwrap().is_empty());
}

#[tokio::test]
async fn returns_404_for_nonexistent_schema() {
    let ctx = TestContext::new().await;
    
    let non_existent_id = Uuid::new_v4();
    
    let response = ctx.client
        .delete(&format!("{}/schemas/{}", ctx.base_url, non_existent_id))
        .send()
        .await
        .expect("Failed to send delete request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
    assert!(error.message.contains(&non_existent_id.to_string()));
}

#[tokio::test]
async fn rejects_invalid_uuid_format() {
    let ctx = TestContext::new().await;
    
    let response = ctx.client
        .delete(&format!("{}/schemas/invalid-uuid", ctx.base_url))
        .send()
        .await
        .expect("Failed to send delete request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_nil_uuid() {
    let ctx = TestContext::new().await;
    
    let nil_uuid = Uuid::nil();
    
    let response = ctx.client
        .delete(&format!("{}/schemas/{}", ctx.base_url, nil_uuid))
        .send()
        .await
        .expect("Failed to send delete request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("cannot be empty"));
}

#[tokio::test]
async fn schema_not_accessible_after_deletion() {
    let ctx = TestContext::new().await;
    
    let schema_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("accessible-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let schema: Schema = schema_response.json().await.unwrap();

    let get_response = ctx.client
        .get(&format!("{}/schemas/{}", ctx.base_url, schema.id))
        .send()
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let delete_response = ctx.client
        .delete(&format!("{}/schemas/{}", ctx.base_url, schema.id))
        .send()
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_after_delete_response = ctx.client
        .get(&format!("{}/schemas/{}", ctx.base_url, schema.id))
        .send()
        .await
        .unwrap();
    assert_eq!(get_after_delete_response.status(), StatusCode::NOT_FOUND);
}
