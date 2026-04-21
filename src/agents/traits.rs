use async_trait::async_trait;
use crate::error::Result;
use crate::llm::LLMProvider;
use super::types::{AgentDefinition, AgentOutput, Task};
use std::sync::Arc;

/// Trait for all agent implementations
#[async_trait]
pub trait Agent: Send + Sync {
    /// Execute a task and return the output
    async fn execute(&self, task: Task) -> Result<AgentOutput>;

    /// Get the agent's definition
    fn definition(&self) -> &AgentDefinition;

    /// Get the agent's unique identifier
    fn id(&self) -> &str {
        &self.definition().id
    }

    /// Get the agent's type
    fn agent_type(&self) -> &str {
        &self.definition().agent_type
    }

    /// Check if this agent can handle a specific task type
    fn can_handle(&self, task_type: &str) -> bool;
}

/// Base agent that uses an LLM provider
///
/// **Note:** This is a stub implementation for Phase 2.
/// Real LLM-based execution will be implemented in Phase 3.
/// Currently returns mock responses for testing integration.
pub struct BaseAgent {
    definition: AgentDefinition,
    llm_provider: Arc<dyn LLMProvider>,
}

impl BaseAgent {
    pub fn new(definition: AgentDefinition, llm_provider: Arc<dyn LLMProvider>) -> Self {
        Self {
            definition,
            llm_provider,
        }
    }

    pub fn llm_provider(&self) -> &Arc<dyn LLMProvider> {
        &self.llm_provider
    }
}

#[async_trait]
impl Agent for BaseAgent {
    async fn execute(&self, task: Task) -> Result<AgentOutput> {
        // Stub implementation - will be enhanced in Phase 3
        Ok(AgentOutput::success(
            self.id(),
            &task.id,
            serde_json::json!({
                "message": "Task executed by base agent",
                "task_description": task.description,
            }),
        ))
    }

    fn definition(&self) -> &AgentDefinition {
        &self.definition
    }

    fn can_handle(&self, _task_type: &str) -> bool {
        true // Base agent can handle any task type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::types::{CompletionOptions, Message};

    struct MockLLMProvider;

    #[async_trait]
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
    async fn test_base_agent_execute() {
        let definition = AgentDefinition::new("agent1", "base");
        let llm = Arc::new(MockLLMProvider);
        let agent = BaseAgent::new(definition, llm);

        let task = Task::new("task1", "Test task");
        let output = agent.execute(task).await.unwrap();

        assert_eq!(output.agent_id, "agent1");
        assert_eq!(output.task_id, "task1");
        assert!(output.result.is_some());
    }

    #[test]
    fn test_agent_metadata() {
        let definition = AgentDefinition::new("agent1", "research");
        let llm = Arc::new(MockLLMProvider);
        let agent = BaseAgent::new(definition, llm);

        assert_eq!(agent.id(), "agent1");
        assert_eq!(agent.agent_type(), "research");
        assert!(agent.can_handle("any_type"));
    }
}
