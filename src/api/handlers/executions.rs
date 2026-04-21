use crate::api::{ApiError, AppState};
use crate::engine::types::ExecutionContext;
use crate::engine::WorkflowExecutor;
use crate::storage::StoredExecutionStatus;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Response for workflow execution
#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub execution_id: Uuid,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
}

/// Execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

/// Detailed execution information
#[derive(Debug, Serialize)]
pub struct ExecutionDetails {
    pub id: Uuid,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outputs: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
}

/// Execute a workflow
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(inputs): Json<HashMap<String, serde_json::Value>>,
) -> Result<Json<ExecutionResponse>, ApiError> {
    // Load workflow from file system
    let workflow = state
        .workflow_store
        .load(&workflow_id)
        .map_err(|_| ApiError::not_found("Workflow", &workflow_id))?;

    // Create execution context
    let execution_id = Uuid::new_v4();
    let mut context = ExecutionContext::new(workflow_id.clone());

    // Add inputs to context
    for (key, value) in inputs {
        context.set_value(format!("input.{}", key), value);
    }

    // Create executor
    let executor = WorkflowExecutor::new(
        state.event_bus.clone(),
        state.llm_registry.primary().clone(),
        state.agent_registry.clone(),
    );

    // Spawn execution in background
    let state_clone = state.clone();
    let workflow_id_clone = workflow_id.clone();
    tokio::spawn(async move {
        let mut exec_context = context;
        let _result = executor.execute(&workflow, &mut exec_context).await;
        // TODO: Store execution result in state store
    });

    Ok(Json(ExecutionResponse {
        execution_id,
        status: ExecutionStatus::Running,
        started_at: Utc::now(),
    }))
}

/// Get execution details
pub async fn get_execution(
    State(_state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<ExecutionDetails>, ApiError> {
    // TODO: Implement execution retrieval from state store
    // For now, return not found
    Err(ApiError::not_found("Execution", &execution_id.to_string()))
}

/// List all executions
pub async fn list_executions(
    State(_state): State<AppState>,
) -> Result<Json<Vec<ExecutionDetails>>, ApiError> {
    // TODO: List executions from state store
    Ok(Json(vec![]))
}

/// Pause an execution
pub async fn pause_execution(
    State(_state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // TODO: Implement pause logic
    let _ = execution_id;
    Ok(StatusCode::OK)
}

/// Resume an execution
pub async fn resume_execution(
    State(_state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // TODO: Implement resume logic
    let _ = execution_id;
    Ok(StatusCode::OK)
}

/// Cancel an execution
pub async fn cancel_execution(
    State(_state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // TODO: Implement cancel logic
    let _ = execution_id;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::AgentRegistry;
    use crate::config::GlobalConfig;
    use crate::engine::EventBus;
    use crate::llm::{LLMRegistry, traits::LLMProvider, types::{CompletionOptions, Message}};
    use crate::security::{AuditLogger, PermissionManager};
    use crate::storage::StateStore;
    use crate::tools::ToolRegistry;
    use async_trait::async_trait;
    use std::path::PathBuf;
    use std::sync::Arc;

    struct MockProvider;

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }

        async fn complete(
            &self,
            _messages: Vec<Message>,
            _options: CompletionOptions,
        ) -> crate::error::Result<String> {
            Ok("mock".to_string())
        }

        fn supports_streaming(&self) -> bool {
            false
        }

        fn max_context_tokens(&self) -> usize {
            4096
        }
    }

    fn create_test_state() -> AppState {
        let config = Arc::new(GlobalConfig::default());
        let llm_registry = Arc::new(LLMRegistry::new(Arc::new(MockProvider)));
        let agent_registry = Arc::new(AgentRegistry::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        let permission_manager = Arc::new(PermissionManager::allow_all());
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_exec_{}.db", Uuid::new_v4()));
        let state_store = Arc::new(StateStore::new(&db_path).unwrap());
        let workflow_store = Arc::new(
            crate::storage::WorkflowStore::new(&temp_dir).unwrap()
        );
        let event_bus = Arc::new(EventBus::new());
        let audit_logger = Arc::new(AuditLogger::new());

        AppState::new(
            config,
            llm_registry,
            agent_registry,
            tool_registry,
            permission_manager,
            state_store,
            workflow_store,
            event_bus,
            audit_logger,
        )
    }

    #[tokio::test]
    async fn test_execute_workflow() {
        let state = create_test_state();

        // First create a workflow
        let workflow = crate::engine::types::WorkflowDefinition {
            name: "Test Workflow".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![crate::engine::types::AgentConfig {
                id: "agent1".to_string(),
                agent_type: "base".to_string(),
                task: "Test task".to_string(),
                depends_on: vec![],
                config: HashMap::new(),
            }],
            inputs: None,
        };

        let workflow_id = "test-workflow-exec";
        state.workflow_store.save(workflow_id, &workflow).unwrap();

        // Now execute it
        let inputs = HashMap::new();
        let result = execute_workflow(
            State(state),
            Path(workflow_id.to_string()),
            Json(inputs),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.status, ExecutionStatus::Running);
    }

    #[tokio::test]
    async fn test_get_execution_not_found() {
        let state = create_test_state();
        let execution_id = Uuid::new_v4();

        let result = get_execution(State(state), Path(execution_id)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_executions() {
        let state = create_test_state();
        let result = list_executions(State(state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 0);
    }

    #[tokio::test]
    async fn test_pause_execution() {
        let state = create_test_state();
        let execution_id = Uuid::new_v4();

        let result = pause_execution(State(state), Path(execution_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_resume_execution() {
        let state = create_test_state();
        let execution_id = Uuid::new_v4();

        let result = resume_execution(State(state), Path(execution_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cancel_execution() {
        let state = create_test_state();
        let execution_id = Uuid::new_v4();

        let result = cancel_execution(State(state), Path(execution_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }
}
