use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::Response,
    Json,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::AppState;
use crate::dto::ErrorResponse;
use crate::{
    dto::log_dto::{LogAction, LogActionResponse},
    AppState, LogEvent,
};

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub schema_id: Option<Uuid>,
}

pub async fn ws_handler(
    State(state): State<AppState>,
    Query(query): Query<WebSocketQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    if let Some(schema_id) = query.schema_id {
        match state.schema_service.get_schema_by_id(schema_id).await {
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::new(
                        "SCHEMA_NOT_FOUND",
                        format!("Schema with id '{}' not found", schema_id),
                    )),
                ));
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("INTERNAL_ERROR", e.to_string())),
                ));
            }
            Ok(Some(_)) => {
                tracing::debug!("WebSocket connection requested for schema_id: {}", schema_id);
            }
        }
    } else {
        tracing::debug!("WebSocket connection requested for all schemas");
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, query)))
}

async fn handle_socket(socket: WebSocket, state: AppState, query: WebSocketQuery) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.log_broadcast.subscribe();

    let (tx, mut send_rx) = mpsc::unbounded_channel::<Message>();

    let tx_clone = tx.clone();
    let mut broadcast_task = tokio::spawn(async move {
        while let Ok(log_event) = rx.recv().await {
            let should_send = match &query.schema_id {
                Some(schema_id) => log_event.schema_id() == *schema_id,
                None => true,
            };

            if should_send {
                if let Ok(json) = serde_json::to_string(&log_event) {
                    if tx_clone.send(Message::Text(json.into())).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => match serde_json::from_str::<LogAction>(&text) {
                    Ok(action) => {
                        let response = handle_log_action(action, &state).await;

                        if let Ok(response_json) = serde_json::to_string(&response) {
                            if tx.send(Message::Text(response_json.into())).is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let error_response = LogActionResponse::error(
                            "parse",
                            format!("Invalid message format: {}", e),
                        );
                        if let Ok(response_json) = serde_json::to_string(&error_response) {
                            let _ = tx.send(Message::Text(response_json.into()));
                        }
                    }
                },
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = send_rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut broadcast_task => {
            recv_task.abort();
            send_task.abort();
        },
        _ = &mut recv_task => {
            broadcast_task.abort();
            send_task.abort();
        },
        _ = &mut send_task => {
            broadcast_task.abort();
            recv_task.abort();
        },
    }

    tracing::info!("WebSocket connection closed");
}

async fn handle_log_action(action: LogAction, state: &AppState) -> LogActionResponse {
    match action {
        LogAction::Create {
            schema_id,
            log_data,
        } => match state.log_service.create_log(schema_id, log_data).await {
            Ok(log) => {
                let event = LogEvent::created_from(log.clone());
                let _ = state.log_broadcast.send(event.clone());
                LogActionResponse::success(event)
            }
            Err(e) => LogActionResponse::error("create", e.to_string()),
        },
        LogAction::Delete { id } => match state.log_service.get_log_by_id(id).await {
            Ok(Some(log)) => match state.log_service.delete_log(id).await {
                Ok(_) => {
                    let event = LogEvent::deleted_from(log);
                    let _ = state.log_broadcast.send(event.clone());
                    LogActionResponse::success(event)
                }
                Err(e) => LogActionResponse::error("delete", e.to_string()),
            },
            Ok(None) => LogActionResponse::error("delete", "Log not found"),
            Err(e) => LogActionResponse::error("delete", e.to_string()),
        },
    }
}
