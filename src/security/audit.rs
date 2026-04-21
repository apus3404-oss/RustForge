use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub execution_id: Uuid,
    pub agent_id: String,
    pub action: AuditAction,
    pub result: AuditResult,
}

/// Types of auditable actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    ToolExecuted {
        tool: String,
        operation: String,
        parameters: Value,
    },
    FileAccessed {
        path: PathBuf,
        operation: String,
    },
    NetworkRequest {
        url: String,
        method: String,
    },
    PermissionGranted {
        tool: String,
        scope: String,
    },
    PermissionDenied {
        tool: String,
        scope: String,
        reason: String,
    },
}

/// Result of an audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failed { error: String },
    Denied { reason: String },
}

/// In-memory audit logger (Phase 3 implementation)
/// Future phases will use persistent storage (redb)
pub struct AuditLogger {
    logs: Arc<RwLock<Vec<AuditLog>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an audit event
    pub async fn log(&self, log: AuditLog) -> Result<()> {
        let mut logs = self.logs.write().await;
        logs.push(log);
        Ok(())
    }

    /// Log a tool execution
    pub async fn log_tool_execution(
        &self,
        execution_id: Uuid,
        agent_id: String,
        tool: String,
        operation: String,
        parameters: Value,
        result: AuditResult,
    ) -> Result<()> {
        let log = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            execution_id,
            agent_id,
            action: AuditAction::ToolExecuted {
                tool,
                operation,
                parameters,
            },
            result,
        };
        self.log(log).await
    }

    /// Log a file access
    pub async fn log_file_access(
        &self,
        execution_id: Uuid,
        agent_id: String,
        path: PathBuf,
        operation: String,
        result: AuditResult,
    ) -> Result<()> {
        let log = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            execution_id,
            agent_id,
            action: AuditAction::FileAccessed { path, operation },
            result,
        };
        self.log(log).await
    }

    /// Log a network request
    pub async fn log_network_request(
        &self,
        execution_id: Uuid,
        agent_id: String,
        url: String,
        method: String,
        result: AuditResult,
    ) -> Result<()> {
        let log = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            execution_id,
            agent_id,
            action: AuditAction::NetworkRequest { url, method },
            result,
        };
        self.log(log).await
    }

    /// Log a permission grant
    pub async fn log_permission_granted(
        &self,
        execution_id: Uuid,
        agent_id: String,
        tool: String,
        scope: String,
    ) -> Result<()> {
        let log = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            execution_id,
            agent_id,
            action: AuditAction::PermissionGranted { tool, scope },
            result: AuditResult::Success,
        };
        self.log(log).await
    }

    /// Log a permission denial
    pub async fn log_permission_denied(
        &self,
        execution_id: Uuid,
        agent_id: String,
        tool: String,
        scope: String,
        reason: String,
    ) -> Result<()> {
        let log = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            execution_id,
            agent_id,
            action: AuditAction::PermissionDenied {
                tool,
                scope,
                reason: reason.clone(),
            },
            result: AuditResult::Denied { reason },
        };
        self.log(log).await
    }

    /// Query logs by execution ID
    pub async fn query_by_execution(&self, execution_id: Uuid) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().await;
        Ok(logs
            .iter()
            .filter(|log| log.execution_id == execution_id)
            .cloned()
            .collect())
    }

    /// Query logs by agent ID
    pub async fn query_by_agent(&self, agent_id: &str) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().await;
        Ok(logs
            .iter()
            .filter(|log| log.agent_id == agent_id)
            .cloned()
            .collect())
    }

    /// Query logs by time range
    pub async fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().await;
        Ok(logs
            .iter()
            .filter(|log| log.timestamp >= start && log.timestamp <= end)
            .cloned()
            .collect())
    }

    /// Get all logs
    pub async fn get_all(&self) -> Result<Vec<AuditLog>> {
        let logs = self.logs.read().await;
        Ok(logs.clone())
    }

    /// Get log count
    pub async fn count(&self) -> usize {
        let logs = self.logs.read().await;
        logs.len()
    }

    /// Clear all logs (for testing)
    pub async fn clear(&self) {
        let mut logs = self.logs.write().await;
        logs.clear();
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_log_tool_execution() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        logger
            .log_tool_execution(
                execution_id,
                "agent1".to_string(),
                "file_system".to_string(),
                "read".to_string(),
                json!({"path": "/test.txt"}),
                AuditResult::Success,
            )
            .await
            .unwrap();

        assert_eq!(logger.count().await, 1);

        let logs = logger.query_by_execution(execution_id).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].agent_id, "agent1");
    }

    #[tokio::test]
    async fn test_log_file_access() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        logger
            .log_file_access(
                execution_id,
                "agent1".to_string(),
                PathBuf::from("/test.txt"),
                "read".to_string(),
                AuditResult::Success,
            )
            .await
            .unwrap();

        let logs = logger.get_all().await.unwrap();
        assert_eq!(logs.len(), 1);

        match &logs[0].action {
            AuditAction::FileAccessed { path, operation } => {
                assert_eq!(path, &PathBuf::from("/test.txt"));
                assert_eq!(operation, "read");
            }
            _ => panic!("Expected FileAccessed action"),
        }
    }

    #[tokio::test]
    async fn test_log_network_request() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        logger
            .log_network_request(
                execution_id,
                "agent1".to_string(),
                "https://example.com".to_string(),
                "GET".to_string(),
                AuditResult::Success,
            )
            .await
            .unwrap();

        let logs = logger.get_all().await.unwrap();
        assert_eq!(logs.len(), 1);

        match &logs[0].action {
            AuditAction::NetworkRequest { url, method } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(method, "GET");
            }
            _ => panic!("Expected NetworkRequest action"),
        }
    }

    #[tokio::test]
    async fn test_log_permission_granted() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        logger
            .log_permission_granted(
                execution_id,
                "agent1".to_string(),
                "file_system".to_string(),
                "/home/user".to_string(),
            )
            .await
            .unwrap();

        let logs = logger.get_all().await.unwrap();
        assert_eq!(logs.len(), 1);

        match &logs[0].action {
            AuditAction::PermissionGranted { tool, scope } => {
                assert_eq!(tool, "file_system");
                assert_eq!(scope, "/home/user");
            }
            _ => panic!("Expected PermissionGranted action"),
        }
    }

    #[tokio::test]
    async fn test_log_permission_denied() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        logger
            .log_permission_denied(
                execution_id,
                "agent1".to_string(),
                "shell".to_string(),
                "rm -rf /".to_string(),
                "Dangerous command".to_string(),
            )
            .await
            .unwrap();

        let logs = logger.get_all().await.unwrap();
        assert_eq!(logs.len(), 1);

        match &logs[0].action {
            AuditAction::PermissionDenied { tool, scope, reason } => {
                assert_eq!(tool, "shell");
                assert_eq!(scope, "rm -rf /");
                assert_eq!(reason, "Dangerous command");
            }
            _ => panic!("Expected PermissionDenied action"),
        }
    }

    #[tokio::test]
    async fn test_query_by_agent() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        logger
            .log_tool_execution(
                execution_id,
                "agent1".to_string(),
                "tool1".to_string(),
                "op1".to_string(),
                json!({}),
                AuditResult::Success,
            )
            .await
            .unwrap();

        logger
            .log_tool_execution(
                execution_id,
                "agent2".to_string(),
                "tool2".to_string(),
                "op2".to_string(),
                json!({}),
                AuditResult::Success,
            )
            .await
            .unwrap();

        let agent1_logs = logger.query_by_agent("agent1").await.unwrap();
        assert_eq!(agent1_logs.len(), 1);
        assert_eq!(agent1_logs[0].agent_id, "agent1");

        let agent2_logs = logger.query_by_agent("agent2").await.unwrap();
        assert_eq!(agent2_logs.len(), 1);
        assert_eq!(agent2_logs[0].agent_id, "agent2");
    }

    #[tokio::test]
    async fn test_query_by_time_range() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        let start = Utc::now();

        logger
            .log_tool_execution(
                execution_id,
                "agent1".to_string(),
                "tool1".to_string(),
                "op1".to_string(),
                json!({}),
                AuditResult::Success,
            )
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let end = Utc::now();

        let logs = logger.query_by_time_range(start, end).await.unwrap();
        assert_eq!(logs.len(), 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        logger
            .log_tool_execution(
                execution_id,
                "agent1".to_string(),
                "tool1".to_string(),
                "op1".to_string(),
                json!({}),
                AuditResult::Success,
            )
            .await
            .unwrap();

        assert_eq!(logger.count().await, 1);

        logger.clear().await;
        assert_eq!(logger.count().await, 0);
    }

    #[tokio::test]
    async fn test_audit_result_variants() {
        let logger = AuditLogger::new();
        let execution_id = Uuid::new_v4();

        // Success
        logger
            .log_tool_execution(
                execution_id,
                "agent1".to_string(),
                "tool1".to_string(),
                "op1".to_string(),
                json!({}),
                AuditResult::Success,
            )
            .await
            .unwrap();

        // Failed
        logger
            .log_tool_execution(
                execution_id,
                "agent1".to_string(),
                "tool2".to_string(),
                "op2".to_string(),
                json!({}),
                AuditResult::Failed {
                    error: "Connection timeout".to_string(),
                },
            )
            .await
            .unwrap();

        // Denied
        logger
            .log_tool_execution(
                execution_id,
                "agent1".to_string(),
                "tool3".to_string(),
                "op3".to_string(),
                json!({}),
                AuditResult::Denied {
                    reason: "Insufficient permissions".to_string(),
                },
            )
            .await
            .unwrap();

        let logs = logger.get_all().await.unwrap();
        assert_eq!(logs.len(), 3);

        assert!(matches!(logs[0].result, AuditResult::Success));
        assert!(matches!(logs[1].result, AuditResult::Failed { .. }));
        assert!(matches!(logs[2].result, AuditResult::Denied { .. }));
    }
}
