use log_server::{Log, Schema};
use reqwest::StatusCode;
use serde_json::Value;

use crate::common::{
    create_valid_log, create_valid_log_with_message, create_valid_schema, get_log,
    get_logs_by_schema_name, get_logs_by_schema_name_and_version, setup_test_app, ErrorResponse,
};

#[tokio::test]
async fn retrieves_log_by_id() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "read-test").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let log_response = create_valid_log(&app, schema.id.to_string()).await;
    let created_log: Log = log_response.json().await.unwrap();

    let response = get_log(&app, created_log.id.to_string()).await;
    assert_eq!(response.status(), StatusCode::OK);

    let retrieved_log: Log = response.json().await.unwrap();
    assert_eq!(retrieved_log.id, created_log.id);
    assert_eq!(retrieved_log.schema_id, schema.id);
    assert_eq!(retrieved_log.log_data["message"], "Test log message");
}

#[tokio::test]
async fn returns_404_for_nonexistent_log() {
    let app = setup_test_app().await;

    let response = get_log(&app, 99999.to_string()).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}

#[tokio::test]
async fn rejects_invalid_log_id_format() {
    let app = setup_test_app().await;

    let response = get_log(&app, "invalid").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gets_logs_by_schema_name() {
    let app = setup_test_app().await;

    let schema_name = "logs-by-name";

    let schema_response = create_valid_schema(&app, schema_name).await;
    let schema: Schema = schema_response.json().await.unwrap();

    for i in 1..=3 {
        let _ = create_valid_log_with_message(
            &app,
            schema.id.to_string(),
            &format!("Log message {}", i),
        )
        .await;
    }

    let response = get_logs_by_schema_name(&app, schema_name).await;
    assert_eq!(response.status(), StatusCode::OK);

    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 3);
}

#[tokio::test]
async fn gets_logs_by_schema_name_and_version() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "logs-by-name-version").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let _ = create_valid_log(&app, schema.id.to_string()).await;

    let response = get_logs_by_schema_name_and_version(&app, "logs-by-name-version", "1.0.0").await;
    assert_eq!(response.status(), StatusCode::OK);

    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 1);
}

// #[tokio::test]
// async fn filters_logs_with_query_parameters() {
//     let app = setup_test_app().await;

//     let schema_response = create_schema(&app, &json!({
//         "name": "filter-test",
//         "version": "1.0.0",
//         "schema_definition": {
//             "type": "object",
//             "properties": {
//                 "message": { "type": "string" },
//                 "level": { "type": "string" }
//             },
//             "required": [ "message" ]
//         }
//     })).await;
//     let schema: Schema = schema_response.json().await.unwrap();

//     for level in ["INFO", "ERROR", "INFO"] {
//         let _ = create_log(&app, &json!({
//             "schema_id": schema.id,
//             "log_data": {
//                 "message": format!("{} log message", level),
//                 "level": level,
//             }
//         })).await;
//     }

//     let response = app
//         .client
//         .get(&format!(
//             "{}/logs/schema/filter-test?level=ERROR",
//             app.address
//         ))
//         .header("X-Api-Key", "secret-key")
//         .send()
//         .await
//         .expect("Failed to get filtered logs");

//     let response =

//     assert_eq!(response.status(), StatusCode::OK);

//     let data: Value = response.json().await.unwrap();
//     let logs = data["logs"].as_array().unwrap();
//     assert_eq!(logs.len(), 1);
//     assert_eq!(logs[0]["log_data"]["level"], "ERROR");
// }

#[tokio::test]
async fn returns_404_for_nonexistent_schema_name() {
    let app = setup_test_app().await;

    let response = get_logs_by_schema_name(&app, "nonexistent-schema").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let error: ErrorResponse = response.json().await.unwrap();
    assert_eq!(error.error, "NOT_FOUND");
}

#[tokio::test]
async fn rejects_empty_schema_name() {
    let app = setup_test_app().await;

    let response = get_logs_by_schema_name(&app, "").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
