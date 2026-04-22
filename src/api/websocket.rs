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
            // Note: Event filtering by execution_id would require adding
            // execution_id to AgentEvent. For now, clients receive all events
            // and can filter client-side based on their execution_id.

            let msg = serde_json::to_string(&event).unwrap_or_default();
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone state for recv_task
    let state_clone = state.clone();

    // Handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Handle client commands
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                    match cmd {
                        ClientCommand::Pause => {
                            tracing::info!("Pause command received for execution {}", execution_id);

                            // Load and update execution status
                            if let Ok(Some(mut stored)) = state_clone.state_store.get_execution(&execution_id.to_string()) {
                                if stored.status == crate::storage::StoredExecutionStatus::Running {
                                    stored.status = crate::storage::StoredExecutionStatus::Paused;
                                    stored.updated_at = chrono::Utc::now().timestamp() as u64;
                                    let _ = state_clone.state_store.save_execution(&stored);
                                    tracing::info!("Execution {} paused", execution_id);
                                } else {
                                    tracing::warn!("Cannot pause execution {} - not running", execution_id);
                                }
                            }
                        }
                        ClientCommand::Resume => {
                            tracing::info!("Resume command received for execution {}", execution_id);

                            // Load and update execution status
                            if let Ok(Some(mut stored)) = state_clone.state_store.get_execution(&execution_id.to_string()) {
                                if stored.status == crate::storage::StoredExecutionStatus::Paused {
                                    stored.status = crate::storage::StoredExecutionStatus::Running;
                                    stored.updated_at = chrono::Utc::now().timestamp() as u64;
                                    let _ = state_clone.state_store.save_execution(&stored);
                                    tracing::info!("Execution {} resumed", execution_id);
                                } else {
                                    tracing::warn!("Cannot resume execution {} - not paused", execution_id);
                                }
                            }
                        }
                        ClientCommand::Cancel => {
                            tracing::info!("Cancel command received for execution {}", execution_id);

                            // Cancel via execution registry
                            if state_clone.execution_registry.exists(&execution_id).await {
                                state_clone.execution_registry.cancel(&execution_id).await;
                                tracing::info!("Execution {} cancelled via registry", execution_id);
                            }

                            // Update status in state store
                            if let Ok(Some(mut stored)) = state_clone.state_store.get_execution(&execution_id.to_string()) {
                                stored.status = crate::storage::StoredExecutionStatus::Cancelled;
                                stored.updated_at = chrono::Utc::now().timestamp() as u64;
                                let _ = state_clone.state_store.save_execution(&stored);
                                tracing::info!("Execution {} status updated to cancelled", execution_id);
                            }
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
