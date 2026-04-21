use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::llm::traits::LLMProvider;
use crate::llm::types::{CompletionOptions, Message, MessageRole};
use reqwest::Client;
use serde_json::json;

pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: Client,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            client: Client::new(),
        }
    }

    fn format_messages(&self, messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::System => "System",
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<String> {
        let prompt = self.format_messages(&messages);

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&json!({
                "model": self.model,
                "prompt": prompt,
                "temperature": options.temperature,
                "stream": false,
                "options": {
                    "num_predict": options.max_tokens,
                }
            }))
            .send()
            .await
            .map_err(|e| Error::Internal(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Internal(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse Ollama response: {}", e)))?;

        body["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Internal("Missing 'response' field in Ollama output".into()))
    }

    fn supports_streaming(&self) -> bool {
        false // Streaming not implemented in Phase 2
    }

    fn max_context_tokens(&self) -> usize {
        4096 // Default for most Ollama models
    }

    fn name(&self) -> &str {
        "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Mock, Server};

    #[tokio::test]
    async fn test_ollama_complete_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"response": "Hello! How can I help you?"}"#)
            .create_async()
            .await;

        let provider = OllamaProvider::new(server.url(), "llama2");
        let messages = vec![Message::user("Hello")];
        let result = provider
            .complete(messages, CompletionOptions::default())
            .await
            .unwrap();

        assert_eq!(result, "Hello! How can I help you?");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_ollama_complete_error() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/api/generate")
            .with_status(500)
            .with_body("Internal server error")
            .create_async()
            .await;

        let provider = OllamaProvider::new(server.url(), "llama2");
        let messages = vec![Message::user("Hello")];
        let result = provider
            .complete(messages, CompletionOptions::default())
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_format_messages() {
        let provider = OllamaProvider::new("http://localhost:11434", "llama2");
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let formatted = provider.format_messages(&messages);
        assert!(formatted.contains("System: You are helpful"));
        assert!(formatted.contains("User: Hello"));
        assert!(formatted.contains("Assistant: Hi there!"));
    }

    #[test]
    fn test_provider_metadata() {
        let provider = OllamaProvider::new("http://localhost:11434", "llama2");
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.max_context_tokens(), 4096);
        assert!(provider.supports_streaming());
    }
}
