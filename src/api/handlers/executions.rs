use crate::api::{ApiError, AppState};
use crate::engine::types::ExecutionContext;
use crate::engine::{CancellationToken, WorkflowExecutor};
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
    State(state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<ExecutionDetails>, ApiError> {
    // Load execution from state store
    let stored = state
        .state_store
        .get_execution(&execution_id.to_string())
        .map_err(|_| ApiError::not_found("Execution", &execution_id.to_string()))?
        .ok_or_else(|| ApiError::not_found("Execution", &execution_id.to_string()))?;

    // Deserialize data field from Vec<u8> to JSON
    let data: serde_json::Value = serde_json::from_slice(&stored.data)
        .unwrap_or(serde_json::json!({}));

    // Parse data field to extract execution details
    let workflow_id = data
        .get("workflow_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let started_at = data
        .get("started_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| {
            chrono::DateTime::from_timestamp(stored.created_at as i64, 0)
                .unwrap_or_else(|| Utc::now())
        });

    let completed_at = data
        .get("completed_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let outputs = data
        .get("result")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    let error = data
        .get("error")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let status = match stored.status {
        StoredExecutionStatus::Running => ExecutionStatus::Running,
        StoredExecutionStatus::Completed => ExecutionStatus::Completed,
        StoredExecutionStatus::Failed => ExecutionStatus::Failed,
        StoredExecutionStatus::Paused => ExecutionStatus::Paused,
        StoredExecutionStatus::Cancelled => ExecutionStatus::Cancelled,
    };

    Ok(Json(ExecutionDetails {
        id: execution_id,
        workflow_id,
        status,
        started_at,
        completed_at,
        outputs,
        error,
    }))
}

/// List all executions
pub async fn list_executions(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExecutionDetails>>, ApiError> {
    // List all executions from state store
    let stored_executions = state
        .state_store
        .list_executions()
        .map_err(|e| ApiError::internal_error(format!("Failed to list executions: {}", e)))?;

    let mut executions = Vec::new();

    for stored in stored_executions {
        // Parse execution ID
        let execution_id = Uuid::parse_str(&stored.id)
            .unwrap_or_else(|_| Uuid::nil());

        // Deserialize data field from Vec<u8> to JSON
        let data: serde_json::Value = serde_json::from_slice(&stored.data)
            .unwrap_or(serde_json::json!({}));

        // Parse data field
        let workflow_id = data
            .get("workflow_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let started_at = data
            .get("started_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| {
                chrono::DateTime::from_timestamp(stored.created_at as i64, 0)
                    .unwrap_or_else(|| Utc::now())
            });

        let completed_at = data
            .get("completed_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let outputs = data
            .get("result")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let error = data
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let status = match stored.status {
            StoredExecutionStatus::Running => ExecutionStatus::Running,
            StoredExecutionStatus::Completed => ExecutionStatus::Completed,
            StoredExecutionStatus::Failed => ExecutionStatus::Failed,
            StoredExecutionStatus::Paused => ExecutionStatus::Paused,
            StoredExecutionStatus::Cancelled => ExecutionStatus::Cancelled,
        };

        executions.push(ExecutionDetails {
            id: execution_id,
            workflow_id,
            status,
            started_at,
            completed_at,
            outputs,
            error,
        });
    }

    Ok(Json(executions))
}

/// Pause an execution
pub async fn pause_execution(
    State(state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Check if execution exists
    let mut stored = state
        .state_store
        .get_execution(&execution_id.to_string())
        .map_err(|_| ApiError::not_found("Execution", &execution_id.to_string()))?
        .ok_or_else(|| ApiError::not_found("Execution", &execution_id.to_string()))?;

    // Check if execution is running
    if stored.status != StoredExecutionStatus::Running {
        return Err(ApiError::bad_request("Execution is not running"));
    }

    // Update status to Paused
    stored.status = StoredExecutionStatus::Paused;
    stored.updated_at = chrono::Utc::now().timestamp() as u64;

    state
        .state_store
        .save_execution(&stored)
        .map_err(|e| ApiError::internal_error(format!("Failed to update execution: {}", e)))?;

    Ok(StatusCode::OK)
}

/// Resume an execution
pub async fn resume_execution(
    State(state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Check if execution exists
    let mut stored = state
        .state_store
        .get_execution(&execution_id.to_string())
        .map_err(|_| ApiError::not_found("Execution", &execution_id.to_string()))?
        .ok_or_else(|| ApiError::not_found("Execution", &execution_id.to_string()))?;

    // Check if execution is paused
    if stored.status != StoredExecutionStatus::Paused {
        return Err(ApiError::bad_request("Execution is not paused"));
    }

    // Update status to Running
    stored.status = StoredExecutionStatus::Running;
    stored.updated_at = chrono::Utc::now().timestamp() as u64;

    state
        .state_store
        .save_execution(&stored)
        .map_err(|e| ApiError::internal_error(format!("Failed to update execution: {}", e)))?;

    Ok(StatusCode::OK)
}

/// Cancel an execution
pub async fn cancel_execution(
    State(state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Check if execution exists in registry and cancel it
    if state.execution_registry.exists(&execution_id).await {
        state.execution_registry.cancel(&execution_id).await;
    }

    // Update status in state store if execution exists
    if let Ok(Some(mut stored)) = state.state_store.get_execution(&execution_id.to_string()) {
        stored.status = StoredExecutionStatus::Cancelled;
        stored.updated_at = chrono::Utc::now().timestamp() as u64;

        state
            .state_store
            .save_execution(&stored)
            .map_err(|e| ApiError::internal_error(format!("Failed to update execution: {}", e)))?;
    }

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
            Arc::new(crate::api::ExecutionRegistry::new()),
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

        // Create a running execution first
        let execution = crate::storage::StoredExecution {
            id: execution_id.to_string(),
            status: crate::storage::StoredExecutionStatus::Running,
            created_at: chrono::Utc::now().timestamp() as u64,
            updated_at: chrono::Utc::now().timestamp() as u64,
            data: serde_json::to_vec(&serde_json::json!({
                "workflow_id": "test-workflow"
            })).unwrap(),
        };
        state.state_store.save_execution(&execution).unwrap();

        let result = pause_execution(State(state.clone()), Path(execution_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);

        // Verify status changed to Paused
        let stored = state.state_store.get_execution(&execution_id.to_string()).unwrap().unwrap();
        assert_eq!(stored.status, crate::storage::StoredExecutionStatus::Paused);
    }

    #[tokio::test]
    async fn test_resume_execution() {
        let state = create_test_state();
        let execution_id = Uuid::new_v4();

        // Create a paused execution first
        let execution = crate::storage::StoredExecution {
            id: execution_id.to_string(),
            status: crate::storage::StoredExecutionStatus::Paused,
            created_at: chrono::Utc::now().timestamp() as u64,
            updated_at: chrono::Utc::now().timestamp() as u64,
            data: serde_json::to_vec(&serde_json::json!({
                "workflow_id": "test-workflow"
            })).unwrap(),
        };
        state.state_store.save_execution(&execution).unwrap();

        let result = resume_execution(State(state.clone()), Path(execution_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);

        // Verify status changed to Running
        let stored = state.state_store.get_execution(&execution_id.to_string()).unwrap().unwrap();
        assert_eq!(stored.status, crate::storage::StoredExecutionStatus::Running);
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
