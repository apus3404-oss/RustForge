use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Parameter type for tool inputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default: Option<Value>,
}

impl ToolParameter {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        param_type: ParameterType,
        required: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type,
            required,
            default: None,
        }
    }

    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }
}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ToolResult {
    pub fn success(output: Value) -> Self {
        Self {
            success: true,
            output,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: Value::Null,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parameter_type_serialization() {
        let param_type = ParameterType::String;
        let serialized = serde_json::to_string(&param_type).unwrap();
        let deserialized: ParameterType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(param_type, deserialized);
    }

    #[test]
    fn test_tool_parameter_builder() {
        let param = ToolParameter::new("name", "User name", ParameterType::String, true)
            .with_default(json!("default_name"));

        assert_eq!(param.name, "name");
        assert_eq!(param.description, "User name");
        assert_eq!(param.param_type, ParameterType::String);
        assert!(param.required);
        assert_eq!(param.default, Some(json!("default_name")));
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success(json!({"data": "test"}))
            .with_metadata("duration", "100ms");

        assert!(result.success);
        assert_eq!(result.output, json!({"data": "test"}));
        assert!(result.error.is_none());
        assert_eq!(result.metadata.get("duration"), Some(&"100ms".to_string()));
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");

        assert!(!result.success);
        assert_eq!(result.output, Value::Null);
        assert_eq!(result.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult::success(json!({"key": "value"}));
        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: ToolResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(result.success, deserialized.success);
        assert_eq!(result.output, deserialized.output);
    }
}
