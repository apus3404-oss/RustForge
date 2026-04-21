use crate::error::{Error, Result};
use crate::tools::traits::Tool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe registry for managing tools
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool with the registry
    pub fn register(&self, tool: Arc<dyn Tool>) -> Result<()> {
        let name = tool.name().to_string();
        let mut tools = self.tools.write().unwrap();

        if tools.contains_key(&name) {
            return Err(Error::Internal(format!(
                "Tool '{}' is already registered",
                name
            )));
        }

        tools.insert(name, tool);
        Ok(())
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().unwrap();
        tools.get(name).cloned()
    }

    /// Check if a tool is registered
    pub fn contains(&self, name: &str) -> bool {
        let tools = self.tools.read().unwrap();
        tools.contains_key(name)
    }

    /// List all registered tool names
    pub fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().unwrap();
        tools.keys().cloned().collect()
    }

    /// Unregister a tool
    pub fn unregister(&self, name: &str) -> Result<()> {
        let mut tools = self.tools.write().unwrap();

        if tools.remove(name).is_none() {
            return Err(Error::Internal(format!("Tool '{}' not found", name)));
        }

        Ok(())
    }

    /// Get the number of registered tools
    pub fn count(&self) -> usize {
        let tools = self.tools.read().unwrap();
        tools.len()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::types::{ParameterType, ToolParameter, ToolResult};
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    struct MockTool {
        name: String,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock tool for testing"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![ToolParameter::new(
                "input",
                "Test input",
                ParameterType::String,
                false,
            )]
        }

        async fn execute(&self, _params: HashMap<String, Value>) -> Result<ToolResult> {
            Ok(ToolResult::success(json!({"result": "ok"})))
        }
    }

    #[test]
    fn test_register_and_get_tool() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockTool::new("test_tool"));

        registry.register(tool.clone()).unwrap();

        let retrieved = registry.get("test_tool").unwrap();
        assert_eq!(retrieved.name(), "test_tool");
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = ToolRegistry::new();
        let tool1 = Arc::new(MockTool::new("test_tool"));
        let tool2 = Arc::new(MockTool::new("test_tool"));

        registry.register(tool1).unwrap();
        let result = registry.register(tool2);

        assert!(result.is_err());
    }

    #[test]
    fn test_contains() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockTool::new("test_tool"));

        assert!(!registry.contains("test_tool"));
        registry.register(tool).unwrap();
        assert!(registry.contains("test_tool"));
    }

    #[test]
    fn test_list_tools() {
        let registry = ToolRegistry::new();
        let tool1 = Arc::new(MockTool::new("tool1"));
        let tool2 = Arc::new(MockTool::new("tool2"));

        registry.register(tool1).unwrap();
        registry.register(tool2).unwrap();

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"tool1".to_string()));
        assert!(tools.contains(&"tool2".to_string()));
    }

    #[test]
    fn test_unregister() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockTool::new("test_tool"));

        registry.register(tool).unwrap();
        assert!(registry.contains("test_tool"));

        registry.unregister("test_tool").unwrap();
        assert!(!registry.contains("test_tool"));
    }

    #[test]
    fn test_unregister_nonexistent() {
        let registry = ToolRegistry::new();
        let result = registry.unregister("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_count() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.count(), 0);

        let tool1 = Arc::new(MockTool::new("tool1"));
        let tool2 = Arc::new(MockTool::new("tool2"));

        registry.register(tool1).unwrap();
        assert_eq!(registry.count(), 1);

        registry.register(tool2).unwrap();
        assert_eq!(registry.count(), 2);

        registry.unregister("tool1").unwrap();
        assert_eq!(registry.count(), 1);
    }
}
