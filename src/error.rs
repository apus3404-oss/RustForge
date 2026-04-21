use thiserror::Error;
use std::time::Duration;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    // Workflow errors
    #[error("Workflow '{workflow_id}' not found")]
    WorkflowNotFound { workflow_id: String },

    #[error("Invalid workflow definition: {reason}")]
    InvalidWorkflowDefinition { reason: String },

    #[error("Variable '{variable}' not found. Did you mean: {suggestions:?}")]
    VariableNotFound {
        variable: String,
        suggestions: Vec<String>,
    },

    #[error("Circular dependency detected: {agents:?}")]
    CircularDependency { agents: Vec<String> },

    // Execution errors
    #[error("Execution '{execution_id}' not found")]
    ExecutionNotFound { execution_id: Uuid },

    #[error("Execution timeout after {timeout:?}")]
    ExecutionTimeout { timeout: Duration },

    // Storage errors
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Config errors
    #[error("Config error: {0}")]
    Config(String),

    // Generic
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

// redb error conversions
impl From<redb::DatabaseError> for Error {
    fn from(err: redb::DatabaseError) -> Self {
        Error::Storage(err.to_string())
    }
}

impl From<redb::TransactionError> for Error {
    fn from(err: redb::TransactionError) -> Self {
        Error::Storage(err.to_string())
    }
}

impl From<redb::TableError> for Error {
    fn from(err: redb::TableError) -> Self {
        Error::Storage(err.to_string())
    }
}

impl From<redb::CommitError> for Error {
    fn from(err: redb::CommitError) -> Self {
        Error::Storage(err.to_string())
    }
}

impl From<redb::StorageError> for Error {
    fn from(err: redb::StorageError) -> Self {
        Error::Storage(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_error_display() {
        let err = Error::WorkflowNotFound {
            workflow_id: "test-workflow".to_string(),
        };
        assert!(err.to_string().contains("test-workflow"));
    }

    #[test]
    fn test_variable_not_found_with_suggestions() {
        let err = Error::VariableNotFound {
            variable: "pdf_reeder".to_string(),
            suggestions: vec!["pdf_reader".to_string()],
        };
        let msg = err.to_string();
        assert!(msg.contains("pdf_reeder"));
        assert!(msg.contains("pdf_reader"));
    }

    #[test]
    fn test_execution_not_found() {
        let id = Uuid::new_v4();
        let err = Error::ExecutionNotFound { execution_id: id };
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn test_circular_dependency() {
        let err = Error::CircularDependency {
            agents: vec!["agent1".to_string(), "agent2".to_string()],
        };
        let msg = err.to_string();
        assert!(msg.contains("agent1"));
        assert!(msg.contains("agent2"));
    }

    #[test]
    fn test_execution_timeout() {
        let err = Error::ExecutionTimeout {
            timeout: Duration::from_secs(30),
        };
        assert!(err.to_string().contains("30"));
    }

    #[test]
    fn test_serialization_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }
}
