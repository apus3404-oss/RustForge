// src/engine/parallel.rs
use crate::engine::{AgentConfig, AgentEvent, EventBus, ExecutionContext};
use crate::error::Result;
use std::sync::Arc;

/// Parallel executor for running multiple agents concurrently
pub struct ParallelExecutor {
    event_bus: Arc<EventBus>,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    /// Execute multiple agents in parallel
    pub async fn execute(
        &self,
        agents: Vec<AgentConfig>,
        context: &ExecutionContext,
    ) -> Result<Vec<String>> {
        let mut tasks = Vec::new();

        for agent in agents {
            let agent_id = agent.id.clone();
            let task = agent.task.clone();
            let event_bus = self.event_bus.clone();
            let execution_id = context.execution_id.to_string();

            let handle = tokio::spawn(async move {
                // Publish task started event
                let _ = event_bus.publish(AgentEvent::TaskStarted {
                    agent_id: agent_id.clone(),
                    task: task.clone(),
                });

                // Simulate agent execution (minimal implementation)
                let output = format!("Agent {} completed task: {}", agent_id, task);

                // Publish task completed event
                let _ = event_bus.publish(AgentEvent::TaskCompleted {
                    agent_id: agent_id.clone(),
                    output: output.clone(),
                });

                Ok::<String, crate::error::Error>(output)
            });

            tasks.push(handle);
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;

        // Collect results, handling join errors
        let mut outputs = Vec::new();
        for result in results {
            match result {
                Ok(Ok(output)) => outputs.push(output),
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(crate::error::Error::Internal(format!(
                        "Task join error: {}",
                        e
                    )))
                }
            }
        }

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_parallel_execution_with_three_agents() {
        let event_bus = Arc::new(EventBus::new());
        let executor = ParallelExecutor::new(event_bus.clone());

        let agents = vec![
            AgentConfig {
                id: "agent1".to_string(),
                agent_type: "test".to_string(),
                task: "Test task 1".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
            AgentConfig {
                id: "agent2".to_string(),
                agent_type: "test".to_string(),
                task: "Test task 2".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
            AgentConfig {
                id: "agent3".to_string(),
                agent_type: "test".to_string(),
                task: "Test task 3".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
        ];

        let context = ExecutionContext::new("test-workflow".to_string());

        let results = executor.execute(agents, &context).await.unwrap();
        assert_eq!(results.len(), 3);
    }
}
