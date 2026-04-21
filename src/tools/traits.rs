use async_trait::async_trait;
use crate::error::Result;
use crate::tools::types::{ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;

/// Tool trait for all tool implementations
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Get the tool description
    fn description(&self) -> &str;

    /// Get the tool parameters
    fn parameters(&self) -> Vec<ToolParameter>;

    /// Execute the tool with given parameters
    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::types::ParameterType;
    use serde_json::json;

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock_tool"
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![ToolParameter::new(
                "input",
                "Test input",
                ParameterType::String,
                true,
            )]
        }

        async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
            let input = params
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            Ok(ToolResult::success(json!({
                "result": format!("Processed: {}", input)
            })))
        }
    }

    #[tokio::test]
    async fn test_tool_trait_implementation() {
        let tool = MockTool;

        assert_eq!(tool.name(), "mock_tool");
        assert_eq!(tool.description(), "A mock tool for testing");
        assert_eq!(tool.parameters().len(), 1);

        let mut params = HashMap::new();
        params.insert("input".to_string(), json!("test"));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output["result"], "Processed: test");
    }

    #[tokio::test]
    async fn test_tool_with_missing_param() {
        let tool = MockTool;
        let params = HashMap::new();

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output["result"], "Processed: default");
    }
}
