// src/storage/workflow_store.rs
use crate::engine::types::WorkflowDefinition;
use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Manages workflow storage on the file system
pub struct WorkflowStore {
    workflows_dir: PathBuf,
}

impl WorkflowStore {
    /// Create a new workflow store
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let workflows_dir = base_dir.as_ref().join("workflows");

        // Create workflows directory if it doesn't exist
        if !workflows_dir.exists() {
            fs::create_dir_all(&workflows_dir)
                .map_err(|e| Error::Internal(format!("Failed to create workflows directory: {}", e)))?;
        }

        Ok(Self { workflows_dir })
    }

    /// Save a workflow definition
    pub fn save(&self, id: &str, definition: &WorkflowDefinition) -> Result<()> {
        let file_path = self.workflow_path(id);

        let yaml = serde_yaml::to_string(definition)
            .map_err(|e| Error::Internal(format!("Failed to serialize workflow: {}", e)))?;

        fs::write(&file_path, yaml)
            .map_err(|e| Error::Internal(format!("Failed to write workflow file: {}", e)))?;

        Ok(())
    }

    /// Load a workflow definition by ID
    pub fn load(&self, id: &str) -> Result<WorkflowDefinition> {
        let file_path = self.workflow_path(id);

        if !file_path.exists() {
            return Err(Error::WorkflowNotFound {
                workflow_id: id.to_string(),
            });
        }

        let yaml = fs::read_to_string(&file_path)
            .map_err(|e| Error::Internal(format!("Failed to read workflow file: {}", e)))?;

        let definition: WorkflowDefinition = serde_yaml::from_str(&yaml)
            .map_err(|e| Error::Internal(format!("Failed to parse workflow: {}", e)))?;

        Ok(definition)
    }

    /// List all workflow IDs
    pub fn list(&self) -> Result<Vec<String>> {
        let mut workflow_ids = Vec::new();

        let entries = fs::read_dir(&self.workflows_dir)
            .map_err(|e| Error::Internal(format!("Failed to read workflows directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| Error::Internal(format!("Failed to read directory entry: {}", e)))?;

            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    workflow_ids.push(stem.to_string());
                }
            }
        }

        Ok(workflow_ids)
    }

    /// Delete a workflow by ID
    pub fn delete(&self, id: &str) -> Result<()> {
        let file_path = self.workflow_path(id);

        if !file_path.exists() {
            return Err(Error::WorkflowNotFound {
                workflow_id: id.to_string(),
            });
        }

        fs::remove_file(&file_path)
            .map_err(|e| Error::Internal(format!("Failed to delete workflow file: {}", e)))?;

        Ok(())
    }

    /// Check if a workflow exists
    pub fn exists(&self, id: &str) -> bool {
        self.workflow_path(id).exists()
    }

    /// Get the file path for a workflow
    fn workflow_path(&self, id: &str) -> PathBuf {
        self.workflows_dir.join(format!("{}.yaml", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::{AgentConfig, ExecutionMode};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            name: "Test Workflow".to_string(),
            mode: ExecutionMode::Sequential,
            agents: vec![AgentConfig {
                id: "agent1".to_string(),
                agent_type: "base".to_string(),
                task: "Test task".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            }],
            inputs: None,
        }
    }

    #[test]
    fn test_save_and_load_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let store = WorkflowStore::new(temp_dir.path()).unwrap();

        let workflow = create_test_workflow();
        let id = "test-workflow";

        // Save workflow
        store.save(id, &workflow).unwrap();
        assert!(store.exists(id));

        // Load workflow
        let loaded = store.load(id).unwrap();
        assert_eq!(loaded.name, workflow.name);
        assert_eq!(loaded.agents.len(), workflow.agents.len());
    }

    #[test]
    fn test_list_workflows() {
        let temp_dir = TempDir::new().unwrap();
        let store = WorkflowStore::new(temp_dir.path()).unwrap();

        // Save multiple workflows
        let workflow = create_test_workflow();
        store.save("workflow1", &workflow).unwrap();
        store.save("workflow2", &workflow).unwrap();
        store.save("workflow3", &workflow).unwrap();

        // List workflows
        let ids = store.list().unwrap();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"workflow1".to_string()));
        assert!(ids.contains(&"workflow2".to_string()));
        assert!(ids.contains(&"workflow3".to_string()));
    }

    #[test]
    fn test_delete_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let store = WorkflowStore::new(temp_dir.path()).unwrap();

        let workflow = create_test_workflow();
        let id = "test-workflow";

        // Save and delete
        store.save(id, &workflow).unwrap();
        assert!(store.exists(id));

        store.delete(id).unwrap();
        assert!(!store.exists(id));
    }

    #[test]
    fn test_load_nonexistent_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let store = WorkflowStore::new(temp_dir.path()).unwrap();

        let result = store.load("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let store = WorkflowStore::new(temp_dir.path()).unwrap();

        let result = store.delete("nonexistent");
        assert!(result.is_err());
    }
}
