use crate::error::{Error, Result};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;

// Storage-layer types (simplified for persistence)
// These are separate from engine::types domain models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredExecution {
    pub id: String,
    pub status: StoredExecutionStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Checkpoint {
    pub execution_id: String,
    pub step_id: String,
    pub timestamp: u64,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoredExecutionStatus {
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

// Define redb table schemas
const EXECUTIONS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("executions");
const CHECKPOINTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("checkpoints");

pub struct StateStore {
    db: Database,
}

impl StateStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::create(path)?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(EXECUTIONS_TABLE)?;
            let _ = write_txn.open_table(CHECKPOINTS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db })
    }

    pub fn save_execution(&self, execution: &StoredExecution) -> Result<()> {
        let serialized = bincode::serialize(execution)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(EXECUTIONS_TABLE)?;
            table.insert(execution.id.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn get_execution(&self, id: &str) -> Result<Option<StoredExecution>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EXECUTIONS_TABLE)?;

        match table.get(id)? {
            Some(value) => {
                let bytes = value.value();
                let execution: StoredExecution = bincode::deserialize(bytes)?;
                Ok(Some(execution))
            }
            None => Ok(None),
        }
    }

    pub fn list_executions(&self) -> Result<Vec<StoredExecution>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EXECUTIONS_TABLE)?;

        let mut executions = Vec::new();
        for result in table.iter()? {
            let (_key, value) = result?;
            let bytes = value.value();
            let execution: StoredExecution = bincode::deserialize(bytes)?;
            executions.push(execution);
        }

        Ok(executions)
    }

    pub fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let serialized = bincode::serialize(checkpoint)?;

        // Use composite key: execution_id:timestamp for ordering
        let key = format!("{}:{:020}", checkpoint.execution_id, checkpoint.timestamp);

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CHECKPOINTS_TABLE)?;
            table.insert(key.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn get_latest_checkpoint(&self, execution_id: &str) -> Result<Option<Checkpoint>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CHECKPOINTS_TABLE)?;

        // Find the latest checkpoint by scanning keys with matching execution_id prefix
        let prefix = format!("{}:", execution_id);
        let mut latest: Option<Checkpoint> = None;

        for result in table.iter()? {
            let (key, value) = result?;
            let key_str = key.value();

            if key_str.starts_with(&prefix) {
                let bytes = value.value();
                let checkpoint: Checkpoint = bincode::deserialize(bytes)?;

                match &latest {
                    None => latest = Some(checkpoint),
                    Some(current) => {
                        if checkpoint.timestamp > current.timestamp {
                            latest = Some(checkpoint);
                        }
                    }
                }
            }
        }

        Ok(latest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_state_store() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("state.db");

        let store = StateStore::new(&db_path);
        assert!(store.is_ok());
    }

    #[test]
    fn test_save_and_get_execution() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("state.db");
        let store = StateStore::new(&db_path).unwrap();

        let execution = StoredExecution {
            id: "exec-123".to_string(),
            status: StoredExecutionStatus::Running,
            created_at: 1000,
            updated_at: 1000,
            data: vec![1, 2, 3, 4],
        };

        // Save execution
        store.save_execution(&execution).unwrap();

        // Retrieve execution
        let retrieved = store.get_execution("exec-123").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), execution);
    }

    #[test]
    fn test_get_nonexistent_execution() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("state.db");
        let store = StateStore::new(&db_path).unwrap();

        let result = store.get_execution("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_execution() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("state.db");
        let store = StateStore::new(&db_path).unwrap();

        let mut execution = StoredExecution {
            id: "exec-456".to_string(),
            status: StoredExecutionStatus::Running,
            created_at: 1000,
            updated_at: 1000,
            data: vec![1, 2, 3],
        };

        store.save_execution(&execution).unwrap();

        // Update execution
        execution.status = StoredExecutionStatus::Completed;
        execution.updated_at = 2000;
        store.save_execution(&execution).unwrap();

        // Verify update
        let retrieved = store.get_execution("exec-456").unwrap().unwrap();
        assert_eq!(retrieved.status, StoredExecutionStatus::Completed);
        assert_eq!(retrieved.updated_at, 2000);
    }

    #[test]
    fn test_save_and_get_checkpoint() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("state.db");
        let store = StateStore::new(&db_path).unwrap();

        let checkpoint = Checkpoint {
            execution_id: "exec-789".to_string(),
            step_id: "step-1".to_string(),
            timestamp: 5000,
            state: vec![10, 20, 30],
        };

        store.save_checkpoint(&checkpoint).unwrap();

        let retrieved = store.get_latest_checkpoint("exec-789").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), checkpoint);
    }

    #[test]
    fn test_get_latest_checkpoint_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("state.db");
        let store = StateStore::new(&db_path).unwrap();

        // Save multiple checkpoints for same execution
        let checkpoint1 = Checkpoint {
            execution_id: "exec-999".to_string(),
            step_id: "step-1".to_string(),
            timestamp: 1000,
            state: vec![1],
        };

        let checkpoint2 = Checkpoint {
            execution_id: "exec-999".to_string(),
            step_id: "step-2".to_string(),
            timestamp: 2000,
            state: vec![2],
        };

        let checkpoint3 = Checkpoint {
            execution_id: "exec-999".to_string(),
            step_id: "step-3".to_string(),
            timestamp: 3000,
            state: vec![3],
        };

        store.save_checkpoint(&checkpoint1).unwrap();
        store.save_checkpoint(&checkpoint2).unwrap();
        store.save_checkpoint(&checkpoint3).unwrap();

        // Should return the latest checkpoint
        let latest = store.get_latest_checkpoint("exec-999").unwrap().unwrap();
        assert_eq!(latest.timestamp, 3000);
        assert_eq!(latest.step_id, "step-3");
    }

    #[test]
    fn test_get_checkpoint_nonexistent_execution() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("state.db");
        let store = StateStore::new(&db_path).unwrap();

        let result = store.get_latest_checkpoint("nonexistent").unwrap();
        assert!(result.is_none());
    }
}
