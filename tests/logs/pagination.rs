use log_server::{Log, Schema};
use reqwest::StatusCode;
use serde_json::Value;

use crate::common::{
    create_valid_log_with_message, create_valid_schema, get_logs_with_cursor, setup_test_app,
};

#[tokio::test]
async fn forward_pagination_fetches_older_logs() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "pagination-forward").await;
    let schema: Schema = schema_response.json().await.unwrap();

    for i in 1..=5 {
        create_valid_log_with_message(&app, schema.id.to_string(), &format!("message-{}", i)).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response = get_logs_with_cursor(&app, &schema.id.to_string(), None, 3, "forward").await;

    let status = response.status();
    if status != StatusCode::OK {
        let error_text = response.text().await.unwrap();
        panic!("Expected 200 OK, got {}: {}", status, error_text);
    }
    assert_eq!(status, StatusCode::OK);

    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 3);
    assert_eq!(data["cursor"]["limit"], 3);
    assert_eq!(data["cursor"]["has_more"], true);

    let first_page_messages: Vec<&str> = logs
        .iter()
        .filter_map(|l| l["log_data"]["message"].as_str())
        .collect();
    assert_eq!(
        first_page_messages,
        vec!["message-5", "message-4", "message-3"]
    );

    let next_cursor = data["cursor"]["next_cursor"].as_i64().unwrap();
    let prev_cursor = data["cursor"]["prev_cursor"].as_i64();
    assert!(prev_cursor.is_some());

    let response2 = get_logs_with_cursor(
        &app,
        &schema.id.to_string(),
        Some(next_cursor as i32),
        3,
        "forward",
    )
    .await;
    assert_eq!(response2.status(), StatusCode::OK);

    let data2: Value = response2.json().await.unwrap();
    let logs2 = data2["logs"].as_array().unwrap();
    assert_eq!(logs2.len(), 2);
    assert_eq!(data2["cursor"]["has_more"], false);

    let second_page_messages: Vec<&str> = logs2
        .iter()
        .filter_map(|l| l["log_data"]["message"].as_str())
        .collect();
    assert_eq!(second_page_messages, vec!["message-2", "message-1"]);
}

#[tokio::test]
async fn backward_pagination_fetches_newer_logs() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "pagination-backward").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let mut log_ids = Vec::new();
    for i in 1..=5 {
        let log_response =
            create_valid_log_with_message(&app, schema.id.to_string(), &format!("message-{}", i))
                .await;
        let log: Log = log_response.json().await.unwrap();
        log_ids.push(log.id);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let cursor = log_ids[1];
    let response =
        get_logs_with_cursor(&app, &schema.id.to_string(), Some(cursor), 3, "backward").await;
    assert_eq!(response.status(), StatusCode::OK);

    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 3);

    let messages: Vec<&str> = logs
        .iter()
        .filter_map(|l| l["log_data"]["message"].as_str())
        .collect();
    assert_eq!(messages, vec!["message-5", "message-4", "message-3"]);

    let has_more = data["cursor"]["has_more"].as_bool().unwrap();
    assert_eq!(has_more, false);
}

#[tokio::test]
async fn prev_cursor_navigates_to_newer_logs_in_forward_mode() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "pagination-prev").await;
    let schema: Schema = schema_response.json().await.unwrap();

    for i in 1..=7 {
        create_valid_log_with_message(&app, schema.id.to_string(), &format!("message-{}", i)).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response1 = get_logs_with_cursor(&app, &schema.id.to_string(), None, 3, "forward").await;
    let data1: Value = response1.json().await.unwrap();
    let next_cursor1 = data1["cursor"]["next_cursor"].as_i64().unwrap();

    let response2 = get_logs_with_cursor(
        &app,
        &schema.id.to_string(),
        Some(next_cursor1 as i32),
        3,
        "forward",
    )
    .await;
    let data2: Value = response2.json().await.unwrap();
    let logs2 = data2["logs"].as_array().unwrap();
    let messages2: Vec<&str> = logs2
        .iter()
        .filter_map(|l| l["log_data"]["message"].as_str())
        .collect();
    assert_eq!(messages2, vec!["message-4", "message-3", "message-2"]);

    let prev_cursor2 = data2["cursor"]["prev_cursor"].as_i64().unwrap();

    let response3 = get_logs_with_cursor(
        &app,
        &schema.id.to_string(),
        Some(prev_cursor2 as i32),
        3,
        "backward",
    )
    .await;
    let data3: Value = response3.json().await.unwrap();
    let logs3 = data3["logs"].as_array().unwrap();
    let messages3: Vec<&str> = logs3
        .iter()
        .filter_map(|l| l["log_data"]["message"].as_str())
        .collect();
    assert_eq!(messages3, vec!["message-7", "message-6", "message-5"]);
}

#[tokio::test]
async fn empty_result_with_cursor() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "pagination-empty").await;
    let schema: Schema = schema_response.json().await.unwrap();

    for i in 1..=2 {
        create_valid_log_with_message(&app, schema.id.to_string(), &format!("message-{}", i)).await;
    }

    let response1 = get_logs_with_cursor(&app, &schema.id.to_string(), None, 5, "forward").await;
    let data1: Value = response1.json().await.unwrap();
    let logs1 = data1["logs"].as_array().unwrap();
    assert_eq!(logs1.len(), 2);
    assert_eq!(data1["cursor"]["has_more"], false);

    let next_cursor = data1["cursor"]["next_cursor"].as_i64();
    assert!(next_cursor.is_none());
}

#[tokio::test]
async fn pagination_respects_filters() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "pagination-filters").await;
    let schema: Schema = schema_response.json().await.unwrap();

    for i in 1..=5 {
        let message = if i % 2 == 0 {
            format!("even-{}", i)
        } else {
            format!("odd-{}", i)
        };
        create_valid_log_with_message(&app, schema.id.to_string(), &message).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response = app
        .auth()
        .get(format!("/logs/schemas/{}", schema.id))
        .query(&[("limit", "10"), ("direction", "forward")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 5);
}

#[tokio::test]
async fn default_direction_is_forward() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "pagination-default").await;
    let schema: Schema = schema_response.json().await.unwrap();

    for i in 1..=3 {
        create_valid_log_with_message(&app, schema.id.to_string(), &format!("message-{}", i)).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response = app
        .auth()
        .get(format!("/logs/schemas/{}", schema.id))
        .query(&[("limit", "2")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let data: Value = response.json().await.unwrap();
    let logs = data["logs"].as_array().unwrap();
    assert_eq!(logs.len(), 2);

    let messages: Vec<&str> = logs
        .iter()
        .filter_map(|l| l["log_data"]["message"].as_str())
        .collect();
    assert_eq!(messages, vec!["message-3", "message-2"]);
}
