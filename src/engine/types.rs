// src/engine/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub mode: ExecutionMode,
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub inputs: Option<Vec<InputDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub task: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Sequential,
    Parallel,
    Dag,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub workflow_id: String,
    pub execution_id: Uuid,
    pub context_store: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            execution_id: Uuid::new_v4(),
            context_store: HashMap::new(),
        }
    }

    pub fn set_value(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.context_store.insert(key.into(), value);
    }

    pub fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.context_store.get(key)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub agent_id: String,
    pub status: ExecutionStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_definition_deserialization() {
        let yaml = r#"
name: "Test Workflow"
mode: sequential
agents:
  - id: agent1
    type: TestAgent
    task: "Do something"
"#;
        let workflow: WorkflowDefinition = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.agents.len(), 1);
        assert_eq!(workflow.agents[0].id, "agent1");
    }

    #[test]
    fn test_execution_context_creation() {
        let context = ExecutionContext::new("test-workflow".to_string());
        assert_eq!(context.workflow_id, "test-workflow");
        assert!(context.context_store.is_empty());
    }

    #[test]
    fn test_execution_context_store_value() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("key1", serde_json::json!("value1"));

        let value = context.get_value("key1").unwrap();
        assert_eq!(value, &serde_json::json!("value1"));
    }

    #[test]
    fn test_execution_status_serialization() {
        let status = ExecutionStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Running"));

        let deserialized: ExecutionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_execution_mode_deserialization() {
        let yaml = "sequential";
        let mode: ExecutionMode = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(mode, ExecutionMode::Sequential);
    }
}
