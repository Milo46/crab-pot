use crate::common::{create_valid_schema, setup_test_app};
use futures_util::{SinkExt, StreamExt};
use log_server::Schema;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn successfully_connects_to_websocket_endpoint() {
    let app = setup_test_app().await;

    let ws_url = app.address.replace("http", "ws");
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
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "ws-connection-test").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let ws_url = app.address.replace("http", "ws");
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
    let app = setup_test_app().await;

    let nonexistent_id = uuid::Uuid::new_v4();

    let ws_url = app.address.replace("http", "ws");
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
    let app = setup_test_app().await;

    let ws_url = app.address.replace("http", "ws");
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
    let app = setup_test_app().await;

    let ws_url = app.address.replace("http", "ws");
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
