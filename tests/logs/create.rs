use crab_pot::{Log, Schema};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::{
    create_log, create_valid_log, create_valid_schema, setup_test_app, ErrorResponse,
};

#[tokio::test]
async fn creates_log_with_valid_data() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "log-create-test").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let response = create_valid_log(&app, schema.id.to_string()).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let log: Log = response.json().await.unwrap();
    assert_eq!(log.schema_id, schema.id);
    assert_eq!(log.log_data["message"], "Test log message");
    assert!(log.id > 0);
    assert!(log.created_at.timestamp() > 0);
}

#[tokio::test]
async fn rejects_nonexistent_schema_id() {
    let app = setup_test_app().await;

    let nonexistent_id = Uuid::new_v4();
    let log_payload = json!({
        "schema_id": nonexistent_id,
        "log_data": {
            "message": "Test message"
        }
    });

    let response = create_log(&app, &log_payload).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
    // The error message comes from the foreign key constraint violation
    assert!(error.message.contains("schema") || error.message.contains("not found"));
}

#[tokio::test]
async fn rejects_nil_schema_id() {
    let app = setup_test_app().await;

    let log_payload = json!({
        "schema_id": Uuid::nil(),
        "log_data": {
            "message": "Test message"
        }
    });

    let response = create_log(&app, &log_payload).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "BAD_REQUEST");
}

#[tokio::test]
async fn rejects_missing_required_fields() {
    let app = setup_test_app().await;

    let invalid_payload = json!({
        "log_data": {
            "message": "Test message"
        }
    });

    let response = create_log(&app, &invalid_payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// #[tokio::test]
// async fn validates_log_data_against_schema() {
//     let app = setup_test_app().await;

//     let schema_response = create_valid_schema(&app, "validation-test").await;
//     let schema: Schema = schema_response.json().await.unwrap();

//     let invalid_log_payload = json!({
//         "schema_id": schema.id,
//         "log_data": {
//             "other_field": "value"
//         }
//     });

//     let response = create_log(&app, &invalid_log_payload).await;
//     assert_eq!(response.status(), StatusCode::BAD_REQUEST);

//     let error: ErrorResponse = response.json().await.unwrap();
//     assert_eq!(error.error, "VALIDATION_FAILED");
// }

#[tokio::test]
async fn accepts_additional_properties() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "additional-props-test").await;
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

    let response = create_log(&app, &log_payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let log: Log = response.json().await.unwrap();
    assert_eq!(log.log_data["message"], "Required field");
    assert_eq!(log.log_data["level"], "INFO");
    assert_eq!(log.log_data["extra_data"]["nested"], "value");
}
