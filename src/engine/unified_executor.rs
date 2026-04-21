// src/engine/unified_executor.rs
use crate::agents::AgentRegistry;
use crate::engine::{
    EventBus, ExecutionContext, ExecutionMode, MergeStrategy, ParallelExecutor,
    SequentialExecutor, WorkflowDefinition,
};
use crate::error::Result;
use crate::llm::LLMProvider;
use std::sync::Arc;

/// Unified executor that supports both sequential and parallel execution modes
pub struct WorkflowExecutor {
    sequential_executor: SequentialExecutor,
    parallel_executor: ParallelExecutor,
}

impl WorkflowExecutor {
    pub fn new(
        event_bus: Arc<EventBus>,
        llm_provider: Arc<dyn LLMProvider>,
        agent_registry: Arc<AgentRegistry>,
    ) -> Self {
        let sequential_executor =
            SequentialExecutor::new(event_bus.clone(), llm_provider, agent_registry);
        let parallel_executor = ParallelExecutor::new(event_bus);

        Self {
            sequential_executor,
            parallel_executor,
        }
    }

    /// Execute workflow based on its execution mode
    pub async fn execute(
        &self,
        workflow: &WorkflowDefinition,
        context: &mut ExecutionContext,
    ) -> Result<serde_json::Value> {
        match workflow.mode {
            ExecutionMode::Sequential => {
                // Use sequential executor
                self.sequential_executor.execute(workflow, context).await
            }
            ExecutionMode::Parallel => {
                // Use parallel executor
                let results = self
                    .parallel_executor
                    .execute(workflow.agents.clone(), context)
                    .await?;

                // Convert parallel results to same format as sequential
                let mut agents_output = Vec::new();
                for (i, result) in results.iter().enumerate() {
                    agents_output.push(serde_json::json!({
                        "agent_id": workflow.agents[i].id,
                        "task_id": format!("task_{}", i),
                        "status": "completed",
                        "result": result,
                        "error": null,
                    }));
                }

                Ok(serde_json::json!({
                    "workflow": workflow.name,
                    "execution_id": context.execution_id.to_string(),
                    "agents": agents_output,
                }))
            }
            ExecutionMode::Dag => {
                // TODO: Implement DAG execution in future phase
                Err(crate::error::Error::Internal(
                    "DAG execution mode not yet implemented".to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            Ok("Mock response".to_string())
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
    async fn test_unified_executor_with_sequential_mode() {
        let event_bus = Arc::new(EventBus::new());
        let llm = Arc::new(MockLLMProvider);
        let registry = Arc::new(AgentRegistry::new());
        let executor = WorkflowExecutor::new(event_bus, llm, registry);

        let workflow = WorkflowDefinition {
            name: "Sequential Test".to_string(),
            mode: ExecutionMode::Sequential,
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    agent_type: "test".to_string(),
                    task: "Task 1".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
                AgentConfig {
                    id: "agent2".to_string(),
                    agent_type: "test".to_string(),
                    task: "Task 2".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
            ],
            inputs: None,
        };

        let mut context = ExecutionContext::new("test-workflow".to_string());
        let result = executor.execute(&workflow, &mut context).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_object());
    }

    #[tokio::test]
    async fn test_unified_executor_with_parallel_mode() {
        let event_bus = Arc::new(EventBus::new());
        let llm = Arc::new(MockLLMProvider);
        let registry = Arc::new(AgentRegistry::new());
        let executor = WorkflowExecutor::new(event_bus, llm, registry);

        let workflow = WorkflowDefinition {
            name: "Parallel Test".to_string(),
            mode: ExecutionMode::Parallel,
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    agent_type: "test".to_string(),
                    task: "Task 1".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
                AgentConfig {
                    id: "agent2".to_string(),
                    agent_type: "test".to_string(),
                    task: "Task 2".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
                AgentConfig {
                    id: "agent3".to_string(),
                    agent_type: "test".to_string(),
                    task: "Task 3".to_string(),
                    depends_on: vec![],
                    config: HashMap::new(),
                },
            ],
            inputs: None,
        };

        let mut context = ExecutionContext::new("test-workflow".to_string());
        let result = executor.execute(&workflow, &mut context).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_object());
    }
}
