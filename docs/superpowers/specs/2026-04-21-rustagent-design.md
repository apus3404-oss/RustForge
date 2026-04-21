# RustAgent Design Specification

**Date:** 2026-04-21  
**Project:** RustForge - Local AI Agent Orchestrator  
**Repository:** https://github.com/apus3404-oss/RustForge

---

## Executive Summary

RustAgent is a high-performance, local-first AI agent orchestration platform built in Rust. It combines the workflow capabilities of n8n with the autonomous agent features of AutoGPT/CrewAI, while delivering 5-10x better performance and significantly lower memory footprint than Python-based alternatives.

**Key Differentiators:**
- **Ultra-fast execution**: Rust + Tokio for parallel agent coordination
- **Low memory footprint**: 500-800 MB vs 5-10 GB for Python alternatives
- **Local-first**: Privacy-focused, all data stays on user's machine
- **Hybrid approach**: Visual workflow builder + code-based configuration
- **Multi-pattern execution**: Sequential, parallel, and hierarchical (supervisor) modes

---

## Project Goals

### Primary Goals

1. **Performance Leadership**: Become the fastest local AI agent orchestrator
   - Startup time < 100ms
   - Agent spawn time < 50ms per agent
   - Support 10+ parallel agents simultaneously
   - Memory usage: 500-800 MB baseline

2. **Developer Experience**: Make agent orchestration accessible yet powerful
   - Visual workflow builder for non-technical users
   - YAML/TOML configuration for developers
   - Git-friendly workflow definitions
   - Clear error messages with actionable suggestions

3. **Privacy & Security**: Local-first architecture with robust security
   - All data stays on user's machine
   - Permission-based tool execution
   - Process isolation for agents
   - Audit logging for compliance

4. **Extensibility**: Easy to add custom tools and agents
   - Built-in tools for common tasks
   - Python/Lua scripting for custom tools (v1.0)
   - Plugin system for community contributions

### Target Use Cases

**Primary (MVP):**
- General automation + research workflows (C + B mix)
- PDF processing pipelines (read → summarize → export)
- Multi-source web research (parallel scraping + aggregation)
- File organization and analysis
- Daily assistant tasks (news aggregation, report generation)

**Secondary (v1.0):**
- Code analysis and refactoring suggestions
- Security scanning and vulnerability detection
- Personal knowledge management with RAG

---

## Success Metrics

### GitHub Visibility (Primary Goal)
- 1,000+ stars in first 3 months
- 100+ forks in first 3 months
- Active community contributions (issues, PRs)

### Technical Metrics
- Benchmark: 5-10x faster than Python alternatives
- Memory: < 1 GB for typical workflows
- Reliability: 99%+ workflow completion rate

### User Adoption
- 1,000+ downloads in first month
- Active Discord/community engagement
- Positive feedback on performance and UX
## System Architecture

### Layered Architecture Overview

RustAgent follows a layered architecture pattern that balances rapid MVP development with long-term extensibility:

```
┌─────────────────────────────────────┐
│   UI Layer                          │
│   - Axum web server                 │
│   - React/Svelte web UI             │
│   - Tauri desktop wrapper (opt)     │
├─────────────────────────────────────┤
│   API Layer                         │
│   - REST endpoints                  │
│   - WebSocket (real-time updates)   │
├─────────────────────────────────────┤
│   Orchestration Engine              │
│   - Workflow executor               │
│   - Agent coordinator               │
│   - Permission manager              │
│   - Event bus                       │
├─────────────────────────────────────┤
│   Agent Layer                       │
│   - Agent registry & definitions    │
│   - Agent trait & implementations   │
│   - Memory store                    │
├─────────────────────────────────────┤
│   LLM Layer                         │
│   - LLM provider trait              │
│   - Ollama / OpenAI / Anthropic     │
│   - Streaming & tool calling        │
│   - Context management              │
├─────────────────────────────────────┤
│   Tool Layer                        │
│   - Built-in tools (Rust)           │
│   - Script runner (Python/Lua)      │
│   - Tool registry                   │
├─────────────────────────────────────┤
│   Storage Layer                     │
│   - Config files (YAML/TOML)        │
│   - State database (redb)           │
│   - Vector store (optional, v1.0)   │
├─────────────────────────────────────┤
│   Config & CLI Layer                │
│   - Global configuration            │
│   - CLI commands (clap)             │
│   - Logging (tracing)               │
└─────────────────────────────────────┘
```

**Why This Architecture:**
- **Clear separation of concerns**: Each layer has well-defined responsibilities
- **Testable**: Layers can be tested independently with mock implementations
- **Extensible**: New features can be added without touching core layers
- **Performance**: Rust's type system enforces layer boundaries at compile time
- **Maintainable**: Easy for contributors to understand and modify specific layers

---

## Core Components

### 1. Orchestration Engine (`src/engine/`)

The heart of RustAgent, responsible for workflow execution and agent coordination.

#### Workflow Executor

**Responsibilities:**
- Parse YAML/TOML workflow definitions
- Build and validate DAG (Directed Acyclic Graph)
- Execute workflows based on mode (sequential/parallel/supervisor)
- Manage state transitions (pending → running → completed/failed)
- Handle checkpoints for pause/resume functionality

**Key Types:**
```rust
pub struct WorkflowDefinition {
    pub name: String,
    pub mode: ExecutionMode,
    pub agents: Vec<AgentConfig>,
    pub timeout: Option<Duration>,
    pub post_process: Option<PostProcessConfig>,
}

pub enum ExecutionMode {
    Sequential,
    Parallel { merge_strategy: MergeStrategy },
    Supervisor { max_workers: usize }, // v1.0
}

pub enum MergeStrategy {
    Concat,
    Vote,
    LlmMerge { agent_type: String, prompt: String },
    Custom { script: String }, // v1.0
}
```

#### Agent Coordinator

**Responsibilities:**
- Spawn and monitor agent processes
- Manage agent lifecycle (start, pause, resume, terminate)
- Coordinate inter-agent communication (supervisor pattern)
- Apply resource limits (CPU, memory, execution time)
- Implement retry logic and error recovery

**Key Features:**
- Process isolation: Each agent runs in separate process
- Resource monitoring: Track CPU/memory usage per agent
- Graceful shutdown: Send cancellation tokens before force kill
- Health checks: Detect and restart crashed agents

#### Permission Manager

**Responsibilities:**
- Enforce permission policies before tool execution
- Prompt user for permission when needed
- Maintain audit log of all tool executions
- Support multiple permission scopes (once, session, forever)

**Permission Model:**
```rust
pub struct PermissionPolicy {
    pub default: PolicyAction, // Allow | Deny
    pub rules: Vec<PermissionRule>,
}

pub struct PermissionRule {
    pub tool: String,
    pub operations: Vec<String>,
    pub scope: Option<Scope>, // paths, domains, commands
    pub action: PolicyAction,
}

pub enum PolicyAction {
    Allow,
    Deny,
    Prompt,
}
```

#### Event Bus

**Responsibilities:**
- Broadcast events across the system
- Enable real-time UI updates
- Support supervisor pattern communication
- Facilitate audit logging

**Event Types:**
```rust
pub enum AgentEvent {
    TaskStarted { agent_id: String, task_id: String },
    TaskCompleted { agent_id: String, output: AgentOutput },
    TaskFailed { agent_id: String, error: String },
    ToolExecuted { agent_id: String, tool: String, result: ToolResult },
    MessageSent { from: String, to: String, content: String },
}
```

**Implementation:**
- Uses `tokio::sync::broadcast` for efficient pub/sub
- Multiple subscribers (UI, audit logger, supervisor)
- Non-blocking: Events don't slow down execution
### 2. Agent Layer (`src/agents/`)

Defines agent types, behaviors, and memory management.

#### Agent Definition

```rust
pub struct AgentDefinition {
    pub id: String,
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub llm_provider: String, // "ollama:llama3", "openai:gpt-4"
    pub available_tools: Vec<String>,
    pub memory_config: MemoryConfig,
    pub max_iterations: usize,
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&mut self, task: Task, context: ExecutionContext) 
        -> Result<AgentOutput>;
    fn get_definition(&self) -> &AgentDefinition;
    fn get_memory(&self) -> &dyn MemoryStore;
}
```

#### Built-in Agent Types

**ResearchAgent (MVP):**
- Purpose: Web research, data collection, document analysis
- Tools: web_scraper, pdf_parser, file_system, api_client
- Behavior: Iterative research with source tracking

**AnalysisAgent (MVP):**
- Purpose: Data analysis, summarization, report generation
- Tools: file_system (primarily for output)
- Behavior: Structured analysis with clear reasoning

**SupervisorAgent (v1.0):**
- Purpose: Coordinate multiple worker agents
- Tools: All tools (delegates to workers)
- Behavior: Task decomposition, worker assignment, result synthesis

**CodeAgent (v1.0):**
- Purpose: Code analysis, refactoring suggestions
- Tools: file_system, shell_executor, git
- Behavior: AST-based analysis with context awareness

#### Agent Registry

```rust
pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
}

impl AgentRegistry {
    pub fn register(&mut self, definition: AgentDefinition);
    pub fn get(&self, id: &str) -> Option<&AgentDefinition>;
    pub fn list(&self) -> Vec<&AgentDefinition>;
    pub fn create_instance(&self, id: &str) -> Result<Box<dyn Agent>>;
}
```

#### Memory Store

**MVP: Simple Conversation Memory**
```rust
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn add_message(&mut self, agent_id: &str, message: Message) 
        -> Result<()>;
    async fn get_conversation(&self, agent_id: &str, limit: usize) 
        -> Result<Vec<Message>>;
    async fn clear(&mut self, agent_id: &str) -> Result<()>;
}

pub struct SimpleMemoryStore {
    db: redb::Database,
}
```

**v1.0: Vector Memory (RAG)**
```rust
#[async_trait]
pub trait MemoryStore: Send + Sync {
    // ... conversation methods ...
    
    async fn add_fact(&mut self, agent_id: &str, fact: Fact) -> Result<()>;
    async fn search_facts(&self, agent_id: &str, query: &str) 
        -> Result<Vec<Fact>>;
}

pub struct VectorMemoryStore {
    db: redb::Database,
    vector_store: Box<dyn VectorStore>, // qdrant or lance
}
```

---

### 3. LLM Layer (`src/llm/`)

**Critical component** - handles all LLM provider integrations.

#### LLM Provider Trait

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) 
        -> Result<String>;
    
    async fn stream(&self, messages: Vec<Message>, options: CompletionOptions) 
        -> Result<impl Stream<Item = String>>;
    
    async fn complete_with_tools(&self, messages: Vec<Message>, tools: Vec<Tool>) 
        -> Result<ToolCallResponse>;
    
    fn supports_streaming(&self) -> bool;
    fn supports_tool_calling(&self) -> bool;
    fn max_context_tokens(&self) -> usize;
}

pub struct CompletionOptions {
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub stop_sequences: Vec<String>,
    pub top_p: Option<f32>,
}
```

#### Provider Implementations

**OllamaProvider (MVP - Primary)**
```rust
pub struct OllamaProvider {
    base_url: String, // http://localhost:11434
    model: String,    // llama3, mistral, etc.
    client: reqwest::Client,
}
```

**Why Ollama as default:**
- Local-first, privacy-focused
- Easy setup (single binary)
- Good model selection (Llama 3, Mistral, etc.)
- Active community support

**OpenAIProvider (MVP - Fallback)**
```rust
pub struct OpenAIProvider {
    api_key: String,
    model: String, // gpt-4o, gpt-4o-mini
    client: reqwest::Client,
}
```

**AnthropicProvider (v1.0)**
```rust
pub struct AnthropicProvider {
    api_key: String,
    model: String, // claude-3-5-sonnet, claude-3-5-haiku
    client: reqwest::Client,
}
```

**GroqProvider (v1.0)**
```rust
pub struct GroqProvider {
    api_key: String,
    model: String, // llama3-70b, mixtral-8x7b
    client: reqwest::Client,
}
```

#### LLM Registry with Fallback

```rust
pub struct LLMRegistry {
    primary: Box<dyn LLMProvider>,
    fallback: Option<Box<dyn LLMProvider>>,
}

impl LLMRegistry {
    pub async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        match self.primary.complete(messages.clone(), options).await {
            Ok(response) => Ok(response),
            Err(e) if self.should_fallback(&e) => {
                if let Some(fallback) = &self.fallback {
                    fallback.complete(messages, options).await
                } else {
                    Err(e)
                }
            }
            Err(e) => Err(e),
        }
    }
    
    fn should_fallback(&self, error: &Error) -> bool {
        matches!(error, 
            Error::ProviderUnavailable | 
            Error::Timeout | 
            Error::RateLimitExceeded
        )
    }
}
```

#### Context Management

**Token Counting:**
```rust
pub struct ContextManager {
    tokenizer: Tokenizer,
    max_tokens: usize,
}

impl ContextManager {
    pub fn count_tokens(&self, messages: &[Message]) -> usize;
    
    pub fn truncate_if_needed(&self, messages: Vec<Message>) 
        -> Vec<Message> {
        let total_tokens = self.count_tokens(&messages);
        if total_tokens <= self.max_tokens {
            return messages;
        }
        
        // Keep system message + recent messages
        let mut truncated = vec![messages[0].clone()]; // system
        let recent = self.get_recent_messages(&messages, 
            self.max_tokens - self.count_tokens(&truncated));
        truncated.extend(recent);
        truncated
    }
}
```

#### Tool Calling Format Adaptation

Different providers have different tool calling formats. The LLM layer normalizes this:

```rust
pub struct ToolCallResponse {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub reasoning: Option<String>,
}

// Each provider implements format conversion
impl OpenAIProvider {
    fn parse_tool_call(&self, response: &str) -> Result<ToolCallResponse> {
        // Parse OpenAI's function calling format
    }
}

impl OllamaProvider {
    fn parse_tool_call(&self, response: &str) -> Result<ToolCallResponse> {
        // Parse Ollama's JSON mode format
    }
}
```
### 4. Tool Layer (`src/tools/`)

Provides built-in tools and extensibility through scripting.

#### Tool Trait

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    fn required_permissions(&self) -> Vec<Permission>;
    
    async fn execute(&self, params: ToolParams, context: &ExecutionContext) 
        -> Result<ToolResult>;
}

pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

#### Built-in Tools (MVP)

**FileSystemTool**
```rust
pub struct FileSystemTool;

// Operations:
// - read_file(path: String) -> String
// - write_file(path: String, content: String) -> ()
// - list_directory(path: String) -> Vec<String>
// - search_files(pattern: String, path: String) -> Vec<String>
// - delete_file(path: String) -> ()
// - create_directory(path: String) -> ()

// Permissions: Checked per operation and path
```

**WebScraperTool**
```rust
pub struct WebScraperTool {
    client: reqwest::Client,
}

// Operations:
// - fetch_url(url: String) -> String (HTML content)
// - fetch_json(url: String) -> Value
// - extract_links(html: String) -> Vec<String>
// - extract_text(html: String, selector: Option<String>) -> String

// Uses: reqwest + scraper crate
// Permissions: Domain-based allow/deny list
```

**PdfParserTool**
```rust
pub struct PdfParserTool;

// Operations:
// - extract_text(path: String) -> String
// - extract_metadata(path: String) -> PdfMetadata
// - extract_pages(path: String, pages: Vec<usize>) -> Vec<String>

// Uses: pdf-extract or lopdf crate
// Permissions: File system read access
```

**ShellExecutorTool**
```rust
pub struct ShellExecutorTool;

// Operations:
// - execute(command: String, args: Vec<String>) -> CommandOutput
// - execute_script(script: String, interpreter: String) -> CommandOutput

pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

// Security:
// - Runs in separate process with timeout
// - Command whitelist/blacklist
// - Resource limits (CPU, memory)
// - No shell injection (uses Command::new, not sh -c)
```

**ApiClientTool**
```rust
pub struct ApiClientTool {
    client: reqwest::Client,
}

// Operations:
// - get(url: String, headers: HashMap) -> Response
// - post(url: String, body: Value, headers: HashMap) -> Response
// - put(url: String, body: Value, headers: HashMap) -> Response
// - delete(url: String, headers: HashMap) -> Response

// Features:
// - Automatic retry with exponential backoff
// - Rate limiting
// - Response caching (optional)
```

**ClipboardTool**
```rust
pub struct ClipboardTool;

// Operations:
// - read() -> String
// - write(content: String) -> ()

// Uses: arboard crate (cross-platform)
// Permissions: Clipboard access (prompt once per session)
```

#### Tool Registry

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Box<dyn Tool>);
    pub fn get(&self, name: &str) -> Option<&dyn Tool>;
    pub fn list(&self) -> Vec<&dyn Tool>;
    
    pub async fn execute(&self, 
        tool_name: &str, 
        params: ToolParams,
        context: &ExecutionContext,
        permission_manager: &PermissionManager
    ) -> Result<ToolResult> {
        let tool = self.get(tool_name)
            .ok_or(Error::ToolNotFound)?;
        
        // Check permissions
        permission_manager.check_permissions(
            tool.required_permissions(),
            context
        ).await?;
        
        // Execute with timeout
        tokio::time::timeout(
            Duration::from_secs(300),
            tool.execute(params, context)
        ).await?
    }
}
```

#### Script Runner (v1.0)

**Python Executor**
```rust
pub struct PythonScriptRunner {
    // Uses PyO3 for embedded Python or subprocess
}

// Example usage in workflow:
// tools:
//   - type: python_script
//     script: |
//       def process(data):
//           # Custom logic
//           return result
```

**Lua Executor**
```rust
pub struct LuaScriptRunner {
    // Uses mlua crate (embedded Lua)
}

// Lua is lighter and safer than Python for simple scripts
// Example:
// tools:
//   - type: lua_script
//     script: |
//       function merge(results)
//           -- Custom merge logic
//           return combined
//       end
```

**Sandboxing:**
- File system access limited to specified paths
- Network access controlled by permissions
- CPU and memory limits enforced
- Timeout for script execution

---

### 5. Storage Layer (`src/storage/`)

Hybrid storage: Config files for definitions, embedded DB for runtime state.

#### Config Store

**Workflow Definitions (YAML/TOML)**

```yaml
# workflows/pdf-research.yaml
name: "PDF Research Pipeline"
description: "Extract and summarize PDF content"
version: "1.0"

mode: sequential

inputs:
  - name: pdf_path
    type: string
    required: true
    description: "Path to PDF file"

agents:
  - id: pdf_reader
    type: ResearchAgent
    llm: "ollama:llama3"
    task: "Read PDF from {input.pdf_path} and extract key points"
    tools: [pdf_parser, file_system]
    timeout: 120
  
  - id: summarizer
    type: AnalysisAgent
    llm: "ollama:llama3"
    task: "Summarize: {pdf_reader.output}"
    tools: []
    timeout: 60

post_process:
  agent_type: AnalysisAgent
  task: "Format as markdown report: {workflow.output}"
  output:
    format: markdown
    path: "reports/{timestamp}_summary.md"
```

**User Settings**
```toml
# .rustforge/config.toml
[llm]
default_provider = "ollama:llama3"

[llm.providers.openai]
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4o-mini"

[llm.providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama3"

[execution]
max_parallel_agents = 10
default_timeout = 300
enable_checkpoints = true

[permissions]
default_policy = "prompt"  # allow | deny | prompt

[ui]
port = 3000
auto_open_browser = true

[logging]
level = "info"  # trace | debug | info | warn | error
```

**Why Git-Friendly Config:**
- Workflow definitions can be versioned
- Easy to share workflows via GitHub
- Diff-friendly YAML format
- Enables workflow templates and examples

#### State Database (redb)

**Why redb:**
- Pure Rust, embedded (no separate server)
- ACID transactions
- Very fast (comparable to LMDB)
- Simple API
- Small binary size

**Schema:**

```rust
// Workflow executions
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub inputs: serde_json::Value,
    pub outputs: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

// Agent messages (conversation history)
pub struct AgentMessage {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub agent_id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tokens: Option<usize>,
}

pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

// Tool executions (audit log)
pub struct ToolExecution {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub agent_id: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

// Checkpoints (for pause/resume)
pub struct Checkpoint {
    pub execution_id: Uuid,
    pub completed_agents: Vec<String>,
    pub context_snapshot: Vec<u8>, // Serialized ExecutionContext
    pub timestamp: DateTime<Utc>,
}
```

**Database Operations:**

```rust
pub struct StateStore {
    db: redb::Database,
}

impl StateStore {
    pub async fn save_execution(&self, execution: &WorkflowExecution) 
        -> Result<()>;
    
    pub async fn get_execution(&self, id: Uuid) 
        -> Result<Option<WorkflowExecution>>;
    
    pub async fn list_executions(&self, limit: usize) 
        -> Result<Vec<WorkflowExecution>>;
    
    pub async fn save_message(&self, message: &AgentMessage) 
        -> Result<()>;
    
    pub async fn get_conversation(&self, execution_id: Uuid) 
        -> Result<Vec<AgentMessage>>;
    
    pub async fn save_tool_execution(&self, tool_exec: &ToolExecution) 
        -> Result<()>;
    
    pub async fn create_checkpoint(&self, checkpoint: &Checkpoint) 
        -> Result<()>;
    
    pub async fn get_latest_checkpoint(&self, execution_id: Uuid) 
        -> Result<Option<Checkpoint>>;
}
```
### 6. API Layer (`src/api/`)

Provides REST and WebSocket interfaces for the UI and external integrations.

#### REST Endpoints

**Workflow Management**
```rust
// POST /api/workflows
// Create a new workflow
pub async fn create_workflow(
    Json(definition): Json<WorkflowDefinition>
) -> Result<Json<WorkflowResponse>>;

// GET /api/workflows
// List all workflows
pub async fn list_workflows(
    Query(params): Query<ListParams>
) -> Result<Json<Vec<WorkflowSummary>>>;

// GET /api/workflows/{id}
// Get workflow details
pub async fn get_workflow(
    Path(id): Path<String>
) -> Result<Json<WorkflowDefinition>>;

// PUT /api/workflows/{id}
// Update workflow
pub async fn update_workflow(
    Path(id): Path<String>,
    Json(definition): Json<WorkflowDefinition>
) -> Result<Json<WorkflowResponse>>;

// DELETE /api/workflows/{id}
// Delete workflow
pub async fn delete_workflow(
    Path(id): Path<String>
) -> Result<StatusCode>;
```

**Execution Management**
```rust
// POST /api/workflows/{id}/execute
// Start workflow execution
pub async fn execute_workflow(
    Path(id): Path<String>,
    Json(inputs): Json<HashMap<String, Value>>
) -> Result<Json<ExecutionResponse>>;

// GET /api/executions/{id}
// Get execution status and results
pub async fn get_execution(
    Path(id): Path<Uuid>
) -> Result<Json<ExecutionDetails>>;

// GET /api/executions
// List recent executions
pub async fn list_executions(
    Query(params): Query<ListParams>
) -> Result<Json<Vec<ExecutionSummary>>>;

// POST /api/executions/{id}/pause
// Pause running execution
pub async fn pause_execution(
    Path(id): Path<Uuid>
) -> Result<StatusCode>;

// POST /api/executions/{id}/resume
// Resume paused execution
pub async fn resume_execution(
    Path(id): Path<Uuid>
) -> Result<StatusCode>;

// DELETE /api/executions/{id}
// Cancel execution
pub async fn cancel_execution(
    Path(id): Path<Uuid>
) -> Result<StatusCode>;
```

**Agent & Tool Management**
```rust
// GET /api/agents
// List available agent types
pub async fn list_agents() -> Result<Json<Vec<AgentDefinition>>>;

// GET /api/tools
// List available tools
pub async fn list_tools() -> Result<Json<Vec<ToolInfo>>>;

// GET /api/tools/{name}
// Get tool details and parameters
pub async fn get_tool(
    Path(name): Path<String>
) -> Result<Json<ToolDetails>>;
```

**Configuration**
```rust
// GET /api/config
// Get current configuration
pub async fn get_config() -> Result<Json<GlobalConfig>>;

// PUT /api/config
// Update configuration
pub async fn update_config(
    Json(config): Json<GlobalConfig>
) -> Result<StatusCode>;

// GET /api/config/llm-providers
// List configured LLM providers
pub async fn list_llm_providers() 
    -> Result<Json<Vec<LLMProviderInfo>>>;
```

#### WebSocket Interface

**Real-time Updates**
```rust
// WS /api/ws/executions/{id}
// Subscribe to execution events

pub enum WebSocketMessage {
    // Server -> Client
    ExecutionStarted { execution_id: Uuid, timestamp: DateTime },
    AgentStarted { agent_id: String, task: String },
    AgentProgress { agent_id: String, message: String },
    ToolExecuted { agent_id: String, tool: String, result: ToolResult },
    AgentCompleted { agent_id: String, output: AgentOutput },
    ExecutionCompleted { execution_id: Uuid, output: Value },
    ExecutionFailed { execution_id: Uuid, error: String },
    
    // Client -> Server
    Subscribe { execution_id: Uuid },
    Unsubscribe { execution_id: Uuid },
    PauseExecution { execution_id: Uuid },
    ResumeExecution { execution_id: Uuid },
    CancelExecution { execution_id: Uuid },
}
```

**Implementation:**
```rust
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to event bus
    let mut event_rx = state.event_bus.subscribe();
    
    // Forward events to WebSocket
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            handle_client_message(&text, &state).await;
        }
    }
}
```

#### Error Responses

```rust
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

// Standard error codes:
// - WORKFLOW_NOT_FOUND
// - EXECUTION_NOT_FOUND
// - INVALID_WORKFLOW_DEFINITION
// - AGENT_TIMEOUT
// - TOOL_PERMISSION_DENIED
// - LLM_PROVIDER_UNAVAILABLE
// - INTERNAL_ERROR
```

---

### 7. UI Layer (`src/ui/`)

Hybrid approach: Web UI + optional Tauri wrapper for desktop experience.

#### Web UI Architecture

**Technology Stack:**
- **Framework**: React or Svelte (decision: Svelte for smaller bundle size)
- **Workflow Builder**: Svelte Flow (visual node editor)
- **State Management**: Svelte stores + WebSocket for real-time
- **Styling**: Tailwind CSS (utility-first, fast development)
- **Build**: Vite (fast HMR, optimized builds)

**Why Svelte:**
- Smaller bundle size than React (~50% smaller)
- Better performance (compiles to vanilla JS)
- Simpler syntax, faster development
- Great TypeScript support
- Excellent for real-time updates

#### Core UI Components

**1. Workflow Builder**
```svelte
<!-- WorkflowBuilder.svelte -->
<script lang="ts">
  import { SvelteFlow, Background, Controls } from '@xyflow/svelte';
  
  let nodes = [];
  let edges = [];
  
  function onNodeAdd(type: string) {
    // Add new agent node
  }
  
  function onConnect(connection) {
    // Connect agents (sequential flow)
  }
  
  function onSave() {
    // Convert visual graph to YAML workflow
  }
</script>

<div class="workflow-builder">
  <Toolbar on:add-node={onNodeAdd} />
  
  <SvelteFlow 
    {nodes} 
    {edges}
    on:connect={onConnect}
  >
    <Background />
    <Controls />
  </SvelteFlow>
  
  <PropertiesPanel bind:selectedNode />
</div>
```

**Node Types:**
- **Agent Node**: Configure agent type, task, tools, LLM
- **Input Node**: Define workflow inputs
- **Output Node**: Configure output format and destination
- **Merge Node**: Configure merge strategy for parallel flows

**2. Execution Monitor**
```svelte
<!-- ExecutionMonitor.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { writable } from 'svelte/store';
  
  export let executionId: string;
  
  const events = writable([]);
  const status = writable('running');
  
  onMount(() => {
    const ws = new WebSocket(`ws://localhost:3000/api/ws/executions/${executionId}`);
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      events.update(e => [...e, data]);
      
      if (data.type === 'ExecutionCompleted') {
        status.set('completed');
      }
    };
    
    return () => ws.close();
  });
</script>

<div class="execution-monitor">
  <StatusBadge {$status} />
  
  <Timeline>
    {#each $events as event}
      <TimelineItem {event} />
    {/each}
  </Timeline>
  
  <ConversationView {executionId} />
  <ToolExecutionLog {executionId} />
</div>
```

**3. Agent Conversation View**
```svelte
<!-- ConversationView.svelte -->
<script lang="ts">
  export let executionId: string;
  
  let messages = [];
  
  // Real-time message updates via WebSocket
</script>

<div class="conversation">
  {#each messages as msg}
    <Message 
      role={msg.role}
      content={msg.content}
      timestamp={msg.timestamp}
    />
  {/each}
</div>
```

**4. Settings Panel**
```svelte
<!-- Settings.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  
  let config = {};
  
  onMount(async () => {
    const res = await fetch('/api/config');
    config = await res.json();
  });
  
  async function saveConfig() {
    await fetch('/api/config', {
      method: 'PUT',
      body: JSON.stringify(config)
    });
  }
</script>

<div class="settings">
  <Section title="LLM Providers">
    <LLMProviderConfig bind:config={config.llm} />
  </Section>
  
  <Section title="Permissions">
    <PermissionConfig bind:config={config.permissions} />
  </Section>
  
  <Section title="Execution">
    <ExecutionConfig bind:config={config.execution} />
  </Section>
  
  <Button on:click={saveConfig}>Save</Button>
</div>
```

#### Tauri Desktop Wrapper (Optional)

**Why Tauri:**
- Rust-based (consistent with backend)
- Small binary size (~3-5 MB)
- Native system integration
- Secure (no Node.js runtime)
- Cross-platform (Windows, macOS, Linux)

**Desktop Features:**
```rust
// src-tauri/src/main.rs

#[tauri::command]
async fn start_workflow(workflow_id: String) -> Result<String, String> {
    // Call backend API
}

#[tauri::command]
async fn select_file() -> Result<String, String> {
    // Native file picker
}

fn main() {
    tauri::Builder::default()
        .system_tray(SystemTray::new())
        .on_system_tray_event(|app, event| {
            // System tray menu
        })
        .invoke_handler(tauri::generate_handler![
            start_workflow,
            select_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Desktop-Specific Features:**
- System tray icon with quick actions
- Native notifications for workflow completion
- File system dialogs (open/save)
- Auto-start on boot (optional)
- Global keyboard shortcuts

#### UI/UX Principles

**1. Progressive Disclosure**
- Simple workflows: Just drag-drop agents
- Advanced: Expose YAML editor for power users
- Expert: Direct file editing with live preview

**2. Real-time Feedback**
- WebSocket updates for execution progress
- Live agent conversation display
- Tool execution logs in real-time

**3. Error Clarity**
- Clear error messages with suggestions
- Highlight problematic nodes in workflow
- Link to relevant documentation

**4. Performance**
- Lazy load workflow list
- Virtual scrolling for long conversations
- Debounced auto-save
- Optimistic UI updates
### 8. Config & CLI Layer (`src/config/`, `src/cli/`)

Provides global configuration management and command-line interface.

#### Global Configuration

```rust
pub struct GlobalConfig {
    pub llm: LLMConfig,
    pub execution: ExecutionConfig,
    pub permissions: PermissionConfig,
    pub ui: UIConfig,
    pub logging: LoggingConfig,
    pub data_dir: PathBuf,
}

pub struct LLMConfig {
    pub default_provider: String, // "ollama:llama3"
    pub providers: HashMap<String, ProviderConfig>,
    pub fallback_enabled: bool,
}

pub struct ProviderConfig {
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
    pub timeout: Duration,
}

pub struct ExecutionConfig {
    pub max_parallel_agents: usize,
    pub default_timeout: Duration,
    pub enable_checkpoints: bool,
    pub checkpoint_interval: Duration,
}

pub struct PermissionConfig {
    pub default_policy: PolicyAction,
    pub rules: Vec<PermissionRule>,
    pub audit_log_enabled: bool,
}

pub struct UIConfig {
    pub port: u16,
    pub auto_open_browser: bool,
    pub enable_cors: bool,
}

pub struct LoggingConfig {
    pub level: String, // trace, debug, info, warn, error
    pub format: LogFormat,
    pub output: LogOutput,
}

pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

pub enum LogOutput {
    Stdout,
    File { path: PathBuf },
    Both { path: PathBuf },
}
```

#### Configuration Loading

```rust
impl GlobalConfig {
    pub fn load() -> Result<Self> {
        // Priority order:
        // 1. Environment variables (RUSTFORGE_*)
        // 2. .rustforge/config.toml (project-specific)
        // 3. ~/.rustforge/config.toml (user-global)
        // 4. Default values
        
        let mut config = Self::default();
        
        // Load from user-global config
        if let Some(user_config) = Self::load_user_config()? {
            config.merge(user_config);
        }
        
        // Load from project config
        if let Some(project_config) = Self::load_project_config()? {
            config.merge(project_config);
        }
        
        // Override with environment variables
        config.apply_env_overrides();
        
        Ok(config)
    }
    
    pub fn save(&self, scope: ConfigScope) -> Result<()> {
        let path = match scope {
            ConfigScope::User => Self::user_config_path(),
            ConfigScope::Project => Self::project_config_path(),
        };
        
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }
}
```

#### CLI Interface (clap)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustforge")]
#[command(about = "Local AI Agent Orchestrator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Config file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web UI
    Ui {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Don't open browser automatically
        #[arg(long)]
        no_browser: bool,
    },
    
    /// Run a workflow
    Run {
        /// Path to workflow YAML file
        workflow: PathBuf,
        
        /// Workflow inputs (JSON)
        #[arg(short, long)]
        inputs: Option<String>,
        
        /// Watch mode (re-run on file changes)
        #[arg(short, long)]
        watch: bool,
    },
    
    /// List workflows
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Show execution logs
    Logs {
        /// Execution ID
        execution_id: Uuid,
        
        /// Follow logs in real-time
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Resume a paused execution
    Resume {
        /// Execution ID
        execution_id: Uuid,
    },
    
    /// Agent management
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    
    /// Tool management
    Tool {
        #[command(subcommand)]
        command: ToolCommands,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    
    /// Initialize a new RustForge project
    Init {
        /// Project directory
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// List available agents
    List,
    
    /// Show agent details
    Info { agent_id: String },
}

#[derive(Subcommand)]
enum ToolCommands {
    /// List available tools
    List,
    
    /// Show tool details
    Info { tool_name: String },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    
    /// Set a configuration value
    Set {
        key: String,
        value: String,
        
        /// Save to user config (default: project)
        #[arg(long)]
        global: bool,
    },
    
    /// Get a configuration value
    Get { key: String },
}
```

#### CLI Usage Examples

```bash
# Start web UI
rustforge ui
rustforge ui --port 8080 --no-browser

# Run a workflow
rustforge run workflows/pdf-research.yaml
rustforge run workflows/web-scraper.yaml --inputs '{"url": "https://example.com"}'
rustforge run workflows/analysis.yaml --watch

# List workflows
rustforge list
rustforge list --detailed

# View execution logs
rustforge logs 550e8400-e29b-41d4-a716-446655440000
rustforge logs 550e8400-e29b-41d4-a716-446655440000 --follow

# Resume paused execution
rustforge resume 550e8400-e29b-41d4-a716-446655440000

# Agent management
rustforge agent list
rustforge agent info ResearchAgent

# Tool management
rustforge tool list
rustforge tool info web_scraper

# Configuration
rustforge config show
rustforge config set default_llm ollama:llama3
rustforge config set default_llm openai:gpt-4o --global
rustforge config get default_llm

# Initialize new project
rustforge init
rustforge init my-project
```

#### Project Initialization

```rust
pub fn init_project(path: Option<PathBuf>) -> Result<()> {
    let project_dir = path.unwrap_or_else(|| PathBuf::from("."));
    
    // Create directory structure
    fs::create_dir_all(project_dir.join(".rustforge"))?;
    fs::create_dir_all(project_dir.join("workflows"))?;
    fs::create_dir_all(project_dir.join("reports"))?;
    
    // Create default config
    let config = GlobalConfig::default();
    config.save_to(project_dir.join(".rustforge/config.toml"))?;
    
    // Create example workflow
    let example_workflow = include_str!("../templates/example-workflow.yaml");
    fs::write(
        project_dir.join("workflows/example.yaml"),
        example_workflow
    )?;
    
    // Create .gitignore
    let gitignore = include_str!("../templates/.gitignore");
    fs::write(project_dir.join(".gitignore"), gitignore)?;
    
    println!("✓ Initialized RustForge project at {}", project_dir.display());
    println!("\nNext steps:");
    println!("  1. Edit workflows/example.yaml");
    println!("  2. Run: rustforge run workflows/example.yaml");
    println!("  3. Or start UI: rustforge ui");
    
    Ok(())
}
```

---

## Execution Patterns

### Sequential Execution

**Flow:**
```
User → Workflow Definition
  ↓
Parse & validate DAG
  ↓
Agent 1 executes → Output 1
  ↓
Agent 2 receives Output 1 → Output 2
  ↓
Agent 3 receives Output 2 → Final Output
  ↓
Post-process (optional)
  ↓
Save results to DB
```

**Implementation:**
```rust
pub async fn execute_sequential(
    workflow: &WorkflowDefinition,
    context: &mut ExecutionContext,
) -> Result<AgentOutput> {
    for agent_config in &workflow.agents {
        // Interpolate variables in task
        let task = interpolate_variables(&agent_config.task, context)?;
        
        // Create agent instance
        let mut agent = create_agent(agent_config)?;
        
        // Execute with timeout
        let output = tokio::time::timeout(
            agent_config.timeout.unwrap_or(Duration::from_secs(300)),
            agent.execute(task, context)
        ).await??;
        
        // Store output in context
        context.set_agent_output(&agent_config.id, output.clone());
        
        // Publish event
        context.event_bus.publish(AgentEvent::AgentCompleted {
            agent_id: agent_config.id.clone(),
            output: output.clone(),
        })?;
        
        // Create checkpoint
        if context.config.enable_checkpoints {
            create_checkpoint(context).await?;
        }
    }
    
    // Get final output
    let final_output = context.get_agent_output(
        &workflow.agents.last().unwrap().id
    ).unwrap();
    
    Ok(final_output.clone())
}
```

**Example Workflow:**
```yaml
name: "PDF Research Pipeline"
mode: sequential

agents:
  - id: pdf_reader
    type: ResearchAgent
    task: "Read PDF from {input.pdf_path} and extract key points"
    tools: [pdf_parser]
  
  - id: summarizer
    type: AnalysisAgent
    task: "Summarize: {pdf_reader.output}"
  
  - id: writer
    type: AnalysisAgent
    task: "Write markdown report: {summarizer.output}"
    tools: [file_system]
```

### Parallel Execution

**Flow:**
```
User → Workflow Definition
  ↓
Parse & validate
  ↓
Spawn multiple agents concurrently
  ↓
Agent 1 ──┐
Agent 2 ──┼─→ Execute in parallel
Agent 3 ──┘
  ↓
Collect results (handle timeouts/failures)
  ↓
Apply merge strategy
  ↓
Post-process (optional)
  ↓
Save results
```

**Implementation:**
```rust
pub async fn execute_parallel(
    workflow: &WorkflowDefinition,
    context: &mut ExecutionContext,
    merge_strategy: &MergeStrategy,
) -> Result<AgentOutput> {
    let mut tasks = Vec::new();
    
    // Spawn all agents
    for agent_config in &workflow.agents {
        let task = interpolate_variables(&agent_config.task, context)?;
        let mut agent = create_agent(agent_config)?;
        let ctx = context.clone();
        
        let handle = tokio::spawn(async move {
            let result = tokio::time::timeout(
                agent_config.timeout.unwrap_or(Duration::from_secs(300)),
                agent.execute(task, &ctx)
            ).await;
            
            (agent_config.id.clone(), result)
        });
        
        tasks.push(handle);
    }
    
    // Wait for all agents
    let results = futures::future::join_all(tasks).await;
    
    // Collect successful outputs
    let mut outputs = Vec::new();
    for result in results {
        match result {
            Ok((agent_id, Ok(Ok(output)))) => {
                outputs.push(output);
                context.set_agent_output(&agent_id, output.clone());
            }
            Ok((agent_id, Ok(Err(e)))) => {
                eprintln!("Agent {} failed: {}", agent_id, e);
            }
            Ok((agent_id, Err(_))) => {
                eprintln!("Agent {} timed out", agent_id);
            }
            Err(e) => {
                eprintln!("Task join error: {}", e);
            }
        }
    }
    
    // Apply merge strategy
    let merged = apply_merge_strategy(outputs, merge_strategy, context).await?;
    
    Ok(merged)
}
```

**Merge Strategies:**

```rust
pub async fn apply_merge_strategy(
    outputs: Vec<AgentOutput>,
    strategy: &MergeStrategy,
    context: &ExecutionContext,
) -> Result<AgentOutput> {
    match strategy {
        MergeStrategy::Concat => {
            // Concatenate all outputs
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
            // Find most common output
            let mut counts = HashMap::new();
            for output in &outputs {
                *counts.entry(&output.content).or_insert(0) += 1;
            }
            
            let most_common = counts.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(content, _)| (*content).clone())
                .unwrap();
            
            Ok(AgentOutput {
                content: most_common,
                metadata: HashMap::new(),
            })
        }
        
        MergeStrategy::LlmMerge { agent_type, prompt } => {
            // Use LLM to intelligently merge
            let combined_results = outputs.iter()
                .enumerate()
                .map(|(i, o)| format!("Result {}:\n{}", i + 1, o.content))
                .collect::<Vec<_>>()
                .join("\n\n");
            
            let merge_prompt = prompt.replace("{results}", &combined_results);
            
            let mut merge_agent = create_agent_by_type(agent_type)?;
            let merged = merge_agent.execute(
                Task::new(merge_prompt),
                context
            ).await?;
            
            Ok(merged)
        }
        
        MergeStrategy::Custom { script } => {
            // v1.0: Execute custom merge script
            unimplemented!("Custom merge strategy (v1.0)")
        }
    }
}
```

**Example Workflow:**
```yaml
name: "Multi-Source Research"
mode: parallel
merge_strategy:
  type: llm_merge
  agent_type: AnalysisAgent
  prompt: "Combine these research results, remove duplicates, synthesize insights: {results}"

agents:
  - id: google_search
    type: ResearchAgent
    task: "Search Google for: {input.query}"
    tools: [web_scraper]
  
  - id: bing_search
    type: ResearchAgent
    task: "Search Bing for: {input.query}"
    tools: [web_scraper]
  
  - id: duckduckgo_search
    type: ResearchAgent
    task: "Search DuckDuckGo for: {input.query}"
    tools: [web_scraper]
```

### Supervisor Pattern (v1.0)

**Flow:**
```
User → High-level task
  ↓
Supervisor Agent
  ├─→ Analyze task
  ├─→ Decompose into sub-tasks
  ├─→ Select worker agents
  ↓
Loop:
  ├─→ Assign sub-tasks to workers
  ├─→ Monitor worker progress
  ├─→ Validate results
  ├─→ Request corrections if needed
  ├─→ Decide next action
  └─→ Break if complete
  ↓
Synthesize final output
```

**Note:** Supervisor pattern is deferred to v1.0 due to complexity. MVP focuses on Sequential and Parallel patterns.
## Error Handling and Recovery

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    // Agent errors
    #[error("Agent {agent_id} timed out after {timeout_secs}s")]
    AgentTimeout { agent_id: String, timeout_secs: u64 },
    
    #[error("Agent {agent_id} failed: {error}")]
    AgentFailed { agent_id: String, error: String },
    
    #[error("Agent {agent_id} crashed with exit code {exit_code}")]
    AgentCrashed { agent_id: String, exit_code: i32 },
    
    // Tool errors
    #[error("Tool '{tool_name}' not found")]
    ToolNotFound { tool_name: String },
    
    #[error("Permission denied for tool '{tool_name}': {reason}")]
    ToolPermissionDenied { tool_name: String, reason: String },
    
    #[error("Tool '{tool_name}' execution failed: {error}")]
    ToolExecutionFailed { tool_name: String, error: String },
    
    // LLM errors
    #[error("LLM provider '{provider}' unavailable")]
    LLMProviderUnavailable { provider: String },
    
    #[error("Rate limit exceeded for '{provider}', retry after {retry_after:?}")]
    LLMRateLimitExceeded { provider: String, retry_after: Duration },
    
    #[error("Context too long: {tokens} tokens (max: {max_tokens})")]
    LLMContextTooLong { tokens: usize, max_tokens: usize },
    
    // Workflow errors
    #[error("Invalid workflow definition: {reason}")]
    InvalidWorkflowDefinition { reason: String },
    
    #[error("Variable '{variable}' not found. Did you mean: {suggestions:?}")]
    VariableNotFound { variable: String, suggestions: Vec<String> },
    
    #[error("Circular dependency detected: {agents:?}")]
    CircularDependency { agents: Vec<String> },
}
```

### Retry Strategy

**Configuration:**
```yaml
agents:
  - id: web_scraper
    retry:
      max_attempts: 3
      backoff: exponential  # linear | exponential | fixed
      initial_delay: 1s
      max_delay: 30s
      retry_on:
        - ToolExecutionFailed
        - LLMRateLimitExceeded
        - NetworkError
```

**Implementation:**
```rust
pub struct RetryConfig {
    pub max_attempts: usize,
    pub backoff: BackoffStrategy,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub retry_on: Vec<ErrorType>,
}

pub enum BackoffStrategy {
    Linear,
    Exponential,
    Fixed,
}

pub async fn execute_with_retry<F, T>(
    operation: F,
    config: &RetryConfig,
) -> Result<T>
where
    F: Fn() -> BoxFuture<'static, Result<T>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= config.max_attempts => return Err(e),
            Err(e) if config.should_retry(&e) => {
                eprintln!("Attempt {} failed: {}. Retrying in {:?}...", 
                    attempt, e, delay);
                
                tokio::time::sleep(delay).await;
                
                // Calculate next delay
                delay = match config.backoff {
                    BackoffStrategy::Linear => 
                        (delay + config.initial_delay).min(config.max_delay),
                    BackoffStrategy::Exponential => 
                        (delay * 2).min(config.max_delay),
                    BackoffStrategy::Fixed => 
                        config.initial_delay,
                };
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Checkpoint and Resume

**Purpose:** Allow workflows to be paused and resumed without losing progress.

**Checkpoint Creation:**
```rust
pub struct Checkpoint {
    pub execution_id: Uuid,
    pub workflow_id: String,
    pub completed_agents: Vec<String>,
    pub context_snapshot: Vec<u8>, // Serialized ExecutionContext
    pub timestamp: DateTime<Utc>,
}

pub async fn create_checkpoint(
    execution_id: Uuid,
    context: &ExecutionContext,
    state_store: &StateStore,
) -> Result<()> {
    let checkpoint = Checkpoint {
        execution_id,
        workflow_id: context.workflow_id.clone(),
        completed_agents: context.get_completed_agents(),
        context_snapshot: bincode::serialize(context)?,
        timestamp: Utc::now(),
    };
    
    state_store.save_checkpoint(&checkpoint).await?;
    Ok(())
}
```

**Resume Execution:**
```rust
pub async fn resume_execution(
    execution_id: Uuid,
    state_store: &StateStore,
) -> Result<ExecutionContext> {
    let checkpoint = state_store
        .get_latest_checkpoint(execution_id)
        .await?
        .ok_or(Error::CheckpointNotFound)?;
    
    let mut context: ExecutionContext = 
        bincode::deserialize(&checkpoint.context_snapshot)?;
    
    // Restore event bus subscription
    context.event_bus = Arc::new(EventBus::new());
    
    Ok(context)
}
```

**CLI Usage:**
```bash
# Pause execution
rustforge pause <execution-id>

# Resume execution
rustforge resume <execution-id>
```

### Graceful Degradation

**Parallel Execution with Partial Failures:**
- If some agents fail, continue with successful ones
- Merge strategy handles missing results
- Log failures for debugging

**LLM Fallback:**
- Primary provider fails → try fallback provider
- Both fail → return clear error with suggestions

**Tool Execution:**
- Tool fails → agent can retry or use alternative approach
- Permission denied → prompt user, don't fail silently

---

## Security and Permissions

### Permission System

**Permission Model:**
```rust
pub struct PermissionPolicy {
    pub default: PolicyAction,
    pub rules: Vec<PermissionRule>,
    pub audit_enabled: bool,
}

pub struct PermissionRule {
    pub tool: String,
    pub operations: Vec<String>,
    pub scope: Option<Scope>,
    pub action: PolicyAction,
}

pub enum PolicyAction {
    Allow,
    Deny,
    Prompt,
}

pub enum Scope {
    FilePaths(Vec<PathBuf>),
    Domains(Vec<String>),
    Commands(Vec<String>),
}
```

**Configuration Example:**
```yaml
# .rustforge/permissions.yaml
default_policy: prompt

rules:
  # File system
  - tool: file_system
    operations: [read]
    scope:
      paths: ["./data/**", "./reports/**"]
    action: allow
  
  - tool: file_system
    operations: [write, delete]
    scope:
      paths: ["./reports/**"]
    action: allow
  
  - tool: file_system
    operations: [write, delete]
    scope:
      paths: ["/", "~/.ssh/**", "~/.aws/**"]
    action: deny
  
  # Shell executor
  - tool: shell_executor
    operations: [execute]
    scope:
      commands: ["ls", "cat", "grep", "find"]
    action: allow
  
  - tool: shell_executor
    operations: [execute]
    scope:
      commands: ["rm", "sudo", "chmod"]
    action: prompt
  
  # Web scraper
  - tool: web_scraper
    operations: [fetch]
    scope:
      domains: ["*.wikipedia.org", "github.com", "*.arxiv.org"]
    action: allow
```

**Runtime Permission Check:**
```rust
impl PermissionManager {
    pub async fn check_permission(
        &self,
        tool: &str,
        operation: &str,
        scope_value: &str,
        context: &ExecutionContext,
    ) -> Result<PermissionDecision> {
        // Check rules in order
        for rule in &self.policy.rules {
            if rule.matches(tool, operation, scope_value) {
                return match rule.action {
                    PolicyAction::Allow => Ok(PermissionDecision::Allowed),
                    PolicyAction::Deny => Ok(PermissionDecision::Denied),
                    PolicyAction::Prompt => {
                        self.prompt_user(tool, operation, scope_value).await
                    }
                };
            }
        }
        
        // No rule matched, use default policy
        match self.policy.default {
            PolicyAction::Allow => Ok(PermissionDecision::Allowed),
            PolicyAction::Deny => Ok(PermissionDecision::Denied),
            PolicyAction::Prompt => {
                self.prompt_user(tool, operation, scope_value).await
            }
        }
    }
    
    async fn prompt_user(
        &self,
        tool: &str,
        operation: &str,
        scope_value: &str,
    ) -> Result<PermissionDecision> {
        println!("\n⚠️  Permission Required\n");
        println!("Tool: {}", tool);
        println!("Operation: {}", operation);
        println!("Target: {}", scope_value);
        println!("\n[A]llow once  [T]his session  [F]orever  [D]eny\n");
        
        let response = read_user_input()?;
        
        match response.to_lowercase().as_str() {
            "a" | "allow" => Ok(PermissionDecision::AllowedOnce),
            "t" | "session" => {
                self.add_session_rule(tool, operation, scope_value, true);
                Ok(PermissionDecision::Allowed)
            }
            "f" | "forever" => {
                self.add_permanent_rule(tool, operation, scope_value, true)?;
                Ok(PermissionDecision::Allowed)
            }
            "d" | "deny" => Ok(PermissionDecision::Denied),
            _ => self.prompt_user(tool, operation, scope_value).await,
        }
    }
}
```

### Process Isolation

**Agent Process Isolation:**
```rust
pub struct IsolatedAgent {
    process: Child,
    limits: ResourceLimits,
    cancellation_token: CancellationToken,
}

pub struct ResourceLimits {
    pub max_memory_mb: usize,      // 512 MB default
    pub max_cpu_percent: u8,       // 50% default
    pub max_execution_time: Duration,
    pub max_file_descriptors: usize,
}

impl IsolatedAgent {
    pub async fn spawn(
        agent_config: &AgentConfig,
        limits: ResourceLimits,
    ) -> Result<Self> {
        let mut cmd = Command::new(env::current_exe()?);
        cmd.arg("--agent-mode")
           .arg(&agent_config.id)
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Apply resource limits (platform-specific)
        #[cfg(unix)]
        {
            use nix::sys::resource::{setrlimit, Resource};
            
            // Memory limit
            setrlimit(
                Resource::RLIMIT_AS,
                limits.max_memory_mb * 1024 * 1024,
                limits.max_memory_mb * 1024 * 1024,
            )?;
            
            // File descriptor limit
            setrlimit(
                Resource::RLIMIT_NOFILE,
                limits.max_file_descriptors,
                limits.max_file_descriptors,
            )?;
        }
        
        let process = cmd.spawn()?;
        let cancellation_token = CancellationToken::new();
        
        Ok(Self {
            process,
            limits,
            cancellation_token,
        })
    }
    
    pub async fn execute(&mut self, task: Task) -> Result<AgentOutput> {
        // Send task to agent process via stdin
        let task_json = serde_json::to_string(&task)?;
        self.process.stdin.as_mut()
            .unwrap()
            .write_all(task_json.as_bytes())
            .await?;
        
        // Read output with timeout
        tokio::select! {
            result = self.read_output() => result,
            _ = tokio::time::sleep(self.limits.max_execution_time) => {
                self.terminate().await?;
                Err(Error::AgentTimeout)
            }
            _ = self.cancellation_token.cancelled() => {
                self.terminate().await?;
                Err(Error::AgentCancelled)
            }
        }
    }
    
    pub async fn terminate(&mut self) -> Result<()> {
        // Graceful shutdown: send SIGTERM, wait 5s, then SIGKILL
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            
            let pid = Pid::from_raw(self.process.id() as i32);
            
            // Send SIGTERM
            kill(pid, Signal::SIGTERM)?;
            
            // Wait up to 5 seconds
            tokio::select! {
                _ = self.process.wait() => {}
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    // Force kill
                    kill(pid, Signal::SIGKILL)?;
                    self.process.wait().await?;
                }
            }
        }
        
        #[cfg(windows)]
        {
            self.process.kill().await?;
        }
        
        Ok(())
    }
}
```

### Secret Management

**Secure Storage:**
```rust
use keyring::Entry;

pub struct SecretManager {
    service_name: String,
}

impl SecretManager {
    pub fn new() -> Self {
        Self {
            service_name: "rustforge".to_string(),
        }
    }
    
    pub fn store_secret(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.set_password(value)?;
        Ok(())
    }
    
    pub fn get_secret(&self, key: &str) -> Result<String> {
        let entry = Entry::new(&self.service_name, key)?;
        Ok(entry.get_password()?)
    }
    
    pub fn delete_secret(&self, key: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.delete_password()?;
        Ok(())
    }
}

// Usage in config
pub fn load_api_key(provider: &str) -> Result<String> {
    let secret_manager = SecretManager::new();
    
    // Try environment variable first
    if let Ok(key) = env::var(format!("{}_API_KEY", provider.to_uppercase())) {
        return Ok(key);
    }
    
    // Try system keyring
    secret_manager.get_secret(&format!("{}_api_key", provider))
}
```

### Audit Logging

**Audit Log Structure:**
```rust
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub execution_id: Uuid,
    pub agent_id: String,
    pub action: AuditAction,
    pub result: AuditResult,
    pub user: Option<String>,
}

pub enum AuditAction {
    ToolExecuted {
        tool: String,
        operation: String,
        parameters: serde_json::Value,
    },
    FileAccessed {
        path: PathBuf,
        operation: FileOperation,
    },
    NetworkRequest {
        url: String,
        method: String,
    },
    PermissionGranted {
        tool: String,
        scope: String,
        duration: PermissionDuration,
    },
    PermissionDenied {
        tool: String,
        scope: String,
        reason: String,
    },
}

pub enum AuditResult {
    Success,
    Failed { error: String },
    Denied { reason: String },
}
```

**Audit Log Storage:**
- Stored in redb database
- Indexed by execution_id, agent_id, timestamp
- Queryable via CLI: `rustforge audit --execution-id <id>`
- Exportable to JSON/CSV for compliance
## Data Flow and Context Management

### Variable Interpolation

**Supported Variable Types:**
```yaml
# Workflow inputs
task: "Process file: {input.file_path}"

# Previous agent outputs
task: "Summarize: {agent_id.output}"

# Nested field access
task: "Use key points: {agent_id.output.key_points}"

# Shared memory
task: "Reference findings: {memory.agent_id.research_data}"

# Environment variables
task: "Use API key: {env.API_KEY}"

# Global config
task: "Using LLM: {config.default_llm}"

# Workflow metadata
output_path: "reports/{workflow.name}_{timestamp}.md"
```

**Interpolation Engine:**
```rust
pub struct VariableInterpolator {
    context: Arc<ExecutionContext>,
}

impl VariableInterpolator {
    pub fn interpolate(&self, template: &str) -> Result<String> {
        let mut result = template.to_string();
        
        // Find all variables: {variable.path}
        let re = Regex::new(r"\{([^}]+)\}").unwrap();
        
        for cap in re.captures_iter(template) {
            let var_path = &cap[1];
            let value = self.resolve_variable(var_path)?;
            result = result.replace(&cap[0], &value);
        }
        
        Ok(result)
    }
    
    fn resolve_variable(&self, path: &str) -> Result<String> {
        let parts: Vec<&str> = path.split('.').collect();
        
        match parts[0] {
            "input" => self.resolve_input(&parts[1..]),
            "env" => self.resolve_env(&parts[1..]),
            "config" => self.resolve_config(&parts[1..]),
            "memory" => self.resolve_memory(&parts[1..]),
            "workflow" => self.resolve_workflow_meta(&parts[1..]),
            agent_id => self.resolve_agent_output(agent_id, &parts[1..]),
        }
    }
    
    fn resolve_agent_output(&self, agent_id: &str, path: &[&str]) -> Result<String> {
        let output = self.context.get_agent_output(agent_id)
            .ok_or_else(|| Error::VariableNotFound {
                variable: agent_id.to_string(),
                suggestions: self.suggest_similar_agents(agent_id),
            })?;
        
        if path.is_empty() {
            // {agent_id.output} -> full output
            Ok(output.content.clone())
        } else {
            // {agent_id.output.field} -> nested field
            self.extract_field(&output.content, path)
        }
    }
    
    fn suggest_similar_agents(&self, agent_id: &str) -> Vec<String> {
        // Levenshtein distance for suggestions
        self.context.get_completed_agents()
            .into_iter()
            .filter(|id| levenshtein_distance(id, agent_id) <= 2)
            .collect()
    }
}
```

**Error Messages:**
```
Error: Variable interpolation failed
  Variable: {pdf_reader.output.key_points}
  Reason: Agent 'pdf_reader' output does not contain field 'key_points'
  Available fields: ['summary', 'page_count', 'metadata']
  Suggestion: Use {pdf_reader.output.summary} instead

Error: Variable interpolation failed
  Variable: {pdf_reeder.output}
  Reason: Agent 'pdf_reeder' not found
  Did you mean: 'pdf_reader'?
```

### Execution Context

```rust
pub struct ExecutionContext {
    pub workflow_id: String,
    pub execution_id: Uuid,
    
    // Flexible context store
    pub context_store: HashMap<String, serde_json::Value>,
    
    // Shared resources
    pub shared_memory: Arc<RwLock<dyn MemoryStore>>,
    pub event_bus: Arc<EventBus>,
    pub permission_manager: Arc<PermissionManager>,
    pub llm_registry: Arc<LLMRegistry>,
    pub tool_registry: Arc<ToolRegistry>,
    
    // Configuration
    pub config: Arc<GlobalConfig>,
}

impl ExecutionContext {
    pub fn get_input(&self, key: &str) -> Option<&Value> {
        self.context_store.get(&format!("input.{}", key))
    }
    
    pub fn get_agent_output(&self, agent_id: &str) -> Option<&Value> {
        self.context_store.get(&format!("agent.{}.output", agent_id))
    }
    
    pub fn set_agent_output(&mut self, agent_id: &str, output: AgentOutput) {
        self.context_store.insert(
            format!("agent.{}.output", agent_id),
            serde_json::to_value(output).unwrap()
        );
    }
    
    pub fn get_completed_agents(&self) -> Vec<String> {
        self.context_store.keys()
            .filter_map(|k| {
                if k.starts_with("agent.") && k.ends_with(".output") {
                    Some(k.strip_prefix("agent.")
                        .unwrap()
                        .strip_suffix(".output")
                        .unwrap()
                        .to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}
```

---

## Performance Targets and Benchmarks

### Target Metrics (MVP)

**Startup Performance:**
- Binary startup time: < 100ms
- Web UI ready: < 500ms
- First workflow execution: < 1s (after startup)

**Runtime Performance:**
- Agent spawn time: < 50ms per agent
- Parallel execution: 10+ agents simultaneously
- Context switching overhead: < 5ms
- Event bus latency: < 1ms

**Memory Footprint:**
- Baseline (idle): 50-100 MB
- With 1 active workflow: 200-400 MB
- With 10 parallel agents: 500-800 MB
- Maximum (under load): < 1 GB

**Comparison to Python Alternatives:**
- 5-10x faster execution
- 5-10x lower memory usage
- 10x faster startup time

### Benchmark Suite

**Benchmarks to Implement:**
```rust
// benches/workflow_execution.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sequential_workflow(c: &mut Criterion) {
    c.bench_function("sequential_3_agents", |b| {
        b.iter(|| {
            // Execute workflow with 3 sequential agents
        });
    });
}

fn bench_parallel_workflow(c: &mut Criterion) {
    c.bench_function("parallel_10_agents", |b| {
        b.iter(|| {
            // Execute workflow with 10 parallel agents
        });
    });
}

fn bench_agent_spawn(c: &mut Criterion) {
    c.bench_function("agent_spawn", |b| {
        b.iter(|| {
            // Spawn single agent
        });
    });
}

fn bench_variable_interpolation(c: &mut Criterion) {
    c.bench_function("interpolate_complex", |b| {
        b.iter(|| {
            // Interpolate template with 10+ variables
        });
    });
}

criterion_group!(
    benches,
    bench_sequential_workflow,
    bench_parallel_workflow,
    bench_agent_spawn,
    bench_variable_interpolation
);
criterion_main!(benches);
```

**Real-World Benchmark Workflows:**
1. **PDF Processing**: Read 10 PDFs, extract text, summarize each
2. **Web Research**: Scrape 20 URLs in parallel, aggregate results
3. **File Analysis**: Scan 1000 files, categorize, generate report
4. **API Integration**: Make 50 API calls, process responses

**Benchmark Reporting:**
- Publish results on GitHub README
- Compare against: LangChain, CrewAI, AutoGPT
- Update benchmarks with each release
- CI/CD integration for regression detection

---

## Technology Stack Summary

### Core Dependencies

**Runtime & Async:**
- `tokio` (1.x) - Async runtime
- `async-trait` - Async trait support
- `futures` - Future combinators

**Web & API:**
- `axum` (0.7.x) - Web framework
- `tower` - Middleware
- `tower-http` - HTTP middleware (CORS, tracing)
- `serde` + `serde_json` - Serialization
- `reqwest` - HTTP client

**Storage:**
- `redb` (2.x) - Embedded database
- `bincode` - Binary serialization for checkpoints

**LLM Integration:**
- `ollama-rs` - Ollama client
- Custom OpenAI/Anthropic clients (reqwest-based)

**Tools:**
- `scraper` - HTML parsing
- `pdf-extract` or `lopdf` - PDF parsing
- `arboard` - Clipboard access

**CLI & Config:**
- `clap` (4.x) - CLI parsing
- `toml` - TOML parsing
- `serde_yaml` - YAML parsing

**Security:**
- `keyring` - OS keyring integration
- `sha2` - Hashing
- `uuid` - UUID generation

**Logging & Tracing:**
- `tracing` - Structured logging
- `tracing-subscriber` - Log formatting

**Error Handling:**
- `thiserror` - Error derive macros
- `anyhow` - Error context

**Testing:**
- `criterion` - Benchmarking
- `mockall` - Mocking
- `tempfile` - Temporary files for tests

### UI Dependencies

**Frontend (Svelte):**
- `svelte` (5.x)
- `@xyflow/svelte` - Workflow builder
- `tailwindcss` - Styling
- `vite` - Build tool
- `typescript` - Type safety

**Desktop (Tauri):**
- `tauri` (2.x)
- `tauri-plugin-shell` - Shell commands
- `tauri-plugin-fs` - File system access

---

## Project Structure

```
rustforge/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
│
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports
│   │
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs         # CLI command handlers
│   │   └── init.rs             # Project initialization
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   ├── global.rs           # GlobalConfig
│   │   └── loader.rs           # Config loading logic
│   │
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── executor.rs         # Workflow executor
│   │   ├── coordinator.rs      # Agent coordinator
│   │   ├── permissions.rs      # Permission manager
│   │   └── events.rs           # Event bus
│   │
│   ├── agents/
│   │   ├── mod.rs
│   │   ├── traits.rs           # Agent trait
│   │   ├── registry.rs         # Agent registry
│   │   ├── research.rs         # ResearchAgent
│   │   ├── analysis.rs         # AnalysisAgent
│   │   └── supervisor.rs       # SupervisorAgent (v1.0)
│   │
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── traits.rs           # LLMProvider trait
│   │   ├── registry.rs         # LLM registry with fallback
│   │   ├── ollama.rs           # OllamaProvider
│   │   ├── openai.rs           # OpenAIProvider
│   │   ├── anthropic.rs        # AnthropicProvider (v1.0)
│   │   ├── groq.rs             # GroqProvider (v1.0)
│   │   └── context.rs          # Context management
│   │
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── traits.rs           # Tool trait
│   │   ├── registry.rs         # Tool registry
│   │   ├── filesystem.rs       # FileSystemTool
│   │   ├── web_scraper.rs      # WebScraperTool
│   │   ├── pdf_parser.rs       # PdfParserTool
│   │   ├── shell.rs            # ShellExecutorTool
│   │   ├── api_client.rs       # ApiClientTool
│   │   ├── clipboard.rs        # ClipboardTool
│   │   └── script_runner.rs    # Python/Lua runner (v1.0)
│   │
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── state.rs            # StateStore (redb)
│   │   ├── config.rs           # Config file management
│   │   └── memory.rs           # MemoryStore implementations
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs           # REST endpoints
│   │   ├── websocket.rs        # WebSocket handler
│   │   ├── handlers/           # Request handlers
│   │   │   ├── workflows.rs
│   │   │   ├── executions.rs
│   │   │   ├── agents.rs
│   │   │   └── tools.rs
│   │   └── error.rs            # API error types
│   │
│   ├── ui/
│   │   └── mod.rs              # UI server setup
│   │
│   └── utils/
│       ├── mod.rs
│       ├── interpolation.rs    # Variable interpolation
│       ├── retry.rs            # Retry logic
│       └── secrets.rs          # Secret management
│
├── ui/                         # Svelte frontend
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── src/
│   │   ├── App.svelte
│   │   ├── main.ts
│   │   ├── lib/
│   │   │   ├── api.ts          # API client
│   │   │   └── websocket.ts    # WebSocket client
│   │   ├── components/
│   │   │   ├── WorkflowBuilder.svelte
│   │   │   ├── ExecutionMonitor.svelte
│   │   │   ├── ConversationView.svelte
│   │   │   ├── Settings.svelte
│   │   │   └── ...
│   │   └── routes/
│   │       ├── +page.svelte    # Home
│   │       ├── workflows/
│   │       ├── executions/
│   │       └── settings/
│   └── public/
│
├── src-tauri/                  # Tauri desktop wrapper (optional)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       └── main.rs
│
├── workflows/                  # Example workflows
│   ├── pdf-research.yaml
│   ├── web-scraper.yaml
│   └── file-organizer.yaml
│
├── docs/
│   ├── README.md
│   ├── getting-started.md
│   ├── workflow-syntax.md
│   ├── agent-types.md
│   ├── tool-reference.md
│   └── superpowers/
│       └── specs/
│           └── 2026-04-21-rustagent-design.md
│
├── tests/
│   ├── integration/
│   │   ├── workflow_execution.rs
│   │   ├── agent_coordination.rs
│   │   └── tool_execution.rs
│   └── fixtures/
│       ├── workflows/
│       └── test_data/
│
├── benches/
│   ├── workflow_execution.rs
│   ├── agent_spawn.rs
│   └── variable_interpolation.rs
│
└── examples/
    ├── simple_workflow.rs
    ├── parallel_research.rs
    └── custom_agent.rs
```
## MVP vs v1.0 Scope

### MVP (3-4 weeks) - Core Functionality

**Goal:** Deliver a working, fast, local AI agent orchestrator that demonstrates clear value.

**Features:**

**1. Execution Patterns**
- ✅ Sequential execution (fully implemented)
- ✅ Parallel execution with merge strategies (concat, vote, llm_merge)
- ❌ Supervisor pattern (deferred to v1.0)

**2. Agent Types**
- ✅ ResearchAgent (web research, document analysis)
- ✅ AnalysisAgent (summarization, report generation)
- ❌ CodeAgent (deferred to v1.0)
- ❌ SupervisorAgent (deferred to v1.0)

**3. LLM Providers**
- ✅ Ollama (primary, local-first)
- ✅ OpenAI (fallback option)
- ❌ Anthropic (deferred to v1.0)
- ❌ Groq (deferred to v1.0)

**4. Built-in Tools**
- ✅ FileSystemTool (read, write, list, search)
- ✅ WebScraperTool (fetch, parse HTML)
- ✅ PdfParserTool (extract text, metadata)
- ✅ ShellExecutorTool (execute commands with limits)
- ✅ ApiClientTool (REST API calls)
- ✅ ClipboardTool (read/write clipboard)

**5. Storage**
- ✅ YAML/TOML workflow definitions
- ✅ redb for state and history
- ✅ Simple conversation memory
- ❌ Vector memory (RAG) (deferred to v1.0)

**6. UI**
- ✅ Web UI (Svelte + Axum)
- ✅ Basic workflow builder (visual node editor)
- ✅ Execution monitor (real-time updates)
- ✅ Settings panel
- ❌ Tauri desktop wrapper (deferred to v1.0)

**7. Security**
- ✅ Permission system (allow/deny/prompt)
- ✅ Process isolation for agents
- ✅ Audit logging
- ✅ Secret management (OS keyring)

**8. CLI**
- ✅ `rustforge ui` - Start web UI
- ✅ `rustforge run` - Execute workflow
- ✅ `rustforge list` - List workflows
- ✅ `rustforge logs` - View execution logs
- ✅ `rustforge init` - Initialize project
- ✅ `rustforge config` - Manage configuration

**9. Error Handling**
- ✅ Retry logic with backoff
- ✅ Timeout handling
- ✅ Clear error messages with suggestions
- ✅ Graceful degradation (partial failures in parallel mode)

**10. Documentation**
- ✅ README with quick start
- ✅ Workflow syntax guide
- ✅ Tool reference
- ✅ Example workflows (3-5 examples)

**MVP Success Criteria:**
- ✅ Single binary distribution
- ✅ < 100ms startup time
- ✅ < 800 MB memory for typical workflows
- ✅ 5+ example workflows that work out of the box
- ✅ Clear documentation for getting started

---

### v1.0 (MVP + 2-3 weeks) - Advanced Features

**Goal:** Add advanced patterns and extensibility that differentiate from competitors.

**New Features:**

**1. Supervisor Pattern**
- ✅ Hierarchical agent coordination
- ✅ Dynamic task decomposition
- ✅ Worker agent selection and assignment
- ✅ Result validation and correction loops
- ✅ Adaptive planning

**2. Vector Memory (RAG)**
- ✅ Semantic search over agent memory
- ✅ Shared memory between agents
- ✅ Integration with Ollama embeddings
- ✅ Optional qdrant or lance backend

**3. Script Runner**
- ✅ Python script execution (PyO3 or subprocess)
- ✅ Lua script execution (mlua)
- ✅ Custom merge strategies via scripts
- ✅ Custom tool implementations

**4. Additional LLM Providers**
- ✅ Anthropic (Claude)
- ✅ Groq (fast inference)
- ✅ LocalAI support
- ✅ Custom provider plugin system

**5. CodeAgent**
- ✅ Code analysis and refactoring suggestions
- ✅ Security vulnerability scanning
- ✅ AST-based code understanding
- ✅ Git integration

**6. Tauri Desktop App**
- ✅ Native desktop experience
- ✅ System tray integration
- ✅ Native notifications
- ✅ Auto-start on boot
- ✅ Global keyboard shortcuts

**7. Advanced Workflow Features**
- ✅ Conditional execution (if/else)
- ✅ Loops (for each, while)
- ✅ Sub-workflows (reusable components)
- ✅ Workflow templates marketplace

**8. Monitoring & Observability**
- ✅ Prometheus metrics export
- ✅ Structured logging (JSON format)
- ✅ Performance profiling
- ✅ Execution analytics dashboard

**9. Export Formats**
- ✅ PDF reports (via headless Chrome or typst)
- ✅ Notion integration
- ✅ Slack notifications
- ✅ Email reports (SMTP)

**10. Community Features**
- ✅ Workflow sharing platform
- ✅ Plugin marketplace
- ✅ Community templates
- ✅ Discord bot integration

---

## Testing Strategy

### Unit Tests

**Coverage Target:** 80%+ for core logic

**Key Areas:**
```rust
// src/engine/executor.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_sequential_execution() {
        // Test sequential workflow execution
    }
    
    #[tokio::test]
    async fn test_parallel_execution() {
        // Test parallel workflow execution
    }
    
    #[tokio::test]
    async fn test_variable_interpolation() {
        // Test variable resolution
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        // Test error propagation and recovery
    }
}

// src/llm/registry.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_fallback_provider() {
        // Test LLM fallback mechanism
    }
    
    #[tokio::test]
    async fn test_context_truncation() {
        // Test context window management
    }
}

// src/tools/registry.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_tool_execution() {
        // Test tool execution with permissions
    }
    
    #[tokio::test]
    async fn test_permission_check() {
        // Test permission system
    }
}
```

### Integration Tests

**Test Scenarios:**
```rust
// tests/integration/workflow_execution.rs

#[tokio::test]
async fn test_pdf_research_workflow() {
    // End-to-end test: PDF → extract → summarize → report
    let workflow = load_workflow("workflows/pdf-research.yaml");
    let result = execute_workflow(workflow, inputs).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_parallel_web_scraping() {
    // Test parallel execution with merge
    let workflow = load_workflow("workflows/web-scraper.yaml");
    let result = execute_workflow(workflow, inputs).await.unwrap();
    assert_eq!(result.agents_executed, 3);
}

#[tokio::test]
async fn test_checkpoint_resume() {
    // Test pause and resume functionality
    let execution_id = start_workflow("workflows/long-running.yaml").await;
    pause_execution(execution_id).await.unwrap();
    
    // Simulate restart
    let result = resume_execution(execution_id).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_permission_denial() {
    // Test that permission denial stops execution
    let workflow = load_workflow("workflows/restricted.yaml");
    let result = execute_workflow(workflow, inputs).await;
    assert!(matches!(result, Err(Error::PermissionDenied { .. })));
}
```

### Performance Tests

**Benchmark Suite:**
```rust
// benches/workflow_execution.rs

fn bench_startup_time(c: &mut Criterion) {
    c.bench_function("startup", |b| {
        b.iter(|| {
            // Measure binary startup time
            Command::new("./target/release/rustforge")
                .arg("--version")
                .output()
                .unwrap();
        });
    });
}

fn bench_workflow_parse(c: &mut Criterion) {
    c.bench_function("parse_workflow", |b| {
        b.iter(|| {
            // Measure YAML parsing time
            parse_workflow("workflows/complex.yaml").unwrap();
        });
    });
}

fn bench_parallel_agents(c: &mut Criterion) {
    c.bench_function("parallel_10_agents", |b| {
        b.iter(|| {
            // Measure parallel execution overhead
            execute_parallel_workflow(10).await.unwrap();
        });
    });
}
```

### End-to-End Tests

**Real-World Scenarios:**
1. **PDF Research Pipeline**: 10 PDFs → extract → summarize → markdown report
2. **Web Research**: 20 URLs → scrape → deduplicate → aggregate
3. **File Organization**: 1000 files → categorize → rename → report
4. **Daily News Digest**: Multiple sources → fetch → summarize → email

**Test Environment:**
- Docker container with Ollama
- Mock LLM responses for deterministic tests
- Fixture data (PDFs, HTML pages, etc.)

### CI/CD Integration

**GitHub Actions Workflow:**
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run benchmarks
        run: cargo bench --no-run
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
  
  integration:
    runs-on: ubuntu-latest
    services:
      ollama:
        image: ollama/ollama:latest
    steps:
      - uses: actions/checkout@v3
      - name: Run integration tests
        run: cargo test --test '*' --features integration
```

---

## Deployment and Distribution

### Binary Distribution

**Platforms:**
- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

**Build Process:**
```bash
# Release build with optimizations
cargo build --release

# Strip debug symbols
strip target/release/rustforge

# Create distributable archive
tar -czf rustforge-linux-x86_64.tar.gz -C target/release rustforge
```

**GitHub Releases:**
- Automated via GitHub Actions
- Attach binaries for all platforms
- Include checksums (SHA256)
- Changelog generated from commits

### Installation Methods

**1. Direct Download (Primary)**
```bash
# Linux/macOS
curl -fsSL https://rustforge.dev/install.sh | sh

# Windows (PowerShell)
irm https://rustforge.dev/install.ps1 | iex
```

**2. Package Managers**
```bash
# Homebrew (macOS/Linux)
brew install rustforge

# Cargo
cargo install rustforge

# Scoop (Windows)
scoop install rustforge

# AUR (Arch Linux)
yay -S rustforge
```

**3. Docker**
```bash
docker pull rustforge/rustforge:latest
docker run -p 3000:3000 -v $(pwd):/workspace rustforge/rustforge
```

### Auto-Update Mechanism

**Update Check:**
```rust
pub async fn check_for_updates() -> Result<Option<Version>> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = fetch_latest_version().await?;
    
    if latest > current {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

// Prompt user on startup (non-intrusive)
if let Some(new_version) = check_for_updates().await? {
    println!("📦 New version available: {}", new_version);
    println!("   Run: rustforge update");
}
```

**Self-Update:**
```bash
rustforge update
# Downloads and replaces binary
```

---

## Future Roadmap (Post v1.0)

### v1.1 - Cloud Sync (Optional)
- Encrypted cloud backup of workflows
- Cross-device sync
- Team collaboration features
- Shared workflow library

### v1.2 - Mobile Companion
- iOS/Android app for monitoring
- Push notifications for workflow completion
- Quick workflow triggers
- View execution logs on mobile

### v1.3 - Advanced Integrations
- Zapier/Make.com integration
- GitHub Actions integration
- Slack/Discord bots
- Browser extension for quick captures

### v2.0 - Enterprise Features
- Multi-user support
- Role-based access control
- Centralized management dashboard
- Compliance reporting (SOC2, GDPR)
- SSO integration

---

## Monitoring and Observability (Brief - v1.0)

**Metrics to Track:**
- Workflow execution count
- Success/failure rates
- Average execution time
- Agent spawn time
- Memory usage over time
- LLM API costs (if using paid providers)

**Logging:**
- Structured logging with `tracing`
- Log levels: trace, debug, info, warn, error
- JSON format for machine parsing
- File rotation for long-running instances

**Observability (v1.0):**
- Prometheus metrics export
- Grafana dashboard templates
- OpenTelemetry integration
- Distributed tracing for complex workflows

---

## Conclusion

RustAgent (RustForge) is designed to be the fastest, most efficient local AI agent orchestrator available. By leveraging Rust's performance and safety guarantees, combined with a thoughtful layered architecture, we can deliver:

1. **Performance**: 5-10x faster than Python alternatives
2. **Efficiency**: 5-10x lower memory footprint
3. **Privacy**: Local-first, all data stays on user's machine
4. **Flexibility**: Hybrid approach (visual + code) for all skill levels
5. **Extensibility**: Plugin system for community contributions

**MVP Timeline:** 3-4 weeks
**v1.0 Timeline:** 5-7 weeks total

**Success Metrics:**
- 1,000+ GitHub stars in 3 months
- Active community contributions
- Positive feedback on performance and UX
- Adoption by developers and researchers

**Next Steps:**
1. Set up project structure
2. Implement core engine (orchestration, agents, LLM layer)
3. Build essential tools
4. Create web UI
5. Write documentation and examples
6. Launch MVP on GitHub, Reddit, HackerNews
7. Iterate based on feedback
8. Release v1.0 with advanced features

This design provides a solid foundation for building a production-ready, high-performance AI agent orchestrator that will stand out in the crowded AI tooling space.
