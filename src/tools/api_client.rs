use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::tools::traits::Tool;
use crate::tools::types::{ParameterType, ToolParameter, ToolResult};
use reqwest::{Client, Method};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

pub struct ApiClientTool {
    client: Client,
}

impl ApiClientTool {
    pub fn new(timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap();

        Self { client }
    }

    async fn make_request(
        &self,
        method: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        body: Option<&str>,
    ) -> Result<ToolResult> {
        let method = method.to_uppercase();
        let method = match method.as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            _ => return Err(Error::Internal(format!("Unsupported HTTP method: {}", method))),
        };

        let mut request = self.client.request(method, url);

        // Add headers
        if let Some(headers_map) = headers {
            for (key, value) in headers_map {
                request = request.header(key, value);
            }
        }

        // Add body
        if let Some(body_str) = body {
            request = request.body(body_str.to_string());
        }

        let response = request.send().await.map_err(|e| {
            Error::Internal(format!("Failed to send request: {}", e))
        })?;

        let status = response.status().as_u16();
        let headers_map: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.text().await.map_err(|e| {
            Error::Internal(format!("Failed to read response body: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "status": status,
            "headers": headers_map,
            "body": body
        })))
    }
}

impl Default for ApiClientTool {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[async_trait]
impl Tool for ApiClientTool {
    fn name(&self) -> &str {
        "api_client"
    }

    fn description(&self) -> &str {
        "Make HTTP requests (GET, POST, PUT, DELETE, PATCH) with custom headers and body"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter::new(
                "method",
                "HTTP method: GET, POST, PUT, DELETE, PATCH",
                ParameterType::String,
                true,
            ),
            ToolParameter::new("url", "Request URL", ParameterType::String, true),
            ToolParameter::new(
                "headers",
                "Request headers as JSON object",
                ParameterType::Object,
                false,
            ),
            ToolParameter::new(
                "body",
                "Request body (for POST, PUT, PATCH)",
                ParameterType::String,
                false,
            ),
        ]
    }

    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing method parameter".to_string()))?;

        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing url parameter".to_string()))?;

        let headers = params.get("headers").and_then(|v| {
            v.as_object().map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
        });

        let body = params.get("body").and_then(|v| v.as_str());

        self.make_request(method, url, headers.as_ref(), body)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = ApiClientTool::default();
        assert_eq!(tool.name(), "api_client");
        assert!(!tool.description().is_empty());
        assert_eq!(tool.parameters().len(), 4);
    }

    #[tokio::test]
    async fn test_missing_method() {
        let tool = ApiClientTool::default();
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("http://example.com"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_url() {
        let tool = ApiClientTool::default();
        let mut params = HashMap::new();
        params.insert("method".to_string(), json!("GET"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unsupported_method() {
        let tool = ApiClientTool::default();
        let mut params = HashMap::new();
        params.insert("method".to_string(), json!("INVALID"));
        params.insert("url".to_string(), json!("http://example.com"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }
}
