// src/engine/events.rs
use serde::Serialize;
use tokio::sync::broadcast;

/// Events emitted during agent task execution
#[derive(Debug, Clone, Serialize)]
pub enum AgentEvent {
    /// Agent task has started
    TaskStarted { agent_id: String, task: String },
    /// Agent task completed successfully
    TaskCompleted { agent_id: String, output: String },
    /// Agent task failed with an error
    TaskFailed { agent_id: String, error: String },
}

/// Event bus for publishing and subscribing to agent events
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    /// Create a new event bus with a default channel capacity of 100
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new event bus with a specified channel capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: AgentEvent) -> Result<(), broadcast::error::SendError<AgentEvent>> {
        self.sender.send(event).map(|_| ())
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let event_bus = EventBus::new();
        let mut subscriber = event_bus.subscribe();

        // Publish an event
        event_bus
            .publish(AgentEvent::TaskStarted {
                agent_id: "agent1".to_string(),
                task: "Test task".to_string(),
            })
            .unwrap();

        // Subscriber should receive the event
        let received = subscriber.recv().await.unwrap();
        match received {
            AgentEvent::TaskStarted { agent_id, task } => {
                assert_eq!(agent_id, "agent1");
                assert_eq!(task, "Test task");
            }
            _ => panic!("Expected TaskStarted event"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let event_bus = EventBus::new();
        let mut sub1 = event_bus.subscribe();
        let mut sub2 = event_bus.subscribe();

        // Publish an event
        event_bus
            .publish(AgentEvent::TaskCompleted {
                agent_id: "agent2".to_string(),
                output: "Success".to_string(),
            })
            .unwrap();

        // Both subscribers should receive the event
        let received1 = sub1.recv().await.unwrap();
        let received2 = sub2.recv().await.unwrap();

        match (received1, received2) {
            (
                AgentEvent::TaskCompleted { agent_id: id1, .. },
                AgentEvent::TaskCompleted { agent_id: id2, .. },
            ) => {
                assert_eq!(id1, "agent2");
                assert_eq!(id2, "agent2");
            }
            _ => panic!("Expected TaskCompleted events"),
        }
    }

    #[tokio::test]
    async fn test_task_failed_event() {
        let event_bus = EventBus::new();
        let mut subscriber = event_bus.subscribe();

        event_bus
            .publish(AgentEvent::TaskFailed {
                agent_id: "agent3".to_string(),
                error: "Connection timeout".to_string(),
            })
            .unwrap();

        let received = subscriber.recv().await.unwrap();
        match received {
            AgentEvent::TaskFailed { agent_id, error } => {
                assert_eq!(agent_id, "agent3");
                assert_eq!(error, "Connection timeout");
            }
            _ => panic!("Expected TaskFailed event"),
        }
    }
}
