use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Definition of an agent's configuration and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub id: String,
    pub agent_type: String,
    pub name: String,
    pub description: Option<String>,
    pub config: HashMap<String, Value>,
}

impl AgentDefinition {
    pub fn new(id: impl Into<String>, agent_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            agent_type: agent_type.into(),
            name: String::new(),
            description: None,
            config: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_config(mut self, key: impl Into<String>, value: Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }
}

/// A task to be executed by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub context: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            context: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }
}

/// Output produced by an agent after executing a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub agent_id: String,
    pub task_id: String,
    pub status: AgentStatus,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub completed_at: DateTime<Utc>,
}

impl AgentOutput {
    pub fn success(
        agent_id: impl Into<String>,
        task_id: impl Into<String>,
        result: Value,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            task_id: task_id.into(),
            status: AgentStatus::Completed,
            result: Some(result),
            error: None,
            metadata: HashMap::new(),
            completed_at: Utc::now(),
        }
    }

    pub fn failure(
        agent_id: impl Into<String>,
        task_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            task_id: task_id.into(),
            status: AgentStatus::Failed,
            result: None,
            error: Some(error.into()),
            metadata: HashMap::new(),
            completed_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_definition_builder() {
        let agent = AgentDefinition::new("agent1", "research")
            .with_name("Research Agent")
            .with_description("Performs research tasks")
            .with_config("max_results", json!(10));

        assert_eq!(agent.id, "agent1");
        assert_eq!(agent.agent_type, "research");
        assert_eq!(agent.name, "Research Agent");
        assert_eq!(agent.description, Some("Performs research tasks".to_string()));
        assert_eq!(agent.config.get("max_results"), Some(&json!(10)));
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new("task1", "Analyze data")
            .with_context("dataset", json!("sales.csv"));

        assert_eq!(task.id, "task1");
        assert_eq!(task.description, "Analyze data");
        assert_eq!(task.context.get("dataset"), Some(&json!("sales.csv")));
    }

    #[test]
    fn test_agent_output_success() {
        let output = AgentOutput::success("agent1", "task1", json!({"result": "done"}))
            .with_metadata("duration_ms", json!(1500));

        assert_eq!(output.agent_id, "agent1");
        assert_eq!(output.task_id, "task1");
        assert_eq!(output.status, AgentStatus::Completed);
        assert!(output.result.is_some());
        assert!(output.error.is_none());
        assert_eq!(output.metadata.get("duration_ms"), Some(&json!(1500)));
    }

    #[test]
    fn test_agent_output_failure() {
        let output = AgentOutput::failure("agent1", "task1", "Connection timeout");

        assert_eq!(output.status, AgentStatus::Failed);
        assert!(output.result.is_none());
        assert_eq!(output.error, Some("Connection timeout".to_string()));
    }

    #[test]
    fn test_agent_status_serialization() {
        let status = AgentStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");
    }
}
