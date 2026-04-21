// src/engine/executor.rs
use crate::agents::{Agent, AgentRegistry, BaseAgent, Task as AgentTask};
use crate::engine::events::{AgentEvent, EventBus};
use crate::engine::interpolation::VariableInterpolator;
use crate::engine::types::{ExecutionContext, WorkflowDefinition};
use crate::error::{Error, Result};
use crate::llm::LLMProvider;
use std::sync::Arc;
use tracing::{debug, info};

/// Sequential executor for workflows with real agent integration
pub struct SequentialExecutor {
    event_bus: Arc<EventBus>,
    llm_provider: Arc<dyn LLMProvider>,
    agent_registry: Arc<AgentRegistry>,
}

impl SequentialExecutor {
    pub fn new(
        event_bus: Arc<EventBus>,
        llm_provider: Arc<dyn LLMProvider>,
        agent_registry: Arc<AgentRegistry>,
    ) -> Self {
        Self {
            event_bus,
            llm_provider,
            agent_registry,
        }
    }

    /// Execute a workflow sequentially using real agents
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

        for agent_config in &workflow.agents {
            debug!("Executing agent: {}", agent_config.id);

            // Interpolate task variables
            let interpolator = VariableInterpolator::new(context);
            let interpolated_task = interpolator.interpolate(&agent_config.task)?;

            // Publish TaskStarted event
            self.event_bus
                .publish(AgentEvent::TaskStarted {
                    agent_id: agent_config.id.clone(),
                    task: interpolated_task.clone(),
                })
                .ok(); // Ignore send errors if no subscribers

            // Get or create agent instance
            let agent = self.get_or_create_agent(&agent_config.id, &agent_config.agent_type)?;

            // Create task for agent
            let task = AgentTask::new(&agent_config.id, &interpolated_task);

            // Execute agent
            info!("Agent {} executing task: {}", agent_config.id, interpolated_task);
            let agent_output = agent.execute(task).await?;

            // Store agent output in context for next agents to use
            let output_key = format!("{}.output", agent_config.id);
            if let Some(result) = &agent_output.result {
                context.set_value(&output_key, result.clone());
            }

            // Create output JSON
            let output_json = serde_json::json!({
                "agent_id": agent_output.agent_id,
                "task_id": agent_output.task_id,
                "status": agent_output.status,
                "result": agent_output.result,
                "error": agent_output.error,
            });

            // Publish TaskCompleted event
            self.event_bus
                .publish(AgentEvent::TaskCompleted {
                    agent_id: agent_config.id.clone(),
                    output: agent_output.result.map(|v| v.to_string()).unwrap_or_default(),
                })
                .ok(); // Ignore send errors if no subscribers

            // Add to final output
            if let Some(agents_array) = final_output["agents"].as_array_mut() {
                agents_array.push(output_json);
            }

            debug!("Agent {} completed successfully", agent_config.id);
        }

        info!("Workflow execution completed");
        Ok(final_output)
    }

    fn get_or_create_agent(&self, agent_id: &str, agent_type: &str) -> Result<Arc<dyn Agent>> {
        // Try to get from registry first
        if let Some(agent) = self.agent_registry.get(agent_id) {
            return Ok(agent);
        }

        // Create new BaseAgent if not found
        let definition = crate::agents::AgentDefinition::new(agent_id, agent_type);
        let agent = Arc::new(BaseAgent::new(definition, self.llm_provider.clone()));

        // Register for future use
        self.agent_registry.register(agent.clone())?;

        Ok(agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::AgentDefinition;
    use crate::engine::types::AgentConfig;
    use crate::llm::types::{CompletionOptions, Message};
    use std::collections::HashMap;

    struct MockLLMProvider;

    #[async_trait::async_trait]
    impl LLMProvider for MockLLMProvider {
        async fn complete(
            &self,
            _messages: Vec<Message>,
            _options: CompletionOptions,
        ) -> Result<String> {
            Ok("Mock LLM response".to_string())
        }

        fn supports_streaming(&self) -> bool {
            false
        }

        fn max_context_tokens(&self) -> usize {
            4096
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_sequential_execution_with_single_agent() {
        let event_bus = Arc::new(EventBus::new());
        let llm = Arc::new(MockLLMProvider);
        let registry = Arc::new(AgentRegistry::new());
        let executor = SequentialExecutor::new(event_bus, llm, registry);

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
        let llm = Arc::new(MockLLMProvider);
        let registry = Arc::new(AgentRegistry::new());
        let executor = SequentialExecutor::new(event_bus, llm, registry);

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
        let llm = Arc::new(MockLLMProvider);
        let registry = Arc::new(AgentRegistry::new());
        let executor = SequentialExecutor::new(event_bus, llm, registry);

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
}
