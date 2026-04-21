# Phase 2: LLM & Agent Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement LLM provider abstraction, agent system, and memory management to enable AI-powered workflow execution.

**Architecture:** LLM layer provides unified interface to multiple providers (Ollama, OpenAI). Agent layer defines agent types and behaviors. Memory store handles conversation history.

**Tech Stack:** reqwest (HTTP client), async-trait, serde_json, ollama-rs (optional)

---

## File Structure

**LLM Layer:**
- `src/llm/mod.rs` - Module exports
- `src/llm/traits.rs` - LLMProvider trait
- `src/llm/registry.rs` - LLMRegistry with fallback
- `src/llm/ollama.rs` - OllamaProvider
- `src/llm/openai.rs` - OpenAIProvider
- `src/llm/context.rs` - Context management (token counting, truncation)
- `src/llm/types.rs` - Message, CompletionOptions, ToolCallResponse

**Agent Layer:**
- `src/agents/mod.rs` - Module exports
- `src/agents/traits.rs` - Agent trait
- `src/agents/registry.rs` - AgentRegistry
- `src/agents/types.rs` - AgentDefinition, Task, AgentOutput
- `src/agents/research.rs` - ResearchAgent (stub for Phase 3)
- `src/agents/analysis.rs` - AnalysisAgent (stub for Phase 3)

**Memory Layer:**
- `src/memory/mod.rs` - Module exports
- `src/memory/traits.rs` - MemoryStore trait
- `src/memory/simple.rs` - SimpleMemoryStore (redb-based)

**Tests:**
- `tests/integration/llm_providers.rs` - LLM provider tests
- `tests/integration/agent_execution.rs` - Agent execution tests

---

## Task 1: LLM Types and Traits

**Files:**
- Create: `src/llm/mod.rs`
- Create: `src/llm/types.rs`
- Create: `src/llm/traits.rs`

**Key Steps:**
- [ ] Write tests for Message serialization
- [ ] Define Message struct (role, content, timestamp)
- [ ] Define CompletionOptions (temperature, max_tokens, etc.)
- [ ] Define LLMProvider trait with complete(), stream(), supports_streaming()
- [ ] Add async-trait dependency to Cargo.toml
- [ ] Run tests and commit

**Core Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) 
        -> Result<String>;
    fn supports_streaming(&self) -> bool;
    fn max_context_tokens(&self) -> usize;
}
```

---

## Task 2: Ollama Provider

**Files:**
- Create: `src/llm/ollama.rs`

**Key Steps:**
- [ ] Write test for Ollama API call (mock with mockito)
- [ ] Implement OllamaProvider struct with base_url, model, client
- [ ] Implement LLMProvider trait for OllamaProvider
- [ ] Implement complete() using reqwest POST to /api/generate
- [ ] Handle Ollama response format
- [ ] Add error handling for connection failures
- [ ] Add mockito dependency for tests
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) 
        -> Result<String> {
        let prompt = self.format_messages(&messages);
        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&json!({
                "model": self.model,
                "prompt": prompt,
                "temperature": options.temperature,
                "stream": false,
            }))
            .send()
            .await?;
        
        let body: serde_json::Value = response.json().await?;
        Ok(body["response"].as_str().unwrap_or("").to_string())
    }
    
    fn supports_streaming(&self) -> bool { true }
    fn max_context_tokens(&self) -> usize { 4096 }
}
```

---

## Task 3: OpenAI Provider

**Files:**
- Create: `src/llm/openai.rs`

**Key Steps:**
- [ ] Write test for OpenAI API call (mock)
- [ ] Implement OpenAIProvider with api_key, model, client
- [ ] Implement LLMProvider trait
- [ ] Implement complete() using POST to /v1/chat/completions
- [ ] Handle OpenAI response format
- [ ] Add Authorization header with Bearer token
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) 
        -> Result<String> {
        let openai_messages: Vec<_> = messages.iter()
            .map(|m| json!({
                "role": format!("{:?}", m.role).to_lowercase(),
                "content": m.content,
            }))
            .collect();
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.model,
                "messages": openai_messages,
                "temperature": options.temperature,
            }))
            .send()
            .await?;
        
        let body: serde_json::Value = response.json().await?;
        Ok(body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
    
    fn supports_streaming(&self) -> bool { true }
    fn max_context_tokens(&self) -> usize { 128000 }
}
```

---

## Task 4: LLM Registry with Fallback

**Files:**
- Create: `src/llm/registry.rs`

**Key Steps:**
- [ ] Write test for fallback mechanism (primary fails, fallback succeeds)
- [ ] Implement LLMRegistry with primary and optional fallback providers
- [ ] Implement complete() that tries primary, then fallback on error
- [ ] Implement should_fallback() to check error type
- [ ] Add logging for fallback events
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct LLMRegistry {
    primary: Box<dyn LLMProvider>,
    fallback: Option<Box<dyn LLMProvider>>,
}

impl LLMRegistry {
    pub async fn complete(&self, messages: Vec<Message>, options: CompletionOptions) 
        -> Result<String> {
        match self.primary.complete(messages.clone(), options.clone()).await {
            Ok(response) => Ok(response),
            Err(e) if self.should_fallback(&e) => {
                if let Some(fallback) = &self.fallback {
                    tracing::warn!("Primary LLM failed, using fallback: {}", e);
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
            Error::Internal(_) | 
            Error::Io(_)
        )
    }
}
```

---

## Task 5: Agent Types and Traits

**Files:**
- Create: `src/agents/mod.rs`
- Create: `src/agents/types.rs`
- Create: `src/agents/traits.rs`

**Key Steps:**
- [ ] Write tests for AgentDefinition serialization
- [ ] Define AgentDefinition (id, name, role, system_prompt, llm_provider, tools)
- [ ] Define Task struct (id, description, context)
- [ ] Define AgentOutput (content, metadata)
- [ ] Define Agent trait with execute() method
- [ ] Run tests and commit

**Core Types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub id: String,
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub llm_provider: String,
    pub available_tools: Vec<String>,
    pub max_iterations: usize,
}

pub struct Task {
    pub id: String,
    pub description: String,
}

pub struct AgentOutput {
    pub content: String,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn execute(&mut self, task: Task, context: &ExecutionContext) 
        -> Result<AgentOutput>;
    fn get_definition(&self) -> &AgentDefinition;
}
```

---

## Task 6: Agent Registry

**Files:**
- Create: `src/agents/registry.rs`

**Key Steps:**
- [ ] Write test for registering and retrieving agents
- [ ] Implement AgentRegistry with HashMap<String, AgentDefinition>
- [ ] Implement register(), get(), list() methods
- [ ] Implement load_from_directory() to load agent definitions from YAML files
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, definition: AgentDefinition) {
        self.agents.insert(definition.id.clone(), definition);
    }
    
    pub fn get(&self, id: &str) -> Option<&AgentDefinition> {
        self.agents.get(id)
    }
    
    pub fn list(&self) -> Vec<&AgentDefinition> {
        self.agents.values().collect()
    }
}
```

---

## Task 7: Memory Store

**Files:**
- Create: `src/memory/mod.rs`
- Create: `src/memory/traits.rs`
- Create: `src/memory/simple.rs`

**Key Steps:**
- [ ] Write tests for storing and retrieving messages
- [ ] Define MemoryStore trait (add_message, get_conversation, clear)
- [ ] Implement SimpleMemoryStore using redb
- [ ] Create messages table in redb
- [ ] Implement add_message() to store Message
- [ ] Implement get_conversation() to retrieve last N messages
- [ ] Run tests and commit

**Implementation:**
```rust
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn add_message(&mut self, agent_id: &str, message: Message) -> Result<()>;
    async fn get_conversation(&self, agent_id: &str, limit: usize) 
        -> Result<Vec<Message>>;
    async fn clear(&mut self, agent_id: &str) -> Result<()>;
}

pub struct SimpleMemoryStore {
    db: redb::Database,
}

impl SimpleMemoryStore {
    pub fn new(db_path: &Path) -> Result<Self> {
        let db = redb::Database::create(db_path)?;
        Ok(Self { db })
    }
}

#[async_trait]
impl MemoryStore for SimpleMemoryStore {
    async fn add_message(&mut self, agent_id: &str, message: Message) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MESSAGES_TABLE)?;
            let key = format!("{}:{}", agent_id, message.timestamp.timestamp_nanos());
            let value = bincode::serialize(&message)?;
            table.insert(key.as_bytes(), value.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
    
    async fn get_conversation(&self, agent_id: &str, limit: usize) 
        -> Result<Vec<Message>> {
        // Implementation: scan table, filter by agent_id prefix, take last N
        todo!()
    }
    
    async fn clear(&mut self, agent_id: &str) -> Result<()> {
        // Implementation: delete all keys with agent_id prefix
        todo!()
    }
}
```

---

## Task 8: Basic Agent Implementation (Stub)

**Files:**
- Create: `src/agents/research.rs`
- Create: `src/agents/analysis.rs`

**Key Steps:**
- [ ] Create ResearchAgent struct (stub, returns mock output)
- [ ] Create AnalysisAgent struct (stub, returns mock output)
- [ ] Implement Agent trait for both (minimal implementation)
- [ ] These will be fully implemented in Phase 3 when tools are available
- [ ] Commit

**Stub Implementation:**
```rust
// src/agents/research.rs
pub struct ResearchAgent {
    definition: AgentDefinition,
    llm_registry: Arc<LLMRegistry>,
    memory: Arc<RwLock<dyn MemoryStore>>,
}

#[async_trait]
impl Agent for ResearchAgent {
    async fn execute(&mut self, task: Task, context: &ExecutionContext) 
        -> Result<AgentOutput> {
        // Stub: return mock output
        // Will be fully implemented in Phase 3
        Ok(AgentOutput {
            content: format!("Research result for: {}", task.description),
            metadata: HashMap::new(),
        })
    }
    
    fn get_definition(&self) -> &AgentDefinition {
        &self.definition
    }
}
```

---

## Task 9: Update Executor to Use Real Agents

**Files:**
- Modify: `src/engine/executor.rs`

**Key Steps:**
- [ ] Update SequentialExecutor to create real agent instances
- [ ] Pass LLMRegistry and MemoryStore to agents
- [ ] Call agent.execute() instead of mock
- [ ] Store agent outputs in context
- [ ] Add integration test with real LLM call (use Ollama in CI)
- [ ] Commit

**Updated Executor:**
```rust
impl SequentialExecutor {
    pub async fn execute(
        &self,
        workflow: &WorkflowDefinition,
        context: &mut ExecutionContext,
        llm_registry: Arc<LLMRegistry>,
        agent_registry: &AgentRegistry,
    ) -> Result<serde_json::Value> {
        for agent_config in &workflow.agents {
            // Get agent definition
            let agent_def = agent_registry.get(&agent_config.agent_type)
                .ok_or_else(|| Error::Internal(format!("Agent type not found: {}", agent_config.agent_type)))?;
            
            // Create agent instance
            let mut agent = self.create_agent(agent_def, llm_registry.clone())?;
            
            // Interpolate task
            let interpolator = VariableInterpolator::new(context);
            let task_description = interpolator.interpolate(&agent_config.task)?;
            
            // Execute agent
            let task = Task {
                id: Uuid::new_v4().to_string(),
                description: task_description,
            };
            
            let output = agent.execute(task, context).await?;
            
            // Store output
            context.set_agent_output(&agent_config.id, output.clone());
            
            // Publish event
            self.event_bus.publish(AgentEvent::TaskCompleted {
                agent_id: agent_config.id.clone(),
                output: output.content.clone(),
            })?;
        }
        
        Ok(serde_json::Value::Null)
    }
}
```

---

## Task 10: Integration Test

**Files:**
- Create: `tests/integration/agent_execution.rs`
- Create: `tests/fixtures/workflows/agent-test.yaml`

**Key Steps:**
- [ ] Write end-to-end test: create workflow with agent → execute → verify output
- [ ] Test should use real Ollama (or mock if Ollama not available)
- [ ] Test should verify agent output is stored in context
- [ ] Test should verify memory is updated
- [ ] Run test and commit

---

## Task 11: CLI Integration

**Files:**
- Modify: `src/cli/handlers.rs`
- Modify: `src/main.rs`

**Key Steps:**
- [ ] Initialize LLMRegistry in main.rs from config
- [ ] Initialize AgentRegistry and load built-in agents
- [ ] Pass registries to run command handler
- [ ] Update run handler to use real agents
- [ ] Test manually: `rustforge run workflows/test.yaml`
- [ ] Commit

---

## Task 12: Documentation

**Files:**
- Modify: `README.md`
- Create: `docs/agent-types.md`
- Create: `docs/llm-providers.md`

**Key Steps:**
- [ ] Document LLM provider configuration
- [ ] Document agent types and their capabilities
- [ ] Add example workflows using agents
- [ ] Update README with Phase 2 features
- [ ] Commit

---

## Phase 2 Complete

**Deliverables:**
- ✅ LLM provider abstraction (Ollama, OpenAI)
- ✅ LLM registry with fallback mechanism
- ✅ Agent trait and registry
- ✅ Memory store for conversation history
- ✅ Basic agent implementations (stubs)
- ✅ Updated executor to use real agents
- ✅ Integration tests
- ✅ Documentation

**Next:** Phase 3 - Tool Layer & Security