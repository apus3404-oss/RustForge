use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::tools::traits::Tool;
use crate::tools::types::{ParameterType, ToolParameter, ToolResult};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct PdfParserTool;

impl PdfParserTool {
    pub fn new() -> Self {
        Self
    }

    async fn extract_text(&self, path: &Path) -> Result<ToolResult> {
        // Read PDF file
        let bytes = fs::read(path).map_err(|e| {
            Error::Internal(format!("Failed to read PDF file: {}", e))
        })?;

        // Extract text using pdf_extract
        let text = pdf_extract::extract_text_from_mem(&bytes).map_err(|e| {
            Error::Internal(format!("Failed to extract text from PDF: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "path": path.display().to_string(),
            "text": text,
            "length": text.len()
        })))
    }

    async fn get_metadata(&self, path: &Path) -> Result<ToolResult> {
        let bytes = fs::read(path).map_err(|e| {
            Error::Internal(format!("Failed to read PDF file: {}", e))
        })?;

        // Parse PDF to get metadata
        let doc = lopdf::Document::load_mem(&bytes).map_err(|e| {
            Error::Internal(format!("Failed to parse PDF: {}", e))
        })?;

        let page_count = doc.get_pages().len();

        // Try to extract metadata
        let mut metadata = HashMap::new();

        if let Ok(info_dict) = doc.trailer.get(b"Info") {
            if let Ok(info_ref) = info_dict.as_reference() {
                if let Ok(info_obj) = doc.get_object(info_ref) {
                    if let Ok(info_dict) = info_obj.as_dict() {
                        // Extract common metadata fields
                        for (key, value) in info_dict.iter() {
                            if let Ok(key_str) = std::str::from_utf8(key) {
                                if let Ok(val_str) = value.as_string() {
                                    metadata.insert(key_str.to_string(), val_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ToolResult::success(json!({
            "path": path.display().to_string(),
            "page_count": page_count,
            "metadata": metadata
        })))
    }
}

impl Default for PdfParserTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for PdfParserTool {
    fn name(&self) -> &str {
        "pdf_parser"
    }

    fn description(&self) -> &str {
        "Extract text and metadata from PDF files"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter::new(
                "operation",
                "Operation: extract_text, metadata",
                ParameterType::String,
                true,
            ),
            ToolParameter::new(
                "path",
                "Path to PDF file",
                ParameterType::String,
                true,
            ),
        ]
    }

    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing operation parameter".to_string()))?;

        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing path parameter".to_string()))?;

        let path = Path::new(path_str);

        match operation {
            "extract_text" => self.extract_text(path).await,
            "metadata" => self.get_metadata(path).await,
            _ => Err(Error::Internal(format!("Unknown operation: {}", operation))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = PdfParserTool::new();
        assert_eq!(tool.name(), "pdf_parser");
        assert!(!tool.description().is_empty());
        assert_eq!(tool.parameters().len(), 2);
    }

    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = PdfParserTool::new();
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("invalid"));
        params.insert("path".to_string(), json!("test.pdf"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_path() {
        let tool = PdfParserTool::new();
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("extract_text"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }
}
