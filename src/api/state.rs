use crate::agents::AgentRegistry;
use crate::config::GlobalConfig;
use crate::engine::EventBus;
use crate::llm::LLMRegistry;
use crate::security::{AuditLogger, PermissionManager};
use crate::storage::StateStore;
use crate::tools::ToolRegistry;
use std::sync::Arc;

/// Shared application state for API handlers
#[derive(Clone)]
pub struct AppState {
    /// Global configuration
    pub config: Arc<GlobalConfig>,
    /// LLM provider registry
    pub llm_registry: Arc<LLMRegistry>,
    /// Agent registry
    pub agent_registry: Arc<AgentRegistry>,
    /// Tool registry
    pub tool_registry: Arc<ToolRegistry>,
    /// Permission manager
    pub permission_manager: Arc<PermissionManager>,
    /// State store for executions
    pub state_store: Arc<StateStore>,
    /// Event bus for real-time updates
    pub event_bus: Arc<EventBus>,
    /// Audit logger
    pub audit_logger: Arc<AuditLogger>,
}

impl AppState {
    /// Create a new application state
    pub fn new(
        config: Arc<GlobalConfig>,
        llm_registry: Arc<LLMRegistry>,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<ToolRegistry>,
        permission_manager: Arc<PermissionManager>,
        state_store: Arc<StateStore>,
        event_bus: Arc<EventBus>,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            config,
            llm_registry,
            agent_registry,
            tool_registry,
            permission_manager,
            state_store,
            event_bus,
            audit_logger,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::traits::LLMProvider;
    use crate::llm::types::{Message, MessageRole};
    use crate::security::PermissionPolicy;
    use async_trait::async_trait;
    use std::path::PathBuf;

    // Mock LLM provider for testing
    struct MockProvider;

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }

        async fn complete(
            &self,
            _messages: Vec<Message>,
            _options: crate::llm::types::CompletionOptions,
        ) -> crate::error::Result<String> {
            Ok("mock response".to_string())
        }

        fn supports_streaming(&self) -> bool {
            false
        }

        fn max_context_tokens(&self) -> usize {
            4096
        }
    }

    #[test]
    fn test_app_state_creation() {
        let config = Arc::new(GlobalConfig::default());
        let llm_registry = Arc::new(LLMRegistry::new(Arc::new(MockProvider)));
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        let permission_manager = Arc::new(PermissionManager::allow_all());

        // Use temporary file for Windows compatibility
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("rustforge_test_state.db");
        let state_store = Arc::new(
            StateStore::new(&db_path).unwrap()
        );

        let event_bus = Arc::new(EventBus::new());
        let audit_logger = Arc::new(AuditLogger::new());

        let state = AppState::new(
            config,
            llm_registry,
            agent_registry,
            tool_registry,
            permission_manager,
            state_store,
            event_bus,
            audit_logger,
        );

        // Verify state is clonable
        let _cloned = state.clone();

        // Cleanup
        std::fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_app_state_clone() {
        let config = Arc::new(GlobalConfig::default());
        let llm_registry = Arc::new(LLMRegistry::new(Arc::new(MockProvider)));
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        let permission_manager = Arc::new(PermissionManager::deny_all());

        // Use temporary file for Windows compatibility
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("rustforge_test_state_clone.db");
        let state_store = Arc::new(
            StateStore::new(&db_path).unwrap()
        );

        let event_bus = Arc::new(EventBus::new());
        let audit_logger = Arc::new(AuditLogger::new());

        let state1 = AppState::new(
            config,
            llm_registry,
            agent_registry,
            tool_registry,
            permission_manager,
            state_store,
            event_bus,
            audit_logger,
        );

        let state2 = state1.clone();

        // Both states should share the same underlying data
        assert_eq!(
            state1.permission_manager.default_policy(),
            state2.permission_manager.default_policy()
        );

        // Cleanup
        std::fs::remove_file(db_path).ok();
    }
}
