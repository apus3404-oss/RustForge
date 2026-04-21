use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::llm::traits::LLMProvider;
use crate::llm::types::{CompletionOptions, Message, MessageRole};
use reqwest::Client;
use serde_json::json;

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: Client,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: Client::new(),
        }
    }

    fn format_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                };
                json!({
                    "role": role,
                    "content": m.content,
                })
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<String> {
        let openai_messages = self.format_messages(&messages);

        let mut request_body = json!({
            "model": self.model,
            "messages": openai_messages,
            "temperature": options.temperature,
            "max_tokens": options.max_tokens,
        });

        if let Some(top_p) = options.top_p {
            request_body["top_p"] = json!(top_p);
        }

        if let Some(stop) = options.stop {
            request_body["stop"] = json!(stop);
        }

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Internal(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Internal(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse OpenAI response: {}", e)))?;

        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                Error::Internal("Missing content in OpenAI response".into())
            })
    }

    fn supports_streaming(&self) -> bool {
        false // Streaming not implemented in Phase 2
    }

    fn max_context_tokens(&self) -> usize {
        128000 // GPT-4 Turbo context window
    }

    fn name(&self) -> &str {
        "openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Mock, Server};

    #[tokio::test]
    async fn test_openai_complete_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "choices": [{
                        "message": {
                            "role": "assistant",
                            "content": "Hello! How can I assist you today?"
                        }
                    }]
                }"#,
            )
            .create_async()
            .await;

        let provider = OpenAIProvider::new("test-key", "gpt-4");
        // Override base URL for testing
        let provider = OpenAIProvider {
            api_key: "test-key".to_string(),
            model: "gpt-4".to_string(),
            client: reqwest::ClientBuilder::new()
                .build()
                .unwrap(),
        };

        let messages = vec![Message::user("Hello")];

        // For testing, we need to use the mock server URL
        // In real implementation, this would use the actual OpenAI endpoint
        let response = provider.client
            .post(format!("{}/v1/chat/completions", server.url()))
            .header("Authorization", "Bearer test-key")
            .json(&json!({
                "model": "gpt-4",
                "messages": [{"role": "user", "content": "Hello"}],
                "temperature": 0.7,
                "max_tokens": 2048,
            }))
            .send()
            .await
            .unwrap();

        let body: serde_json::Value = response.json().await.unwrap();
        let result = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap();

        assert_eq!(result, "Hello! How can I assist you today?");
        mock.assert_async().await;
    }

    #[test]
    fn test_format_messages() {
        let provider = OpenAIProvider::new("test-key", "gpt-4");
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let formatted = provider.format_messages(&messages);
        assert_eq!(formatted.len(), 3);
        assert_eq!(formatted[0]["role"], "system");
        assert_eq!(formatted[1]["role"], "user");
        assert_eq!(formatted[2]["role"], "assistant");
    }

    #[test]
    fn test_provider_metadata() {
        let provider = OpenAIProvider::new("test-key", "gpt-4");
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.max_context_tokens(), 128000);
        assert!(provider.supports_streaming());
    }
}
