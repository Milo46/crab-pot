use crate::common::{create_log, create_valid_log, create_valid_schema, setup_test_app};
use futures_util::StreamExt;
use log_server::{Log, LogEvent, Schema};
use serde_json::json;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn receives_created_event_when_log_is_created() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "ws-create-event-test").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let ws_url = app.address.replace("http", "ws");
    let url = format!("{}/ws/logs", ws_url);
    let (mut ws_stream, _) = connect_async(&url).await.unwrap();

    let log_response = create_valid_log(&app, &schema.id.to_string()).await;
    let created_log: Log = log_response.json().await.unwrap();

    let ws_message = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for WebSocket message")
        .expect("WebSocket stream ended")
        .expect("Failed to receive message");

    if let Message::Text(text) = ws_message {
        let event: LogEvent = serde_json::from_str(&text).expect("Failed to parse LogEvent");

        match event {
            LogEvent::Created {
                id,
                schema_id,
                log_data,
                ..
            } => {
                assert_eq!(id, created_log.id);
                assert_eq!(schema_id, schema.id);
                assert_eq!(log_data["message"], "Test log message");
            }
            _ => panic!("Expected Created event, got Deleted"),
        }
    } else {
        panic!("Expected text message, got: {:?}", ws_message);
    }

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn receives_deleted_event_when_log_is_deleted() {
    let ctx = TestContext::new().await;

    let schema_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-delete-event-test"))
        .send()
        .await
        .expect("Failed to create schema");

    let schema: Schema = schema_response.json().await.unwrap();

    let log_response = ctx
        .client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema.id))
        .send()
        .await
        .expect("Failed to create log");

    let created_log: Log = log_response.json().await.unwrap();

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs", ws_url);
    let (mut ws_stream, _) = connect_async(&url).await.unwrap();

    ctx.client
        .delete(&format!("{}/logs/{}", ctx.base_url, created_log.id))
        .send()
        .await
        .expect("Failed to delete log");

    let ws_message = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for WebSocket message")
        .expect("WebSocket stream ended")
        .expect("Failed to receive message");

    if let Message::Text(text) = ws_message {
        let event: LogEvent = serde_json::from_str(&text).expect("Failed to parse LogEvent");

        match event {
            LogEvent::Deleted { id, schema_id } => {
                assert_eq!(id, created_log.id);
                assert_eq!(schema_id, schema.id);
            }
            _ => panic!("Expected Deleted event, got Created"),
        }
    } else {
        panic!("Expected text message, got: {:?}", ws_message);
    }

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn filters_events_by_schema_id() {
    let ctx = TestContext::new().await;

    let schema1_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-filter-test-1"))
        .send()
        .await
        .expect("Failed to create schema 1");

    let schema1: Schema = schema1_response.json().await.unwrap();

    let schema2_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-filter-test-2"))
        .send()
        .await
        .expect("Failed to create schema 2");

    let schema2: Schema = schema2_response.json().await.unwrap();

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs?schema_id={}", ws_url, schema1.id);
    let (mut ws_stream, _) = connect_async(&url).await.unwrap();

    ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema2.id))
        .send()
        .await
        .expect("Failed to create log for schema 2");

    let result = timeout(Duration::from_secs(2), ws_stream.next()).await;
    assert!(
        result.is_err(),
        "Should not receive event from different schema"
    );

    let log_response = ctx
        .client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema1.id))
        .send()
        .await
        .expect("Failed to create log for schema 1");

    let created_log: Log = log_response.json().await.unwrap();

    let ws_message = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for WebSocket message")
        .expect("WebSocket stream ended")
        .expect("Failed to receive message");

    if let Message::Text(text) = ws_message {
        let event: LogEvent = serde_json::from_str(&text).expect("Failed to parse LogEvent");

        match event {
            LogEvent::Created { id, schema_id, .. } => {
                assert_eq!(id, created_log.id);
                assert_eq!(schema_id, schema1.id);
            }
            _ => panic!("Expected Created event"),
        }
    }

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn receives_all_events_without_schema_filter() {
    let ctx = TestContext::new().await;

    let schema1_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-no-filter-test-1"))
        .send()
        .await
        .expect("Failed to create schema 1");

    let schema1: Schema = schema1_response.json().await.unwrap();

    let schema2_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-no-filter-test-2"))
        .send()
        .await
        .expect("Failed to create schema 2");

    let schema2: Schema = schema2_response.json().await.unwrap();

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs", ws_url);
    let (mut ws_stream, _) = connect_async(&url).await.unwrap();

    ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema1.id))
        .send()
        .await
        .expect("Failed to create log for schema 1");

    ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema2.id))
        .send()
        .await
        .expect("Failed to create log for schema 2");

    let mut received_schemas = vec![];

    for _ in 0..2 {
        let ws_message = timeout(Duration::from_secs(5), ws_stream.next())
            .await
            .expect("Timeout waiting for WebSocket message")
            .expect("WebSocket stream ended")
            .expect("Failed to receive message");

        if let Message::Text(text) = ws_message {
            let event: LogEvent = serde_json::from_str(&text).expect("Failed to parse LogEvent");

            match event {
                LogEvent::Created { schema_id, .. } => {
                    received_schemas.push(schema_id);
                }
                _ => panic!("Expected Created event"),
            }
        }
    }

    assert!(
        received_schemas.contains(&schema1.id),
        "Should receive event from schema 1"
    );
    assert!(
        received_schemas.contains(&schema2.id),
        "Should receive event from schema 2"
    );

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn multiple_clients_receive_same_events() {
    let ctx = TestContext::new().await;

    let schema_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-multi-client-test"))
        .send()
        .await
        .expect("Failed to create schema");

    let schema: Schema = schema_response.json().await.unwrap();

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs", ws_url);

    let (ws_stream1, _) = connect_async(&url).await.unwrap();
    let (ws_stream2, _) = connect_async(&url).await.unwrap();
    let (ws_stream3, _) = connect_async(&url).await.unwrap();

    let log_response = ctx
        .client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&valid_log_payload(schema.id))
        .send()
        .await
        .expect("Failed to create log");

    let created_log: Log = log_response.json().await.unwrap();

    let mut clients = vec![ws_stream1, ws_stream2, ws_stream3];

    for (i, ws_stream) in clients.iter_mut().enumerate() {
        let ws_message = timeout(Duration::from_secs(5), ws_stream.next())
            .await
            .expect(&format!("Timeout for client {}", i + 1))
            .expect("WebSocket stream ended")
            .expect("Failed to receive message");

        if let Message::Text(text) = ws_message {
            let event: LogEvent = serde_json::from_str(&text).expect("Failed to parse LogEvent");

            match event {
                LogEvent::Created { id, schema_id, .. } => {
                    assert_eq!(id, created_log.id);
                    assert_eq!(schema_id, schema.id);
                }
                _ => panic!("Expected Created event for client {}", i + 1),
            }
        }
    }

    for ws_stream in clients.iter_mut() {
        ws_stream.close(None).await.unwrap();
    }
}

#[tokio::test]
async fn event_contains_correct_data_structure() {
    let ctx = TestContext::new().await;

    let schema_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-data-structure-test"))
        .send()
        .await
        .expect("Failed to create schema");

    let schema: Schema = schema_response.json().await.unwrap();

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs", ws_url);
    let (mut ws_stream, _) = connect_async(&url).await.unwrap();

    let custom_log_data = json!({
        "schema_id": schema.id,
        "log_data": {
            "message": "Custom test message",
            "level": "INFO",
            "timestamp": "2025-12-05T12:00:00Z"
        }
    });

    ctx.client
        .post(&format!("{}/logs", ctx.base_url))
        .json(&custom_log_data)
        .send()
        .await
        .expect("Failed to create log");

    let ws_message = timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("Timeout waiting for WebSocket message")
        .expect("WebSocket stream ended")
        .expect("Failed to receive message");

    if let Message::Text(text) = ws_message {
        let json_value: serde_json::Value =
            serde_json::from_str(&text).expect("Failed to parse JSON");

        assert_eq!(json_value["event_type"], "created");
        assert!(json_value["id"].is_number());
        assert_eq!(json_value["schema_id"], schema.id.to_string());
        assert!(json_value["created_at"].is_string());
        assert!(json_value["log_data"].is_object());
        assert_eq!(json_value["log_data"]["message"], "Custom test message");
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.unwrap();
}
