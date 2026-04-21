use crate::agents::traits::Agent;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for managing agent instances (thread-safe)
pub struct AgentRegistry {
    agents: RwLock<HashMap<String, Arc<dyn Agent>>>,
}

impl AgentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent with the registry
    pub fn register(&self, agent: Arc<dyn Agent>) -> Result<()> {
        let id = agent.id().to_string();
        let mut agents = self.agents.write().unwrap();

        if agents.contains_key(&id) {
            return Err(Error::InvalidWorkflowDefinition {
                reason: format!("Agent with id '{}' is already registered", id),
            });
        }

        agents.insert(id, agent);
        Ok(())
    }

    /// Get an agent by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().unwrap();
        agents.get(id).cloned()
    }

    /// Check if an agent is registered
    pub fn contains(&self, id: &str) -> bool {
        let agents = self.agents.read().unwrap();
        agents.contains_key(id)
    }

    /// Get all registered agent IDs
    pub fn agent_ids(&self) -> Vec<String> {
        let agents = self.agents.read().unwrap();
        agents.keys().cloned().collect()
    }

    /// Get the number of registered agents
    pub fn len(&self) -> usize {
        let agents = self.agents.read().unwrap();
        agents.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        let agents = self.agents.read().unwrap();
        agents.is_empty()
    }

    /// Remove an agent from the registry
    pub fn unregister(&self, id: &str) -> Option<Arc<dyn Agent>> {
        let mut agents = self.agents.write().unwrap();
        agents.remove(id)
    }

    /// Clear all agents from the registry
    pub fn clear(&self) {
        let mut agents = self.agents.write().unwrap();
        agents.clear();
    }

    /// Find agents by type
    pub fn find_by_type(&self, agent_type: &str) -> Vec<Arc<dyn Agent>> {
        let agents = self.agents.read().unwrap();
        agents
            .values()
            .filter(|agent| agent.agent_type() == agent_type)
            .cloned()
            .collect()
    }

    /// Find agents that can handle a specific task type
    pub fn find_capable(&self, task_type: &str) -> Vec<Arc<dyn Agent>> {
        let agents = self.agents.read().unwrap();
        agents
            .values()
            .filter(|agent| agent.can_handle(task_type))
            .cloned()
            .collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::traits::BaseAgent;
    use crate::agents::types::{AgentDefinition, AgentOutput, Task};
    use crate::llm::types::{CompletionOptions, Message};
    use crate::llm::LLMProvider;
    use async_trait::async_trait;

    struct MockLLMProvider;

    #[async_trait]
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

    fn create_test_agent(id: &str, agent_type: &str) -> Arc<dyn Agent> {
        let definition = AgentDefinition::new(id, agent_type);
        let llm = Arc::new(MockLLMProvider);
        Arc::new(BaseAgent::new(definition, llm))
    }

    #[test]
    fn test_register_agent() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("agent1", "research");

        assert!(registry.register(agent).is_ok());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("agent1"));
    }

    #[test]
    fn test_register_duplicate_agent() {
        let registry = AgentRegistry::new();
        let agent1 = create_test_agent("agent1", "research");
        let agent2 = create_test_agent("agent1", "analysis");

        registry.register(agent1).unwrap();
        let result = registry.register(agent2);

        assert!(result.is_err());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_get_agent() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("agent1", "research");
        registry.register(agent).unwrap();

        let retrieved = registry.get("agent1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), "agent1");
    }

    #[test]
    fn test_get_nonexistent_agent() {
        let registry = AgentRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_unregister_agent() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("agent1", "research");
        registry.register(agent).unwrap();

        let removed = registry.unregister("agent1");
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
        assert!(!registry.contains("agent1"));
    }

    #[test]
    fn test_agent_ids() {
        let registry = AgentRegistry::new();
        registry.register(create_test_agent("agent1", "research")).unwrap();
        registry.register(create_test_agent("agent2", "analysis")).unwrap();

        let ids = registry.agent_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"agent1".to_string()));
        assert!(ids.contains(&"agent2".to_string()));
    }

    #[test]
    fn test_find_by_type() {
        let registry = AgentRegistry::new();
        registry.register(create_test_agent("agent1", "research")).unwrap();
        registry.register(create_test_agent("agent2", "research")).unwrap();
        registry.register(create_test_agent("agent3", "analysis")).unwrap();

        let research_agents = registry.find_by_type("research");
        assert_eq!(research_agents.len(), 2);

        let analysis_agents = registry.find_by_type("analysis");
        assert_eq!(analysis_agents.len(), 1);
    }

    #[test]
    fn test_find_capable() {
        let registry = AgentRegistry::new();
        registry.register(create_test_agent("agent1", "research")).unwrap();
        registry.register(create_test_agent("agent2", "analysis")).unwrap();

        // BaseAgent can handle any task type
        let capable = registry.find_capable("any_task");
        assert_eq!(capable.len(), 2);
    }

    #[test]
    fn test_clear_registry() {
        let registry = AgentRegistry::new();
        registry.register(create_test_agent("agent1", "research")).unwrap();
        registry.register(create_test_agent("agent2", "analysis")).unwrap();

        assert_eq!(registry.len(), 2);
        registry.clear();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_default_registry() {
        let registry = AgentRegistry::default();
        assert!(registry.is_empty());
    }
}
