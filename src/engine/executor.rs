// src/engine/executor.rs
use crate::engine::types::{ExecutionContext, WorkflowDefinition};
use crate::engine::events::{EventBus, AgentEvent};
use crate::engine::interpolation::VariableInterpolator;
use crate::error::Result;
use std::sync::Arc;
use tracing::{info, debug};

/// Sequential executor for workflows
/// This is a stub implementation that logs and publishes events
/// Real agent execution will be implemented in Phase 2
pub struct SequentialExecutor {
    event_bus: Arc<EventBus>,
}

impl SequentialExecutor {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    /// Execute a workflow sequentially
    /// Returns the final output as JSON
    pub async fn execute(
        &self,
        workflow: &WorkflowDefinition,
        context: &mut ExecutionContext,
    ) -> Result<serde_json::Value> {
        info!("Starting sequential execution of workflow: {}", workflow.name);

        let mut final_output = serde_json::json!({
            "workflow": workflow.name,
            "execution_id": context.execution_id.to_string(),
            "agents": []
        });

        for agent in &workflow.agents {
            debug!("Executing agent: {}", agent.id);

            // Interpolate task variables
            let interpolator = VariableInterpolator::new(context);
            let interpolated_task = interpolator.interpolate(&agent.task)?;

            // Publish TaskStarted event
            self.event_bus
                .publish(AgentEvent::TaskStarted {
                    agent_id: agent.id.clone(),
                    task: interpolated_task.clone(),
                })
                .ok(); // Ignore send errors if no subscribers

            // Stub: Simulate agent execution
            // In Phase 2, this will call real agent implementations
            info!("Agent {} executing task: {}", agent.id, interpolated_task);

            // Create mock output for stub
            let agent_output = serde_json::json!({
                "agent_id": agent.id,
                "status": "completed",
                "task": interpolated_task,
                "result": format!("Stub output from {}", agent.id)
            });

            // Store agent output in context for next agents to use
            let output_key = format!("{}.output", agent.id);
            context.set_value(&output_key, agent_output["result"].clone());

            // Publish TaskCompleted event
            self.event_bus
                .publish(AgentEvent::TaskCompleted {
                    agent_id: agent.id.clone(),
                    output: agent_output["result"].to_string(),
                })
                .ok(); // Ignore send errors if no subscribers

            // Add to final output
            if let Some(agents_array) = final_output["agents"].as_array_mut() {
                agents_array.push(agent_output);
            }

            debug!("Agent {} completed successfully", agent.id);
        }

        info!("Sequential execution completed successfully");
        Ok(final_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::AgentConfig;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_sequential_execution_with_single_agent() {
        let event_bus = Arc::new(EventBus::new());
        let executor = SequentialExecutor::new(event_bus.clone());

        let workflow = WorkflowDefinition {
            name: "Test Workflow".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![AgentConfig {
                id: "agent1".to_string(),
                agent_type: "TestAgent".to_string(),
                task: "Do something".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            }],
            inputs: None,
        };

        let mut context = ExecutionContext::new("test-workflow".to_string());
        let result = executor.execute(&workflow, &mut context).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_object());
    }

    #[tokio::test]
    async fn test_sequential_execution_with_multiple_agents() {
        let event_bus = Arc::new(EventBus::new());
        let executor = SequentialExecutor::new(event_bus.clone());

        let workflow = WorkflowDefinition {
            name: "Multi-Agent Workflow".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    agent_type: "TestAgent".to_string(),
                    task: "First task".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
                AgentConfig {
                    id: "agent2".to_string(),
                    agent_type: "TestAgent".to_string(),
                    task: "Second task".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
            ],
            inputs: None,
        };

        let mut context = ExecutionContext::new("test-workflow".to_string());
        let result = executor.execute(&workflow, &mut context).await;

        assert!(result.is_ok());

        // Verify both agents stored their outputs in context
        assert!(context.get_value("agent1.output").is_some());
        assert!(context.get_value("agent2.output").is_some());
    }

    #[tokio::test]
    async fn test_sequential_execution_with_variable_interpolation() {
        let event_bus = Arc::new(EventBus::new());
        let executor = SequentialExecutor::new(event_bus.clone());

        let workflow = WorkflowDefinition {
            name: "Interpolation Test".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    agent_type: "TestAgent".to_string(),
                    task: "Process input".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
                AgentConfig {
                    id: "agent2".to_string(),
                    agent_type: "TestAgent".to_string(),
                    task: "Use output from {agent1.output}".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
            ],
            inputs: None,
        };

        let mut context = ExecutionContext::new("test-workflow".to_string());
        let result = executor.execute(&workflow, &mut context).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_events_published_during_execution() {
        let event_bus = Arc::new(EventBus::new());
        let mut subscriber = event_bus.subscribe();
        let executor = SequentialExecutor::new(event_bus.clone());

        let workflow = WorkflowDefinition {
            name: "Event Test".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![AgentConfig {
                id: "agent1".to_string(),
                agent_type: "TestAgent".to_string(),
                task: "Test task".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            }],
            inputs: None,
        };

        let mut context = ExecutionContext::new("test-workflow".to_string());

        // Execute in a separate task so we can receive events
        let exec_handle = tokio::spawn(async move {
            executor.execute(&workflow, &mut context).await
        });

        // Receive TaskStarted event
        let event1 = subscriber.recv().await.unwrap();
        match event1 {
            AgentEvent::TaskStarted { agent_id, task } => {
                assert_eq!(agent_id, "agent1");
                assert_eq!(task, "Test task");
            }
            _ => panic!("Expected TaskStarted event"),
        }

        // Receive TaskCompleted event
        let event2 = subscriber.recv().await.unwrap();
        match event2 {
            AgentEvent::TaskCompleted { agent_id, .. } => {
                assert_eq!(agent_id, "agent1");
            }
            _ => panic!("Expected TaskCompleted event"),
        }

        // Wait for execution to complete
        let result = exec_handle.await.unwrap();
        assert!(result.is_ok());
    }
}
