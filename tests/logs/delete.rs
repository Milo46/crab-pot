use log_server::{Log, Schema};
use reqwest::StatusCode;

use crate::common::{
    create_valid_log, create_valid_schema, delete_log, get_log, setup_test_app, ErrorResponse,
};

#[tokio::test]
async fn deletes_existing_log_successfully() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "delete-test").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let log_response = create_valid_log(&app, schema.id.to_string()).await;
    let log: Log = log_response.json().await.unwrap();

    let delete_response = delete_log(&app, log.id.to_string()).await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    assert!(delete_response.text().await.unwrap().is_empty());
}

#[tokio::test]
async fn returns_404_for_nonexistent_log() {
    let app = setup_test_app().await;

    let response = delete_log(&app, 99999.to_string()).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}

#[tokio::test]
async fn rejects_invalid_log_id_format() {
    let app = setup_test_app().await;

    let response = delete_log(&app, "invalid").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn log_not_accessible_after_deletion() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "access-after-delete").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let log_response = create_valid_log(&app, schema.id.to_string()).await;
    let log: Log = log_response.json().await.unwrap();

    let get_response = get_log(&app, log.id.to_string()).await;
    assert_eq!(get_response.status(), StatusCode::OK);

    let delete_response = delete_log(&app, log.id.to_string()).await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_after_delete = get_log(&app, log.id.to_string()).await;
    assert_eq!(get_after_delete.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn double_delete_returns_404() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "double-delete").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let log_response = create_valid_log(&app, schema.id.to_string()).await;
    let log: Log = log_response.json().await.unwrap();

    let first_delete = delete_log(&app, log.id.to_string()).await;
    assert_eq!(first_delete.status(), StatusCode::NO_CONTENT);

    let second_delete = delete_log(&app, log.id.to_string()).await;
    assert_eq!(second_delete.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = second_delete.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}
