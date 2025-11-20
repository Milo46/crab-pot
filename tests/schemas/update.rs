use reqwest::StatusCode;
use log_server::{ErrorResponse, Schema};
use uuid::Uuid;
use serde_json::json;

use crate::common::{TestContext, valid_schema_payload};

#[tokio::test]
async fn updates_existing_schema_successfully() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "updated-schema-name",
        "version": "2.0.0",
        "description": "Updated description",
        "schema_definition": {
            "type": "object",
            "properties": {
                "updated_field": {
                    "type": "string",
                    "description": "This field was updated"
                }
            },
            "required": ["updated_field"]
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.id, created_schema.id);
    assert_eq!(updated_schema.name, "updated-schema-name");
    assert_eq!(updated_schema.version, "2.0.0");
    assert_eq!(updated_schema.description, Some("Updated description".to_string()));
    assert_eq!(updated_schema.schema_definition["properties"]["updated_field"]["type"], "string");
    
    assert_eq!(updated_schema.created_at, created_schema.created_at);
    assert_ne!(updated_schema.updated_at, created_schema.updated_at);
}

#[tokio::test]
async fn returns_404_for_nonexistent_schema() {
    let ctx = TestContext::new().await;
    
    let nonexistent_id = Uuid::new_v4();
    let update_payload = json!({
        "name": "new-name",
        "version": "1.0.0",
        "description": "New description",
        "schema_definition": {
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            }
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, nonexistent_id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
    assert!(error.message.contains(&nonexistent_id.to_string()));
}

#[tokio::test]
async fn rejects_invalid_uuid_format() {
    let ctx = TestContext::new().await;
    
    let update_payload = json!({
        "name": "new-name",
        "version": "1.0.0",
        "description": "New description",
        "schema_definition": {
            "type": "object"
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/invalid-uuid", ctx.base_url))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_nil_uuid() {
    let ctx = TestContext::new().await;
    
    let nil_uuid = Uuid::nil();
    let update_payload = json!({
        "name": "new-name",
        "version": "1.0.0",
        "description": "New description",
        "schema_definition": {
            "type": "object"
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, nil_uuid))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("Schema ID cannot be empty"));
}

#[tokio::test]
async fn rejects_empty_schema_name() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-empty-name-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "",
        "version": "2.0.0",
        "description": "Updated description",
        "schema_definition": {
            "type": "object"
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("Schema name cannot be empty"));
}

#[tokio::test]
async fn rejects_whitespace_only_schema_name() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-whitespace-name-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "   ",
        "version": "2.0.0",
        "description": "Updated description",
        "schema_definition": {
            "type": "object"
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("Schema name cannot be empty"));
}

#[tokio::test]
async fn rejects_missing_required_fields() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-missing-fields-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "updated-name"
        // Missing: version, schema_definition
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn handles_special_characters_in_updated_name() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-special-chars-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let special_name = "updated-schema_with!special@chars#and$numbers123";
    let update_payload = json!({
        "name": special_name,
        "version": "2.0.0",
        "description": "Updated with special characters",
        "schema_definition": {
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            }
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.name, special_name);
}

#[tokio::test]
async fn allows_optional_description_to_be_none() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-no-description-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "updated-without-description",
        "version": "2.0.0",
        "description": null,
        "schema_definition": {
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            }
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.description, None);
}

#[tokio::test]
async fn rejects_duplicate_name_when_updating() {
    let ctx = TestContext::new().await;
    
    let schema1_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("original-schema"))
        .send()
        .await
        .expect("Failed to create first schema");
    
    let _schema1: Schema = schema1_response.json().await.unwrap();
    
    let schema2_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("schema-to-update"))
        .send()
        .await
        .expect("Failed to create second schema");
    
    let schema2: Schema = schema2_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "original-schema",
        "version": "2.0.0",
        "description": "Trying to use duplicate name",
        "schema_definition": {
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            }
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, schema2.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "DUPLICATE_NAME");
    assert!(error.message.contains("original-schema"));
}

#[tokio::test]
async fn allows_updating_to_same_name() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("same-name-update-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "same-name-update-test",
        "version": "2.0.0",
        "description": "Updated with same name",
        "schema_definition": {
            "type": "object",
            "properties": {
                "new_field": {"type": "string"}
            }
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.name, "same-name-update-test");
    assert_eq!(updated_schema.version, "2.0.0");
}

#[tokio::test]
async fn rejects_name_exceeding_max_length() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-long-name-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let long_name = "a".repeat(101);
    let update_payload = json!({
        "name": long_name,
        "version": "2.0.0",
        "description": "Testing long name",
        "schema_definition": {
            "type": "object"
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("name") && error.message.contains("length"));
}

#[tokio::test]
async fn rejects_invalid_schema_definition() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-invalid-def-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload = json!({
        "name": "updated-invalid-schema",
        "version": "2.0.0",
        "description": "Invalid schema definition",
        "schema_definition": {
            "type": "invalid_type",
            "properties": "this should be an object"
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_SCHEMA");
}

#[tokio::test]
async fn rejects_malformed_json_payload() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-malformed-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .header("content-type", "application/json")
        .body(r#"{"name": "test", "version": "1.0.0", "invalid": json}"#)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_wrong_content_type() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("update-content-type-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .header("content-type", "text/plain")
        .body("not json")
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn handles_concurrent_updates_gracefully() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("concurrent-update-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    
    let update_payload_1 = json!({
        "name": "concurrent-update-1",
        "version": "2.0.0",
        "description": "First concurrent update",
        "schema_definition": {
            "type": "object",
            "properties": {
                "field1": {"type": "string"}
            }
        }
    });
    
    let update_payload_2 = json!({
        "name": "concurrent-update-2",
        "version": "3.0.0", 
        "description": "Second concurrent update",
        "schema_definition": {
            "type": "object",
            "properties": {
                "field2": {"type": "number"}
            }
        }
    });
    
    let (response1, response2) = tokio::join!(
        ctx.client
            .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
            .json(&update_payload_1)
            .send(),
        ctx.client
            .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
            .json(&update_payload_2)
            .send()
    );

    let response1 = response1.expect("Failed to send first update");
    let response2 = response2.expect("Failed to send second update");
    
    // Both should succeed or one should fail with appropriate error
    // The exact behavior depends on implementation (optimistic/pessimistic locking)
    assert!(
        (response1.status() == StatusCode::OK && response2.status() == StatusCode::OK) ||
        (response1.status() == StatusCode::OK && response2.status() == StatusCode::CONFLICT) ||
        (response1.status() == StatusCode::CONFLICT && response2.status() == StatusCode::OK)
    );
}

#[tokio::test]
async fn preserves_id_and_created_at_fields() {
    let ctx = TestContext::new().await;
    
    let create_response = ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("preserve-fields-test"))
        .send()
        .await
        .expect("Failed to create schema");
    
    let created_schema: Schema = create_response.json().await.unwrap();
    let original_id = created_schema.id;
    let original_created_at = created_schema.created_at.clone();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let update_payload = json!({
        "name": "preserve-test-updated",
        "version": "2.0.0",
        "description": "Testing field preservation",
        "schema_definition": {
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            }
        }
    });
    
    let response = ctx.client
        .put(&format!("{}/schemas/{}", ctx.base_url, created_schema.id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to send update request");

    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_schema: Schema = response.json().await.unwrap();
    

    assert_eq!(updated_schema.id, original_id);
    assert_eq!(updated_schema.created_at, original_created_at);
    
    assert_eq!(updated_schema.name, "preserve-test-updated");
    assert_ne!(updated_schema.updated_at, created_schema.updated_at);
}
