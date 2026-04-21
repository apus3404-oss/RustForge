use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::llm::traits::LLMProvider;
use crate::llm::types::{CompletionOptions, Message};
use std::sync::Arc;
use tracing::{debug, warn};

/// LLM Registry with automatic fallback support
pub struct LLMRegistry {
    primary: Arc<dyn LLMProvider>,
    fallback: Option<Arc<dyn LLMProvider>>,
}

impl LLMRegistry {
    /// Create a new registry with a primary provider
    pub fn new(primary: Arc<dyn LLMProvider>) -> Self {
        Self {
            primary,
            fallback: None,
        }
    }

    /// Create a registry with primary and fallback providers
    pub fn with_fallback(
        primary: Arc<dyn LLMProvider>,
        fallback: Arc<dyn LLMProvider>,
    ) -> Self {
        Self {
            primary,
            fallback: Some(fallback),
        }
    }

    /// Get the primary provider
    pub fn primary(&self) -> &Arc<dyn LLMProvider> {
        &self.primary
    }

    /// Get the fallback provider if configured
    pub fn fallback(&self) -> Option<&Arc<dyn LLMProvider>> {
        self.fallback.as_ref()
    }
}

#[async_trait]
impl LLMProvider for LLMRegistry {
    async fn complete(
        &self,
        messages: Vec<Message>,
        options: CompletionOptions,
    ) -> Result<String> {
        debug!(
            "Attempting completion with primary provider: {}",
            self.primary.name()
        );

        match self.primary.complete(messages.clone(), options.clone()).await {
            Ok(response) => {
                debug!("Primary provider succeeded");
                Ok(response)
            }
            Err(primary_error) => {
                warn!(
                    "Primary provider {} failed: {}",
                    self.primary.name(),
                    primary_error
                );

                if let Some(fallback) = &self.fallback {
                    debug!(
                        "Attempting fallback provider: {}",
                        fallback.name()
                    );

                    match fallback.complete(messages, options).await {
                        Ok(response) => {
                            debug!("Fallback provider succeeded");
                            Ok(response)
                        }
                        Err(fallback_error) => {
                            warn!(
                                "Fallback provider {} also failed: {}",
                                fallback.name(),
                                fallback_error
                            );
                            Err(Error::Internal(format!(
                                "Both primary ({}) and fallback ({}) providers failed. Primary: {}. Fallback: {}",
                                self.primary.name(),
                                fallback.name(),
                                primary_error,
                                fallback_error
                            )))
                        }
                    }
                } else {
                    Err(primary_error)
                }
            }
        }
    }

    fn supports_streaming(&self) -> bool {
        self.primary.supports_streaming()
    }

    fn max_context_tokens(&self) -> usize {
        self.primary.max_context_tokens()
    }

    fn name(&self) -> &str {
        "registry"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::types::MessageRole;

    struct MockProvider {
        name: String,
        should_fail: bool,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        async fn complete(
            &self,
            _messages: Vec<Message>,
            _options: CompletionOptions,
        ) -> Result<String> {
            if self.should_fail {
                Err(Error::Internal(format!("{} failed", self.name)))
            } else {
                Ok(format!("Response from {}", self.name))
            }
        }

        fn supports_streaming(&self) -> bool {
            false
        }

        fn max_context_tokens(&self) -> usize {
            4096
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_primary_succeeds() {
        let primary = Arc::new(MockProvider {
            name: "primary".to_string(),
            should_fail: false,
        });
        let registry = LLMRegistry::new(primary);

        let messages = vec![Message::user("Hello")];
        let result = registry
            .complete(messages, CompletionOptions::default())
            .await
            .unwrap();

        assert_eq!(result, "Response from primary");
    }

    #[tokio::test]
    async fn test_fallback_on_primary_failure() {
        let primary = Arc::new(MockProvider {
            name: "primary".to_string(),
            should_fail: true,
        });
        let fallback = Arc::new(MockProvider {
            name: "fallback".to_string(),
            should_fail: false,
        });
        let registry = LLMRegistry::with_fallback(primary, fallback);

        let messages = vec![Message::user("Hello")];
        let result = registry
            .complete(messages, CompletionOptions::default())
            .await
            .unwrap();

        assert_eq!(result, "Response from fallback");
    }

    #[tokio::test]
    async fn test_both_providers_fail() {
        let primary = Arc::new(MockProvider {
            name: "primary".to_string(),
            should_fail: true,
        });
        let fallback = Arc::new(MockProvider {
            name: "fallback".to_string(),
            should_fail: true,
        });
        let registry = LLMRegistry::with_fallback(primary, fallback);

        let messages = vec![Message::user("Hello")];
        let result = registry
            .complete(messages, CompletionOptions::default())
            .await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Both primary"));
        assert!(error_msg.contains("fallback"));
    }

    #[tokio::test]
    async fn test_no_fallback_configured() {
        let primary = Arc::new(MockProvider {
            name: "primary".to_string(),
            should_fail: true,
        });
        let registry = LLMRegistry::new(primary);

        let messages = vec![Message::user("Hello")];
        let result = registry
            .complete(messages, CompletionOptions::default())
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("primary failed"));
    }

    #[test]
    fn test_registry_metadata() {
        let primary = Arc::new(MockProvider {
            name: "primary".to_string(),
            should_fail: false,
        });
        let registry = LLMRegistry::new(primary);

        assert_eq!(registry.name(), "registry");
        assert_eq!(registry.max_context_tokens(), 4096);
        assert!(!registry.supports_streaming());
    }

    #[test]
    fn test_registry_accessors() {
        let primary = Arc::new(MockProvider {
            name: "primary".to_string(),
            should_fail: false,
        });
        let fallback = Arc::new(MockProvider {
            name: "fallback".to_string(),
            should_fail: false,
        });
        let registry = LLMRegistry::with_fallback(primary.clone(), fallback.clone());

        assert_eq!(registry.primary().name(), "primary");
        assert_eq!(registry.fallback().unwrap().name(), "fallback");
    }
}
