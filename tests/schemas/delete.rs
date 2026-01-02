use log_server::SchemaResponse;
use reqwest::StatusCode;
use uuid::Uuid;

use crate::common::{ErrorResponse, TestContext};

#[tokio::test]
async fn deletes_existing_schema_successfully() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "delete-test").await;
    let schema: SchemaResponse = schema_response.json().await.unwrap();

    let delete_response = delete_schema(&app, &schema.id.to_string()).await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    assert!(delete_response.text().await.unwrap().is_empty());
}

#[tokio::test]
async fn returns_404_for_nonexistent_schema() {
    let app = setup_test_app().await;

    let non_existent_id = Uuid::new_v4();
    let response = delete_schema(&app, &non_existent_id.to_string()).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
    assert!(error.message.contains(&non_existent_id.to_string()));
}

#[tokio::test]
async fn rejects_invalid_uuid_format() {
    let app = setup_test_app().await;

    let response = delete_schema(&app, "invalid-uuid").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_nil_uuid() {
    let app = setup_test_app().await;

    let nil_uuid = Uuid::nil();
    let response = delete_schema(&app, &nil_uuid.to_string()).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "INVALID_INPUT");
    assert!(error.message.contains("cannot be empty"));
}

#[tokio::test]
async fn schema_not_accessible_after_deletion() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "accessible-test").await;
    let schema: SchemaResponse = schema_response.json().await.unwrap();

    let get_response = get_schema_by_id(&app, &schema.id.to_string()).await;
    assert_eq!(get_response.status(), StatusCode::OK);

    let delete_response = delete_schema(&app, &schema.id.to_string()).await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_after_delete_response = get_schema_by_id(&app, &schema.id.to_string()).await;
    assert_eq!(get_after_delete_response.status(), StatusCode::NOT_FOUND);
}
