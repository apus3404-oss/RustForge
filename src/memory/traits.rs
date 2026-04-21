use async_trait::async_trait;
use crate::error::Result;
use crate::llm::Message;

/// Trait for storing and retrieving conversation history
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a message in the conversation history
    async fn store_message(&self, conversation_id: &str, message: Message) -> Result<()>;

    /// Retrieve all messages for a conversation
    async fn get_messages(&self, conversation_id: &str) -> Result<Vec<Message>>;

    /// Retrieve the last N messages for a conversation
    async fn get_recent_messages(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> Result<Vec<Message>>;

    /// Clear all messages for a conversation
    async fn clear_conversation(&self, conversation_id: &str) -> Result<()>;

    /// Get the total number of messages in a conversation
    async fn message_count(&self, conversation_id: &str) -> Result<usize>;
}
