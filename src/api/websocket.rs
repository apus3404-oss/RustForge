use crate::api::{ApiError, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Client command sent via WebSocket
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientCommand {
    Pause,
    Resume,
    Cancel,
}

/// WebSocket handler for execution events
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(execution_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, execution_id, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, execution_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to event bus
    let mut event_rx = state.event_bus.subscribe();

    // Forward events to WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            // TODO: Filter events for this execution_id
            // For now, send all events

            let msg = serde_json::to_string(&event).unwrap_or_default();
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Handle client commands
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                    match cmd {
                        ClientCommand::Pause => {
                            // TODO: Pause execution
                            tracing::info!("Pause command received for execution {}", execution_id);
                        }
                        ClientCommand::Resume => {
                            // TODO: Resume execution
                            tracing::info!("Resume command received for execution {}", execution_id);
                        }
                        ClientCommand::Cancel => {
                            // TODO: Cancel execution
                            tracing::info!("Cancel command received for execution {}", execution_id);
                        }
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_command_deserialization() {
        let pause_json = r#"{"type":"pause"}"#;
        let cmd: ClientCommand = serde_json::from_str(pause_json).unwrap();
        assert!(matches!(cmd, ClientCommand::Pause));

        let resume_json = r#"{"type":"resume"}"#;
        let cmd: ClientCommand = serde_json::from_str(resume_json).unwrap();
        assert!(matches!(cmd, ClientCommand::Resume));

        let cancel_json = r#"{"type":"cancel"}"#;
        let cmd: ClientCommand = serde_json::from_str(cancel_json).unwrap();
        assert!(matches!(cmd, ClientCommand::Cancel));
    }

    #[test]
    fn test_invalid_command() {
        let invalid_json = r#"{"type":"invalid"}"#;
        let result: Result<ClientCommand, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}
