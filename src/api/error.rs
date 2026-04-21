use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn not_found(resource: &str, id: &str) -> Self {
        Self::new(
            "NOT_FOUND",
            format!("{} '{}' not found", resource, id),
        )
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new("PERMISSION_DENIED", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "BAD_REQUEST" | "INVALID_WORKFLOW" | "INVALID_INPUT" => StatusCode::BAD_REQUEST,
            "PERMISSION_DENIED" | "FORBIDDEN" => StatusCode::FORBIDDEN,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl From<crate::error::Error> for ApiError {
    fn from(err: crate::error::Error) -> Self {
        match err {
            crate::error::Error::WorkflowNotFound { workflow_id } => {
                ApiError::not_found("Workflow", &workflow_id)
            }
            crate::error::Error::InvalidWorkflowDefinition { reason } => {
                ApiError::bad_request(format!("Invalid workflow: {}", reason))
            }
            crate::error::Error::Internal(msg) => ApiError::internal_error(msg),
            _ => ApiError::internal_error(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::new("TEST_ERROR", "Test message");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_api_error_with_details() {
        let error = ApiError::new("TEST_ERROR", "Test message")
            .with_details(serde_json::json!({"field": "value"}));
        assert!(error.details.is_some());
    }

    #[test]
    fn test_not_found_error() {
        let error = ApiError::not_found("Workflow", "test-id");
        assert_eq!(error.code, "NOT_FOUND");
        assert!(error.message.contains("Workflow"));
        assert!(error.message.contains("test-id"));
    }

    #[test]
    fn test_bad_request_error() {
        let error = ApiError::bad_request("Invalid input");
        assert_eq!(error.code, "BAD_REQUEST");
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn test_internal_error() {
        let error = ApiError::internal_error("Something went wrong");
        assert_eq!(error.code, "INTERNAL_ERROR");
        assert_eq!(error.message, "Something went wrong");
    }

    #[test]
    fn test_permission_denied_error() {
        let error = ApiError::permission_denied("Access denied");
        assert_eq!(error.code, "PERMISSION_DENIED");
        assert_eq!(error.message, "Access denied");
    }

    #[test]
    fn test_error_serialization() {
        let error = ApiError::new("TEST", "message")
            .with_details(serde_json::json!({"key": "value"}));
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("TEST"));
        assert!(json.contains("message"));
        assert!(json.contains("key"));
    }
}
