use log_server::Schema;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::{valid_schema_payload, ErrorResponse, TestContext};

#[tokio::test]
async fn updates_existing_schema_successfully() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "update-test").await;
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
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.id, created_schema.id);
    assert_eq!(updated_schema.name, "updated-schema-name");
    assert_eq!(updated_schema.version, "2.0.0");
    assert_eq!(
        updated_schema.description,
        Some("Updated description".to_string())
    );
    assert_eq!(
        updated_schema.schema_definition["properties"]["updated_field"]["type"],
        "string"
    );

    assert_eq!(updated_schema.created_at, created_schema.created_at);
    assert_ne!(updated_schema.updated_at, created_schema.updated_at);
}

#[tokio::test]
async fn returns_404_for_nonexistent_schema() {
    let app = setup_test_app().await;

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

    let response = update_schema(&app, &nonexistent_id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
    assert!(error.message.contains(&nonexistent_id.to_string()));
}

#[tokio::test]
async fn rejects_invalid_uuid_format() {
    let app = setup_test_app().await;

    let update_payload = json!({
        "name": "new-name",
        "version": "1.0.0",
        "description": "New description",
        "schema_definition": {
            "type": "object"
        }
    });
    let response = update_schema(&app, "invalid-uuid", &update_payload).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_nil_uuid() {
    let app = setup_test_app().await;

    let nil_uuid = Uuid::nil();
    let update_payload = json!({
        "name": "new-name",
        "version": "1.0.0",
        "description": "New description",
        "schema_definition": {
            "type": "object"
        }
    });

    let response = update_schema(&app, &nil_uuid.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("Schema ID cannot be empty"));
}

#[tokio::test]
async fn rejects_empty_schema_name() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "update-empty-name-test").await;
    let created_schema: Schema = create_response.json().await.unwrap();

    let update_payload = json!({
        "name": "",
        "version": "2.0.0",
        "description": "Updated description",
        "schema_definition": {
            "type": "object"
        }
    });
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("Schema name cannot be empty"));
}

#[tokio::test]
async fn rejects_whitespace_only_schema_name() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "update-whitespace-name-test").await;
    let created_schema: Schema = create_response.json().await.unwrap();

    let update_payload = json!({
        "name": "   ",
        "version": "2.0.0",
        "description": "Updated description",
        "schema_definition": {
            "type": "object"
        }
    });
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("Schema name cannot be empty"));
}

#[tokio::test]
async fn rejects_missing_required_fields() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "update-missing-fields-test").await;
    let created_schema: Schema = create_response.json().await.unwrap();

    let update_payload = json!({
        "name": "updated-name"
        // Missing: version, schema_definition
    });
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn handles_special_characters_in_updated_name() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "update-special-chars-test").await;
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
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.name, special_name);
}

#[tokio::test]
async fn allows_optional_description_to_be_none() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "update-no-description-test").await;
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
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.description, None);
}

#[tokio::test]
async fn rejects_duplicate_name_when_updating() {
    let app = setup_test_app().await;

    let schema1_response = create_valid_schema(&app, "original-schema").await;
    let _schema1: Schema = schema1_response.json().await.unwrap();

    let schema2_response = create_valid_schema(&app, "schema-to-update").await;
    let schema2: Schema = schema2_response.json().await.unwrap();

    let update_payload = json!({
        "name": "original-schema",
        "version": "1.0.0", // This should conflict because original-schema v1.0.0 already exists
        "description": "Trying to use duplicate name and version",
        "schema_definition": {
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            }
        }
    });
    let response = update_schema(&app, &schema2.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "SCHEMA_CONFLICT");
    assert!(error.message.contains("original-schema"));
    assert!(error.message.contains("already exists"));
}

#[tokio::test]
async fn allows_updating_to_same_name() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "same-name-update-test").await;
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
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.name, "same-name-update-test");
    assert_eq!(updated_schema.version, "2.0.0");
}

#[tokio::test]
async fn rejects_invalid_schema_definition() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "update-invalid-def-test").await;
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
    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_SCHEMA");
}

// #[tokio::test]
// async fn rejects_malformed_json_payload() {
//     let app = setup_test_app().await;

//     let create_response = create_valid_schema(&app, "update-malformed-test").await;
//     let created_schema: Schema = create_response.json().await.unwrap();

//     let response = update_schema(
//         &app,
//         &created_schema.id.to_string(),
//         r#"{"name": "test", "version": "1.0.0", "invalid": json}"#,
//     )
//     .await;

//     assert_eq!(response.status(), StatusCode::BAD_REQUEST);
// }

// #[tokio::test]
// async fn rejects_wrong_content_type() {
//     let app = setup_test_app().await;

//     let create_response = create_valid_schema(&app, "update-content-type-test").await;
//     let created_schema: Schema = create_response.json().await.unwrap();

//     let response = update_schema(&app, &created_schema.id.to_string(), "not json").await;

//     let response = app
//         .client
//         .put(&format!("{}/schemas/{}", app.address, created_schema.id))
//         .header("X-Api-Key", "secret-key")
//         .header("content-type", "text/plain")
//         .body("not json")
//         .send()
//         .await
//         .expect("Failed to send update request");

//     assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
// }

#[tokio::test]
async fn handles_concurrent_updates_gracefully() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "concurrent-update-test").await;
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

    let schema_id = created_schema.id.to_string();
    let (response1, response2) = tokio::join!(
        update_schema(&app, &schema_id, &update_payload_1),
        update_schema(&app, &schema_id, &update_payload_2),
    );

    // let response1 = response1.expect("Failed to send first update");
    // let response2 = response2.expect("Failed to send second update");

    // Both should succeed or one should fail with appropriate error
    // The exact behavior depends on implementation (optimistic/pessimistic locking)
    assert!(
        (response1.status() == StatusCode::OK && response2.status() == StatusCode::OK)
            || (response1.status() == StatusCode::OK && response2.status() == StatusCode::CONFLICT)
            || (response1.status() == StatusCode::CONFLICT && response2.status() == StatusCode::OK)
    );
}

#[tokio::test]
async fn preserves_id_and_created_at_fields() {
    let app = setup_test_app().await;

    let create_response = create_valid_schema(&app, "preserve-fields-test").await;
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

    let response = update_schema(&app, &created_schema.id.to_string(), &update_payload).await;
    assert_eq!(response.status(), StatusCode::OK);

    let updated_schema: Schema = response.json().await.unwrap();
    assert_eq!(updated_schema.id, original_id);
    assert_eq!(updated_schema.created_at, original_created_at);

    assert_eq!(updated_schema.name, "preserve-test-updated");
    assert_ne!(updated_schema.updated_at, created_schema.updated_at);
}
