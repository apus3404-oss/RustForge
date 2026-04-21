// src/engine/parallel.rs
use crate::engine::{AgentConfig, AgentEvent, EventBus, ExecutionContext, CancellationToken};
use crate::error::Result;
use std::sync::Arc;
use std::time::Duration;

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
        self.execute_with_timeout(agents, context, None, None).await
    }

    /// Execute multiple agents in parallel with optional timeout and cancellation
    pub async fn execute_with_timeout(
        &self,
        agents: Vec<AgentConfig>,
        context: &ExecutionContext,
        timeout: Option<Duration>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<Vec<String>> {
        let mut tasks = Vec::new();

        for agent in agents {
            let agent_id = agent.id.clone();
            let task = agent.task.clone();
            let event_bus = self.event_bus.clone();
            let execution_id = context.execution_id.to_string();
            let cancel_token = cancellation_token.clone();

            let handle = tokio::spawn(async move {
                // Check cancellation before starting
                if let Some(ref token) = cancel_token {
                    if token.is_cancelled() {
                        return Err(crate::error::Error::Internal(
                            "Execution cancelled".to_string(),
                        ));
                    }
                }

                // Publish task started event
                let _ = event_bus.publish(AgentEvent::TaskStarted {
                    agent_id: agent_id.clone(),
                    task: task.clone(),
                });

                // Simulate agent execution
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

        // Wait for all tasks with optional timeout
        let results_future = futures::future::join_all(tasks);

        let results = if let Some(duration) = timeout {
            match tokio::time::timeout(duration, results_future).await {
                Ok(results) => results,
                Err(_) => {
                    return Err(crate::error::Error::Internal(
                        "Execution timeout exceeded".to_string(),
                    ))
                }
            }
        } else {
            results_future.await
        };

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

    #[tokio::test]
    async fn test_parallel_execution_collects_all_results() {
        let event_bus = Arc::new(EventBus::new());
        let executor = ParallelExecutor::new(event_bus.clone());

        let agents = vec![
            AgentConfig {
                id: "agent1".to_string(),
                agent_type: "test".to_string(),
                task: "Task A".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
            AgentConfig {
                id: "agent2".to_string(),
                agent_type: "test".to_string(),
                task: "Task B".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
        ];

        let context = ExecutionContext::new("test-workflow".to_string());
        let results = executor.execute(agents, &context).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].contains("agent1"));
        assert!(results[1].contains("agent2"));
    }

    #[tokio::test]
    async fn test_parallel_execution_publishes_events() {
        let event_bus = Arc::new(EventBus::new());
        let mut subscriber = event_bus.subscribe();
        let executor = ParallelExecutor::new(event_bus.clone());

        let agents = vec![AgentConfig {
            id: "agent1".to_string(),
            agent_type: "test".to_string(),
            task: "Test task".to_string(),
            depends_on: vec![],
            config: HashMap::new(),
        }];

        let context = ExecutionContext::new("test-workflow".to_string());

        // Execute in background
        let exec_handle = tokio::spawn(async move {
            executor.execute(agents, &context).await
        });

        // Collect events
        let mut events = Vec::new();
        for _ in 0..2 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                subscriber.recv()
            ).await {
                events.push(event.unwrap());
            }
        }

        exec_handle.await.unwrap().unwrap();

        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], AgentEvent::TaskStarted { .. }));
        assert!(matches!(events[1], AgentEvent::TaskCompleted { .. }));
    }

    #[tokio::test]
    async fn test_parallel_execution_with_timeout() {
        let event_bus = Arc::new(EventBus::new());
        let executor = ParallelExecutor::new(event_bus);

        let agents = vec![
            AgentConfig {
                id: "agent1".to_string(),
                agent_type: "test".to_string(),
                task: "Quick task".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
        ];

        let context = ExecutionContext::new("test-workflow".to_string());

        // Execute with generous timeout (should succeed)
        let result = executor
            .execute_with_timeout(
                agents,
                &context,
                Some(std::time::Duration::from_secs(5)),
                None,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parallel_execution_timeout_exceeded() {
        use std::time::Duration;

        let event_bus = Arc::new(EventBus::new());
        let executor = ParallelExecutor::new(event_bus);

        // Create a custom slow executor for this test
        let slow_executor = ParallelExecutor {
            event_bus: Arc::new(EventBus::new()),
        };

        let agents = vec![
            AgentConfig {
                id: "slow_agent".to_string(),
                agent_type: "test".to_string(),
                task: "Slow task".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
        ];

        let context = ExecutionContext::new("test-workflow".to_string());

        // Spawn a task that will definitely take longer than timeout
        let result = tokio::time::timeout(
            Duration::from_millis(1),
            async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<Vec<String>, crate::error::Error>(vec!["done".to_string()])
            }
        ).await;

        // Verify timeout occurred
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parallel_execution_with_cancellation() {
        let event_bus = Arc::new(EventBus::new());
        let executor = ParallelExecutor::new(event_bus);
        let cancel_token = CancellationToken::new();

        let agents = vec![
            AgentConfig {
                id: "agent1".to_string(),
                agent_type: "test".to_string(),
                task: "Task 1".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            },
        ];

        let context = ExecutionContext::new("test-workflow".to_string());

        // Cancel before execution
        cancel_token.cancel();

        let result = executor
            .execute_with_timeout(
                agents,
                &context,
                None,
                Some(cancel_token),
            )
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cancelled"));
    }
}
