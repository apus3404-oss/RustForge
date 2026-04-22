// src/api/execution_registry.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::engine::CancellationToken;

/// Registry for tracking active executions
#[derive(Clone)]
pub struct ExecutionRegistry {
    executions: Arc<RwLock<HashMap<Uuid, ExecutionHandle>>>,
}

/// Handle for a running execution
pub struct ExecutionHandle {
    pub workflow_id: String,
    pub cancellation_token: CancellationToken,
}

impl ExecutionRegistry {
    /// Create a new execution registry
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new execution
    pub async fn register(&self, execution_id: Uuid, workflow_id: String, token: CancellationToken) {
        let mut executions = self.executions.write().await;
        executions.insert(execution_id, ExecutionHandle {
            workflow_id,
            cancellation_token: token,
        });
    }

    /// Unregister an execution
    pub async fn unregister(&self, execution_id: &Uuid) {
        let mut executions = self.executions.write().await;
        executions.remove(execution_id);
    }

    /// Get cancellation token for an execution
    pub async fn get_token(&self, execution_id: &Uuid) -> Option<CancellationToken> {
        let executions = self.executions.read().await;
        executions.get(execution_id).map(|h| h.cancellation_token.clone())
    }

    /// Cancel an execution
    pub async fn cancel(&self, execution_id: &Uuid) -> bool {
        if let Some(token) = self.get_token(execution_id).await {
            token.cancel();
            true
        } else {
            false
        }
    }

    /// Check if execution exists
    pub async fn exists(&self, execution_id: &Uuid) -> bool {
        let executions = self.executions.read().await;
        executions.contains_key(execution_id)
    }
}

impl Default for ExecutionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_unregister() {
        let registry = ExecutionRegistry::new();
        let execution_id = Uuid::new_v4();
        let token = CancellationToken::new();

        registry.register(execution_id, "test-workflow".to_string(), token).await;
        assert!(registry.exists(&execution_id).await);

        registry.unregister(&execution_id).await;
        assert!(!registry.exists(&execution_id).await);
    }

    #[tokio::test]
    async fn test_cancel_execution() {
        let registry = ExecutionRegistry::new();
        let execution_id = Uuid::new_v4();
        let token = CancellationToken::new();

        registry.register(execution_id, "test-workflow".to_string(), token.clone()).await;

        assert!(!token.is_cancelled());
        registry.cancel(&execution_id).await;
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancel_nonexistent() {
        let registry = ExecutionRegistry::new();
        let execution_id = Uuid::new_v4();

        let result = registry.cancel(&execution_id).await;
        assert!(!result);
    }
}
