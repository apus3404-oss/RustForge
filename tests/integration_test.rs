// tests/integration_test.rs
use rustforge::engine::{EventBus, WorkflowExecutor, WorkflowDefinition, ExecutionMode, AgentConfig, ExecutionContext};
use rustforge::agents::AgentRegistry;
use rustforge::llm::LLMProvider;
use rustforge::llm::types::{CompletionOptions, Message};
use rustforge::error::Result;
use std::sync::Arc;
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
async fn test_workflow_execution_integration() {
    let event_bus = Arc::new(EventBus::new());
    let llm = Arc::new(MockLLMProvider);
    let agent_registry = Arc::new(AgentRegistry::new());

    let executor = WorkflowExecutor::new(event_bus, llm, agent_registry);

    // Create a simple workflow
    let workflow = WorkflowDefinition {
        name: "Integration Test Workflow".to_string(),
        mode: ExecutionMode::Sequential,
        agents: vec![
            AgentConfig {
                id: "agent1".to_string(),
                agent_type: "test".to_string(),
                task: "Task 1".to_string(),
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
    assert_eq!(output["workflow"], "Integration Test Workflow");
}

#[tokio::test]
async fn test_parallel_execution_integration() {
    let event_bus = Arc::new(EventBus::new());
    let llm = Arc::new(MockLLMProvider);
    let agent_registry = Arc::new(AgentRegistry::new());

    let executor = WorkflowExecutor::new(event_bus, llm, agent_registry);

    // Create a parallel workflow
    let workflow = WorkflowDefinition {
        name: "Parallel Integration Test".to_string(),
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
    assert_eq!(output["workflow"], "Parallel Integration Test");

    // Verify all 3 agents executed
    let agents = output["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 3);
}

