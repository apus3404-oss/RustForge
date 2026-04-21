use async_trait::async_trait;
use crate::error::Result;
use super::types::{CompletionOptions, Message};

/// Trait for LLM providers (Ollama, OpenAI, etc.)
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a completion from the given messages
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<String>;

    /// Check if this provider supports streaming responses
    fn supports_streaming(&self) -> bool;

    /// Get the maximum context window size in tokens
    fn max_context_tokens(&self) -> usize;

    /// Get the provider name for logging/debugging
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::types::{Message, MessageRole};

    struct MockProvider;

    #[async_trait]
    impl LLMProvider for MockProvider {
        async fn complete(
            &self,
            messages: Vec<Message>,
            _options: CompletionOptions,
        ) -> Result<String> {
            Ok(format!("Mock response to {} messages", messages.len()))
        }

        fn supports_streaming(&self) -> bool {
            false
        }

        fn max_context_tokens(&self) -> usize {
            4096
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider;
        let messages = vec![Message::user("Hello")];
        let result = provider
            .complete(messages, CompletionOptions::default())
            .await
            .unwrap();
        assert_eq!(result, "Mock response to 1 messages");
    }

    #[test]
    fn test_provider_metadata() {
        let provider = MockProvider;
        assert_eq!(provider.name(), "mock");
        assert_eq!(provider.max_context_tokens(), 4096);
        assert!(!provider.supports_streaming());
    }
}
