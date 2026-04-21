use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::llm::Message;
use crate::memory::traits::MemoryStore;
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;

const MESSAGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("messages");

/// Simple memory store using redb for persistence
pub struct SimpleMemoryStore {
    db: Database,
}

impl SimpleMemoryStore {
    /// Create a new memory store at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::create(path)?;

        // Initialize table
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(MESSAGES_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db })
    }

    fn make_key(conversation_id: &str, index: usize) -> String {
        format!("{}:{:010}", conversation_id, index)
    }

    fn parse_key(key: &str) -> Option<(&str, usize)> {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() == 2 {
            let conversation_id = parts[0];
            let index = parts[1].parse().ok()?;
            Some((conversation_id, index))
        } else {
            None
        }
    }
}

#[async_trait]
impl MemoryStore for SimpleMemoryStore {
    async fn store_message(&self, conversation_id: &str, message: Message) -> Result<()> {
        let count = self.message_count(conversation_id).await?;
        let key = Self::make_key(conversation_id, count);
        let serialized = bincode::serialize(&message)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MESSAGES_TABLE)?;
            table.insert(key.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    async fn get_messages(&self, conversation_id: &str) -> Result<Vec<Message>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(MESSAGES_TABLE)?;

        let prefix = format!("{}:", conversation_id);
        let mut messages = Vec::new();

        for result in table.iter()? {
            let (key, value) = result?;
            let key_str = key.value();

            if key_str.starts_with(&prefix) {
                let bytes = value.value();
                let message: Message = bincode::deserialize(bytes)?;
                messages.push(message);
            }
        }

        Ok(messages)
    }

    async fn get_recent_messages(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> Result<Vec<Message>> {
        let all_messages = self.get_messages(conversation_id).await?;
        let start = all_messages.len().saturating_sub(limit);
        Ok(all_messages[start..].to_vec())
    }

    async fn clear_conversation(&self, conversation_id: &str) -> Result<()> {
        let prefix = format!("{}:", conversation_id);
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MESSAGES_TABLE)?;

            // Collect keys to delete
            let keys_to_delete: Vec<String> = {
                let read_txn = self.db.begin_read()?;
                let read_table = read_txn.open_table(MESSAGES_TABLE)?;
                read_table
                    .iter()?
                    .filter_map(|result| {
                        let (key, _) = result.ok()?;
                        let key_str = key.value();
                        if key_str.starts_with(&prefix) {
                            Some(key_str.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            // Delete collected keys
            for key in keys_to_delete {
                table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    async fn message_count(&self, conversation_id: &str) -> Result<usize> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(MESSAGES_TABLE)?;

        let prefix = format!("{}:", conversation_id);
        let count = table
            .iter()?
            .filter(|result| {
                if let Ok((key, _)) = result {
                    key.value().starts_with(&prefix)
                } else {
                    false
                }
            })
            .count();

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MessageRole;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_store_and_retrieve_messages() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("memory.db");
        let store = SimpleMemoryStore::new(&db_path).unwrap();

        let msg1 = Message::user("Hello");
        let msg2 = Message::assistant("Hi there!");

        store.store_message("conv1", msg1.clone()).await.unwrap();
        store.store_message("conv1", msg2.clone()).await.unwrap();

        let messages = store.get_messages("conv1").await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_get_recent_messages() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("memory.db");
        let store = SimpleMemoryStore::new(&db_path).unwrap();

        for i in 0..5 {
            let msg = Message::user(format!("Message {}", i));
            store.store_message("conv1", msg).await.unwrap();
        }

        let recent = store.get_recent_messages("conv1", 2).await.unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].content, "Message 3");
        assert_eq!(recent[1].content, "Message 4");
    }

    #[tokio::test]
    async fn test_message_count() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("memory.db");
        let store = SimpleMemoryStore::new(&db_path).unwrap();

        assert_eq!(store.message_count("conv1").await.unwrap(), 0);

        store.store_message("conv1", Message::user("Test")).await.unwrap();
        assert_eq!(store.message_count("conv1").await.unwrap(), 1);

        store.store_message("conv1", Message::user("Test 2")).await.unwrap();
        assert_eq!(store.message_count("conv1").await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_clear_conversation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("memory.db");
        let store = SimpleMemoryStore::new(&db_path).unwrap();

        store.store_message("conv1", Message::user("Test")).await.unwrap();
        store.store_message("conv1", Message::user("Test 2")).await.unwrap();

        assert_eq!(store.message_count("conv1").await.unwrap(), 2);

        store.clear_conversation("conv1").await.unwrap();
        assert_eq!(store.message_count("conv1").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_multiple_conversations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("memory.db");
        let store = SimpleMemoryStore::new(&db_path).unwrap();

        store.store_message("conv1", Message::user("Conv1 Msg1")).await.unwrap();
        store.store_message("conv2", Message::user("Conv2 Msg1")).await.unwrap();
        store.store_message("conv1", Message::user("Conv1 Msg2")).await.unwrap();

        let conv1_messages = store.get_messages("conv1").await.unwrap();
        let conv2_messages = store.get_messages("conv2").await.unwrap();

        assert_eq!(conv1_messages.len(), 2);
        assert_eq!(conv2_messages.len(), 1);
        assert_eq!(conv1_messages[0].content, "Conv1 Msg1");
        assert_eq!(conv2_messages[0].content, "Conv2 Msg1");
    }
}
