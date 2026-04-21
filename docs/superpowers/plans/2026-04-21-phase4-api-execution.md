# Phase 4: API & Execution Patterns Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement REST and WebSocket APIs for external access, add parallel execution with merge strategies, and implement timeout/cancellation handling.

**Architecture:** Axum-based REST API with WebSocket support for real-time updates. Parallel executor spawns multiple agents concurrently and merges results.

**Tech Stack:** axum (web framework), tower-http (middleware), tokio-tungstenite (WebSocket)

---

## File Structure

**API Layer:**
- `src/api/mod.rs` - Module exports
- `src/api/server.rs` - Axum server setup
- `src/api/routes.rs` - Route definitions
- `src/api/handlers/mod.rs` - Handler module
- `src/api/handlers/workflows.rs` - Workflow endpoints
- `src/api/handlers/executions.rs` - Execution endpoints
- `src/api/handlers/agents.rs` - Agent endpoints
- `src/api/handlers/tools.rs` - Tool endpoints
- `src/api/websocket.rs` - WebSocket handler
- `src/api/error.rs` - API error types
- `src/api/state.rs` - Shared application state

**Engine Updates:**
- Modify: `src/engine/executor.rs` - Add parallel executor
- Create: `src/engine/merge.rs` - Merge strategies
- Create: `src/engine/timeout.rs` - Timeout and cancellation

**Tests:**
- `tests/integration/api_endpoints.rs` - REST API tests
- `tests/integration/websocket.rs` - WebSocket tests
- `tests/integration/parallel_execution.rs` - Parallel execution tests

---

## Task 1: API Types and Error Handling

**Files:**
- Create: `src/api/mod.rs`
- Create: `src/api/error.rs`
- Create: `src/api/state.rs`

**Key Steps:**
- [ ] Add axum, tower-http dependencies to Cargo.toml
- [ ] Define ApiError enum with HTTP status codes
- [ ] Implement IntoResponse for ApiError
- [ ] Define AppState with shared resources (config, registries, state store)
- [ ] Write tests for error serialization
- [ ] Run tests and commit

**Implementation:**
```rust
// src/api/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "WORKFLOW_NOT_FOUND" | "EXECUTION_NOT_FOUND" => StatusCode::NOT_FOUND,
            "INVALID_WORKFLOW_DEFINITION" => StatusCode::BAD_REQUEST,
            "PERMISSION_DENIED" => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        (status, Json(self)).into_response()
    }
}

impl From<crate::error::Error> for ApiError {
    fn from(err: crate::error::Error) -> Self {
        match err {
            crate::error::Error::WorkflowNotFound { workflow_id } => ApiError {
                code: "WORKFLOW_NOT_FOUND".to_string(),
                message: format!("Workflow '{}' not found", workflow_id),
                details: None,
            },
            _ => ApiError {
                code: "INTERNAL_ERROR".to_string(),
                message: err.to_string(),
                details: None,
            },
        }
    }
}

// src/api/state.rs
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GlobalConfig>,
    pub llm_registry: Arc<LLMRegistry>,
    pub agent_registry: Arc<AgentRegistry>,
    pub tool_registry: Arc<ToolRegistry>,
    pub permission_manager: Arc<PermissionManager>,
    pub state_store: Arc<StateStore>,
    pub event_bus: Arc<EventBus>,
}
```

---

## Task 2: Workflow Endpoints

**Files:**
- Create: `src/api/routes.rs`
- Create: `src/api/handlers/mod.rs`
- Create: `src/api/handlers/workflows.rs`

**Key Steps:**
- [ ] Write tests for workflow CRUD endpoints
- [ ] Implement POST /api/workflows (create workflow)
- [ ] Implement GET /api/workflows (list workflows)
- [ ] Implement GET /api/workflows/{id} (get workflow)
- [ ] Implement DELETE /api/workflows/{id} (delete workflow)
- [ ] Add validation for workflow definitions
- [ ] Run tests and commit

**Implementation:**
```rust
// src/api/handlers/workflows.rs
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn create_workflow(
    State(state): State<AppState>,
    Json(definition): Json<WorkflowDefinition>,
) -> Result<Json<WorkflowResponse>, ApiError> {
    // Validate workflow
    WorkflowValidator::validate(&definition)?;
    
    // Save to storage
    let workflow_id = Uuid::new_v4().to_string();
    // TODO: Save workflow definition to file system
    
    Ok(Json(WorkflowResponse {
        id: workflow_id,
        name: definition.name,
        created_at: Utc::now(),
    }))
}

pub async fn list_workflows(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkflowSummary>>, ApiError> {
    // TODO: List workflows from file system
    Ok(Json(vec![]))
}

pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorkflowDefinition>, ApiError> {
    // TODO: Load workflow from file system
    Err(ApiError {
        code: "WORKFLOW_NOT_FOUND".to_string(),
        message: format!("Workflow '{}' not found", id),
        details: None,
    })
}

pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // TODO: Delete workflow file
    Ok(StatusCode::NO_CONTENT)
}
```

---

## Task 3: Execution Endpoints

**Files:**
- Create: `src/api/handlers/executions.rs`

**Key Steps:**
- [ ] Write tests for execution endpoints
- [ ] Implement POST /api/workflows/{id}/execute (start execution)
- [ ] Implement GET /api/executions/{id} (get execution status)
- [ ] Implement GET /api/executions (list executions)
- [ ] Implement POST /api/executions/{id}/pause (pause execution)
- [ ] Implement POST /api/executions/{id}/resume (resume execution)
- [ ] Implement DELETE /api/executions/{id} (cancel execution)
- [ ] Run tests and commit

**Implementation:**
```rust
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(inputs): Json<HashMap<String, Value>>,
) -> Result<Json<ExecutionResponse>, ApiError> {
    // Load workflow
    let workflow = load_workflow(&workflow_id)?;
    
    // Create execution context
    let execution_id = Uuid::new_v4();
    let mut context = ExecutionContext {
        workflow_id: workflow_id.clone(),
        execution_id,
        context_store: HashMap::new(),
    };
    
    // Add inputs to context
    for (key, value) in inputs {
        context.context_store.insert(format!("input.{}", key), value);
    }
    
    // Spawn execution in background
    let state_clone = state.clone();
    tokio::spawn(async move {
        let executor = SequentialExecutor::new(state_clone.event_bus.clone());
        let result = executor.execute(
            &workflow,
            &mut context,
            state_clone.llm_registry.clone(),
            &state_clone.agent_registry,
        ).await;
        
        // Save execution result
        // TODO: Update execution status in state store
    });
    
    Ok(Json(ExecutionResponse {
        execution_id,
        status: ExecutionStatus::Running,
        started_at: Utc::now(),
    }))
}

pub async fn get_execution(
    State(state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<ExecutionDetails>, ApiError> {
    let execution = state.state_store
        .get_execution(execution_id)
        .await?
        .ok_or_else(|| ApiError {
            code: "EXECUTION_NOT_FOUND".to_string(),
            message: format!("Execution '{}' not found", execution_id),
            details: None,
        })?;
    
    Ok(Json(ExecutionDetails {
        id: execution.id,
        workflow_id: execution.workflow_id,
        status: execution.status,
        started_at: execution.started_at,
        completed_at: execution.completed_at,
        outputs: execution.outputs,
        error: execution.error,
    }))
}
```

---

## Task 4: WebSocket Handler

**Files:**
- Create: `src/api/websocket.rs`

**Key Steps:**
- [ ] Write test for WebSocket connection and event streaming
- [ ] Implement WebSocket upgrade handler
- [ ] Subscribe to event bus for execution events
- [ ] Forward events to WebSocket clients
- [ ] Handle client messages (pause, resume, cancel)
- [ ] Add connection management (disconnect cleanup)
- [ ] Run tests and commit

**Implementation:**
```rust
// src/api/websocket.rs
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(execution_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, execution_id, state))
}

async fn handle_socket(socket: WebSocket, execution_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to event bus
    let mut event_rx = state.event_bus.subscribe();
    
    // Forward events to WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            // Filter events for this execution
            // TODO: Add execution_id to events
            
            let msg = serde_json::to_string(&event).unwrap();
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Handle client commands (pause, resume, cancel)
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                    match cmd {
                        ClientCommand::Pause => {
                            // TODO: Pause execution
                        }
                        ClientCommand::Resume => {
                            // TODO: Resume execution
                        }
                        ClientCommand::Cancel => {
                            // TODO: Cancel execution
                        }
                    }
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}

#[derive(Debug, Deserialize)]
enum ClientCommand {
    Pause,
    Resume,
    Cancel,
}
```

---

## Task 5: Axum Server Setup

**Files:**
- Create: `src/api/server.rs`
- Modify: `src/main.rs`

**Key Steps:**
- [ ] Implement start_server() function
- [ ] Set up Axum router with all routes
- [ ] Add CORS middleware
- [ ] Add logging middleware
- [ ] Add graceful shutdown handling
- [ ] Wire up to CLI `ui` command
- [ ] Test server startup manually
- [ ] Commit

**Implementation:**
```rust
// src/api/server.rs
use axum::{
    routing::{get, post, delete},
    Router,
};
use tower_http::cors::CorsLayer;

pub async fn start_server(state: AppState, port: u16) -> Result<()> {
    let app = Router::new()
        // Workflow routes
        .route("/api/workflows", post(handlers::workflows::create_workflow))
        .route("/api/workflows", get(handlers::workflows::list_workflows))
        .route("/api/workflows/:id", get(handlers::workflows::get_workflow))
        .route("/api/workflows/:id", delete(handlers::workflows::delete_workflow))
        
        // Execution routes
        .route("/api/workflows/:id/execute", post(handlers::executions::execute_workflow))
        .route("/api/executions/:id", get(handlers::executions::get_execution))
        .route("/api/executions", get(handlers::executions::list_executions))
        .route("/api/executions/:id/pause", post(handlers::executions::pause_execution))
        .route("/api/executions/:id/resume", post(handlers::executions::resume_execution))
        .route("/api/executions/:id", delete(handlers::executions::cancel_execution))
        
        // WebSocket
        .route("/api/ws/executions/:id", get(websocket::websocket_handler))
        
        // Agent and tool routes
        .route("/api/agents", get(handlers::agents::list_agents))
        .route("/api/tools", get(handlers::tools::list_tools))
        
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .await?;
    
    Ok(())
}
```

---

## Task 6: Parallel Executor

**Files:**
- Create: `src/engine/parallel.rs`
- Modify: `src/engine/executor.rs`

**Key Steps:**
- [ ] Write tests for parallel execution (3 agents run concurrently)
- [ ] Implement ParallelExecutor struct
- [ ] Spawn multiple agents using tokio::spawn
- [ ] Collect results with futures::join_all
- [ ] Handle timeouts and failures gracefully
- [ ] Return Vec<Option<AgentOutput>> (Some for success, None for failure)
- [ ] Run tests and commit

**Implementation:**
```rust
// src/engine/parallel.rs
pub struct ParallelExecutor {
    event_bus: Arc<EventBus>,
}

impl ParallelExecutor {
    pub async fn execute(
        &self,
        workflow: &WorkflowDefinition,
        context: &ExecutionContext,
        llm_registry: Arc<LLMRegistry>,
        agent_registry: &AgentRegistry,
        tool_registry: Arc<ToolRegistry>,
    ) -> Result<Vec<Option<AgentOutput>>> {
        let mut tasks = Vec::new();
        
        for agent_config in &workflow.agents {
            let agent_def = agent_registry.get(&agent_config.agent_type)
                .ok_or_else(|| Error::Internal(format!("Agent type not found")))?
                .clone();
            
            let llm_clone = llm_registry.clone();
            let tool_clone = tool_registry.clone();
            let context_clone = context.clone();
            let config_clone = agent_config.clone();
            let event_bus_clone = self.event_bus.clone();
            
            let task = tokio::spawn(async move {
                // Create agent
                let mut agent = create_agent_instance(&agent_def, llm_clone)?;
                
                // Interpolate task
                let interpolator = VariableInterpolator::new(&context_clone);
                let task_description = interpolator.interpolate(&config_clone.task)?;
                
                // Execute with timeout
                let timeout = config_clone.timeout
                    .unwrap_or(Duration::from_secs(300));
                
                let result = tokio::time::timeout(
                    timeout,
                    agent.execute(
                        Task {
                            id: Uuid::new_v4().to_string(),
                            description: task_description,
                        },
                        &context_clone,
                    )
                ).await;
                
                match result {
                    Ok(Ok(output)) => {
                        event_bus_clone.publish(AgentEvent::TaskCompleted {
                            agent_id: config_clone.id.clone(),
                            output: output.content.clone(),
                        })?;
                        Ok(Some(output))
                    }
                    Ok(Err(e)) => {
                        event_bus_clone.publish(AgentEvent::TaskFailed {
                            agent_id: config_clone.id.clone(),
                            error: e.to_string(),
                        })?;
                        Ok(None)
                    }
                    Err(_) => {
                        event_bus_clone.publish(AgentEvent::TaskFailed {
                            agent_id: config_clone.id.clone(),
                            error: "Timeout".to_string(),
                        })?;
                        Ok(None)
                    }
                }
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks
        let results = futures::future::join_all(tasks).await;
        
        // Collect outputs
        let outputs: Vec<Option<AgentOutput>> = results.into_iter()
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(outputs)
    }
}
```

---

## Task 7: Merge Strategies

**Files:**
- Create: `src/engine/merge.rs`

**Key Steps:**
- [ ] Write tests for concat, vote, llm_merge strategies
- [ ] Implement MergeStrategy enum
- [ ] Implement apply_merge_strategy() function
- [ ] Implement concat (join all outputs)
- [ ] Implement vote (most common output)
- [ ] Implement llm_merge (use LLM to intelligently merge)
- [ ] Run tests and commit

**Implementation:**
```rust
// src/engine/merge.rs
pub enum MergeStrategy {
    Concat,
    Vote,
    LlmMerge {
        agent_type: String,
        prompt: String,
    },
}

pub async fn apply_merge_strategy(
    outputs: Vec<AgentOutput>,
    strategy: &MergeStrategy,
    llm_registry: Arc<LLMRegistry>,
    agent_registry: &AgentRegistry,
) -> Result<AgentOutput> {
    match strategy {
        MergeStrategy::Concat => {
            let combined = outputs.iter()
                .map(|o| o.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            
            Ok(AgentOutput {
                content: combined,
                metadata: HashMap::new(),
            })
        }
        
        MergeStrategy::Vote => {
            let mut counts = HashMap::new();
            for output in &outputs {
                *counts.entry(&output.content).or_insert(0) += 1;
            }
            
            let most_common = counts.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(content, _)| (*content).clone())
                .unwrap_or_default();
            
            Ok(AgentOutput {
                content: most_common,
                metadata: HashMap::new(),
            })
        }
        
        MergeStrategy::LlmMerge { agent_type, prompt } => {
            let combined_results = outputs.iter()
                .enumerate()
                .map(|(i, o)| format!("Result {}:\n{}", i + 1, o.content))
                .collect::<Vec<_>>()
                .join("\n\n");
            
            let merge_prompt = prompt.replace("{results}", &combined_results);
            
            // Create merge agent
            let agent_def = agent_registry.get(agent_type)
                .ok_or_else(|| Error::Internal(format!("Agent type not found")))?;
            
            let mut merge_agent = create_agent_instance(agent_def, llm_registry)?;
            
            let merged = merge_agent.execute(
                Task {
                    id: Uuid::new_v4().to_string(),
                    description: merge_prompt,
                },
                &ExecutionContext::default(),
            ).await?;
            
            Ok(merged)
        }
    }
}
```

---

## Task 8: Update Executor to Support Both Modes

**Files:**
- Modify: `src/engine/executor.rs`
- Modify: `src/engine/types.rs`

**Key Steps:**
- [ ] Add ExecutionMode enum to WorkflowDefinition
- [ ] Update executor to dispatch to sequential or parallel based on mode
- [ ] Add merge_strategy field to parallel mode
- [ ] Update CLI and API to support both modes
- [ ] Add integration test for parallel + merge
- [ ] Commit

---

## Task 9: Timeout and Cancellation

**Files:**
- Create: `src/engine/timeout.rs`

**Key Steps:**
- [ ] Implement CancellationToken using tokio::sync::CancellationToken
- [ ] Add global workflow timeout
- [ ] Add per-agent timeout
- [ ] Implement graceful cancellation (send token, wait, then force kill)
- [ ] Update executors to respect cancellation tokens
- [ ] Add tests for timeout and cancellation
- [ ] Commit

---

## Task 10: Integration Tests

**Files:**
- Create: `tests/integration/api_endpoints.rs`
- Create: `tests/integration/parallel_execution.rs`

**Key Steps:**
- [ ] Test all REST endpoints
- [ ] Test WebSocket connection and event streaming
- [ ] Test parallel execution with 3+ agents
- [ ] Test merge strategies
- [ ] Test timeout handling
- [ ] Run tests and commit

---

## Task 11: Documentation

**Files:**
- Modify: `README.md`
- Create: `docs/api-reference.md`
- Create: `docs/execution-patterns.md`

**Key Steps:**
- [ ] Document REST API endpoints with examples
- [ ] Document WebSocket protocol
- [ ] Document execution patterns (sequential vs parallel)
- [ ] Document merge strategies
- [ ] Add curl examples for API usage
- [ ] Update README with Phase 4 features
- [ ] Commit

---

## Phase 4 Complete

**Deliverables:**
- ✅ REST API (workflows, executions, agents, tools)
- ✅ WebSocket API for real-time updates
- ✅ Parallel execution with concurrent agents
- ✅ Merge strategies (concat, vote, llm_merge)
- ✅ Timeout and cancellation handling
- ✅ Integration tests
- ✅ API documentation

**Next:** Phase 5 - UI Layer