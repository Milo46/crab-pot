use crate::common::{valid_schema_payload, TestContext};
use futures_util::{SinkExt, StreamExt};
use log_server::Schema;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn successfully_connects_to_websocket_endpoint() {
    let ctx = TestContext::new().await;

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs", ws_url);

    let result = connect_async(&url).await;
    assert!(
        result.is_ok(),
        "Should successfully connect to WebSocket endpoint"
    );

    let (mut ws_stream, _) = result.unwrap();

    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn successfully_connects_with_valid_schema_id() {
    let ctx = TestContext::new().await;

    let schema_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("ws-connection-test"))
        .send()
        .await
        .expect("Failed to create schema");

    let schema: Schema = schema_response.json().await.unwrap();

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs?schema_id={}", ws_url, schema.id);

    let result = connect_async(&url).await;
    assert!(
        result.is_ok(),
        "Should successfully connect with valid schema_id"
    );

    let (mut ws_stream, _) = result.unwrap();
    ws_stream.close(None).await.unwrap();
}

#[tokio::test]
async fn rejects_connection_with_nonexistent_schema_id() {
    let ctx = TestContext::new().await;

    let nonexistent_id = uuid::Uuid::new_v4();

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs?schema_id={}", ws_url, nonexistent_id);

    let result = connect_async(&url).await;

    assert!(
        result.is_err(),
        "Should reject connection with non-existent schema_id"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("404") || err_msg.contains("Not Found"),
        "Error should indicate 404 Not Found, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn rejects_connection_with_invalid_schema_id_format() {
    let ctx = TestContext::new().await;

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs?schema_id=invalid-uuid", ws_url);

    let result = connect_async(&url).await;

    assert!(
        result.is_err(),
        "Should reject connection with invalid schema_id format"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("400") || err_msg.contains("Bad Request") || err_msg.contains("404"),
        "Error should indicate bad request or not found, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn handles_graceful_disconnection() {
    let ctx = TestContext::new().await;

    let ws_url = ctx.base_url.replace("http", "ws");
    let url = format!("{}/ws/logs", ws_url);

    let (mut ws_stream, _) = connect_async(&url).await.unwrap();

    ws_stream
        .send(Message::Close(None))
        .await
        .expect("Should send close frame");

    while let Some(msg) = ws_stream.next().await {
        if let Ok(Message::Close(_)) = msg {
            break;
        }
    }

    let result = ws_stream.send(Message::Text("test".into())).await;
    assert!(
        result.is_err(),
        "Should not be able to send after closing connection"
    );
}
