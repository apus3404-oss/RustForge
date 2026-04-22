use crate::api::{ApiError, AppState};
use crate::engine::types::WorkflowDefinition;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Response for workflow creation
#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Summary of a workflow
#[derive(Debug, Serialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub agent_count: usize,
    pub created_at: DateTime<Utc>,
}

/// Simple workflow validation
fn validate_workflow(definition: &WorkflowDefinition) -> Result<(), String> {
    // Check for duplicate agent IDs
    let mut seen_ids = HashSet::new();
    for agent in &definition.agents {
        if !seen_ids.insert(&agent.id) {
            return Err(format!("Duplicate agent ID: {}", agent.id));
        }
    }

    // Check for empty agents list
    if definition.agents.is_empty() {
        return Err("Workflow must have at least one agent".to_string());
    }

    Ok(())
}

/// Create a new workflow
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(definition): Json<WorkflowDefinition>,
) -> Result<Json<WorkflowResponse>, ApiError> {
    // Validate workflow
    validate_workflow(&definition)
        .map_err(|e| ApiError::bad_request(format!("Invalid workflow: {}", e)))?;

    // Generate workflow ID
    let workflow_id = Uuid::new_v4().to_string();

    // Save workflow definition to file system
    state
        .workflow_store
        .save(&workflow_id, &definition)
        .map_err(|e| ApiError::internal_error(format!("Failed to save workflow: {}", e)))?;

    Ok(Json(WorkflowResponse {
        id: workflow_id,
        name: definition.name,
        created_at: Utc::now(),
    }))
}

/// List all workflows
pub async fn list_workflows(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkflowSummary>>, ApiError> {
    // List workflow IDs from file system
    let workflow_ids = state
        .workflow_store
        .list()
        .map_err(|e| ApiError::internal_error(format!("Failed to list workflows: {}", e)))?;

    let mut summaries = Vec::new();

    // Load each workflow to create summary
    for id in workflow_ids {
        if let Ok(definition) = state.workflow_store.load(&id) {
            summaries.push(WorkflowSummary {
                id: id.clone(),
                name: definition.name,
                mode: format!("{:?}", definition.mode).to_lowercase(),
                agent_count: definition.agents.len(),
                created_at: Utc::now(), // TODO: Store actual creation time
            });
        }
    }

    Ok(Json(summaries))
}

/// Get a specific workflow by ID
pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorkflowDefinition>, ApiError> {
    // Load workflow from file system
    let definition = state
        .workflow_store
        .load(&id)
        .map_err(|_| ApiError::not_found("Workflow", &id))?;

    Ok(Json(definition))
}

/// Delete a workflow by ID
pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Delete workflow file
    state
        .workflow_store
        .delete(&id)
        .map_err(|_| ApiError::not_found("Workflow", &id))?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::AgentRegistry;
    use crate::config::GlobalConfig;
    use crate::engine::{EventBus, types::AgentConfig};
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
        let unique_dir = temp_dir.join(format!("rustforge_test_{}", Uuid::new_v4()));
        let db_path = unique_dir.join("test.db");

        // Create unique directory
        std::fs::create_dir_all(&unique_dir).unwrap();

        let state_store = Arc::new(StateStore::new(&db_path).unwrap());
        let workflow_store = Arc::new(
            crate::storage::WorkflowStore::new(&unique_dir).unwrap()
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
    async fn test_create_workflow() {
        let state = create_test_state();

        let workflow = WorkflowDefinition {
            name: "Test Workflow".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![AgentConfig {
                id: "agent1".to_string(),
                agent_type: "base".to_string(),
                task: "Test task".to_string(),
                depends_on: vec![],
                config: std::collections::HashMap::new(),
            }],
            inputs: None,
        };

        let result = create_workflow(State(state), Json(workflow)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.name, "Test Workflow");
        assert!(!response.id.is_empty());
    }

    #[tokio::test]
    async fn test_create_invalid_workflow() {
        let state = create_test_state();

        // Workflow with duplicate agent IDs
        let workflow = WorkflowDefinition {
            name: "Invalid Workflow".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![
                AgentConfig {
                    id: "agent1".to_string(),
                    agent_type: "base".to_string(),
                    task: "Task 1".to_string(),
                    depends_on: vec![],
                    config: std::collections::HashMap::new(),
                },
                AgentConfig {
                    id: "agent1".to_string(),
                    agent_type: "base".to_string(),
                    task: "Task 2".to_string(),
                    depends_on: vec![],
                    config: std::collections::HashMap::new(),
                },
            ],
            inputs: None,
        };

        let result = create_workflow(State(state), Json(workflow)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_workflows() {
        let state = create_test_state();

        // Create a workflow first
        let workflow = WorkflowDefinition {
            name: "Test Workflow".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![AgentConfig {
                id: "agent1".to_string(),
                agent_type: "base".to_string(),
                task: "Test task".to_string(),
                depends_on: vec![],
                config: std::collections::HashMap::new(),
            }],
            inputs: None,
        };

        create_workflow(State(state.clone()), Json(workflow)).await.unwrap();

        // Now list workflows
        let result = list_workflows(State(state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 1);
    }

    #[tokio::test]
    async fn test_get_workflow_not_found() {
        let state = create_test_state();
        let result = get_workflow(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_workflow() {
        let state = create_test_state();

        // First create a workflow
        let workflow = WorkflowDefinition {
            name: "Test Workflow".to_string(),
            mode: crate::engine::types::ExecutionMode::Sequential,
            agents: vec![AgentConfig {
                id: "agent1".to_string(),
                agent_type: "base".to_string(),
                task: "Test task".to_string(),
                depends_on: vec![],
                config: std::collections::HashMap::new(),
            }],
            inputs: None,
        };

        let create_result = create_workflow(State(state.clone()), Json(workflow)).await;
        assert!(create_result.is_ok());
        let workflow_id = create_result.unwrap().0.id;

        // Now delete it
        let result = delete_workflow(State(state), Path(workflow_id)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }
}
