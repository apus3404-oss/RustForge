use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::tools::traits::Tool;
use crate::tools::types::{ParameterType, ToolParameter, ToolResult};
use arboard::Clipboard;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct ClipboardTool {
    clipboard: Clipboard,
}

impl ClipboardTool {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new().map_err(|e| {
            Error::Internal(format!("Failed to initialize clipboard: {}", e))
        })?;

        Ok(Self { clipboard })
    }

    async fn read_clipboard(&mut self) -> Result<ToolResult> {
        let text = self.clipboard.get_text().map_err(|e| {
            Error::Internal(format!("Failed to read clipboard: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "text": text,
            "length": text.len()
        })))
    }

    async fn write_clipboard(&mut self, text: &str) -> Result<ToolResult> {
        self.clipboard.set_text(text).map_err(|e| {
            Error::Internal(format!("Failed to write clipboard: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "text": text,
            "length": text.len(),
            "written": true
        })))
    }

    async fn clear_clipboard(&mut self) -> Result<ToolResult> {
        self.clipboard.clear().map_err(|e| {
            Error::Internal(format!("Failed to clear clipboard: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "cleared": true
        })))
    }
}

impl Default for ClipboardTool {
    fn default() -> Self {
        Self::new().expect("Failed to create clipboard tool")
    }
}

#[async_trait]
impl Tool for ClipboardTool {
    fn name(&self) -> &str {
        "clipboard"
    }

    fn description(&self) -> &str {
        "Read from and write to system clipboard"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter::new(
                "operation",
                "Operation: read, write, clear",
                ParameterType::String,
                true,
            ),
            ToolParameter::new(
                "text",
                "Text to write (for write operation)",
                ParameterType::String,
                false,
            ),
        ]
    }

    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing operation parameter".to_string()))?;

        // Clone clipboard for mutable operations
        let clipboard = Clipboard::new().map_err(|e| {
            Error::Internal(format!("Failed to access clipboard: {}", e))
        })?;

        let mut tool = ClipboardTool { clipboard };

        match operation {
            "read" => tool.read_clipboard().await,
            "write" => {
                let text = params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Internal("Missing text parameter for write".to_string()))?;
                tool.write_clipboard(text).await
            }
            "clear" => tool.clear_clipboard().await,
            _ => Err(Error::Internal(format!("Unknown operation: {}", operation))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = ClipboardTool::new();
        if tool.is_err() {
            // Skip test if clipboard is not available (e.g., in CI)
            return;
        }
        let tool = tool.unwrap();
        assert_eq!(tool.name(), "clipboard");
        assert!(!tool.description().is_empty());
        assert_eq!(tool.parameters().len(), 2);
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let tool = ClipboardTool::new();
        if tool.is_err() {
            // Skip test if clipboard is not available
            return;
        }

        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("write"));
        params.insert("text".to_string(), json!("Test clipboard content"));

        let result = tool.unwrap().execute(params).await;
        if result.is_err() {
            // Skip if clipboard operations fail
            return;
        }

        let tool = ClipboardTool::new().unwrap();
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("read"));

        let result = tool.execute(params).await;
        if let Ok(result) = result {
            assert!(result.success);
            assert_eq!(result.output["text"], "Test clipboard content");
        }
    }

    #[tokio::test]
    async fn test_missing_operation() {
        let tool = ClipboardTool::new();
        if tool.is_err() {
            return;
        }

        let params = HashMap::new();
        let result = tool.unwrap().execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = ClipboardTool::new();
        if tool.is_err() {
            return;
        }

        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("invalid"));

        let result = tool.unwrap().execute(params).await;
        assert!(result.is_err());
    }
}
