# Phase 3: Tool Layer & Security Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement built-in tools (filesystem, web scraper, PDF parser, shell, API client, clipboard) and security layer with permissions and process isolation.

**Architecture:** Tool layer provides unified interface for all tools. Permission manager enforces security policies. Process isolation ensures safe execution.

**Tech Stack:** scraper (HTML parsing), reqwest (HTTP), arboard (clipboard), subprocess management

---

## File Structure

**Tool Layer:**
- `src/tools/mod.rs` - Module exports
- `src/tools/traits.rs` - Tool trait
- `src/tools/registry.rs` - ToolRegistry
- `src/tools/types.rs` - ToolParameter, ToolResult
- `src/tools/filesystem.rs` - FileSystemTool
- `src/tools/web_scraper.rs` - WebScraperTool
- `src/tools/pdf_parser.rs` - PdfParserTool
- `src/tools/shell.rs` - ShellExecutorTool
- `src/tools/api_client.rs` - ApiClientTool
- `src/tools/clipboard.rs` - ClipboardTool

**Security Layer:**
- `src/security/mod.rs` - Module exports
- `src/security/permissions.rs` - PermissionManager
- `src/security/isolation.rs` - Process isolation utilities
- `src/security/audit.rs` - Audit logging

**Tests:**
- `tests/integration/tool_execution.rs` - Tool execution tests
- `tests/integration/permissions.rs` - Permission system tests

---

## Task 1: Tool Trait and Types

**Files:**
- Create: `src/tools/mod.rs`
- Create: `src/tools/traits.rs`
- Create: `src/tools/types.rs`

**Key Steps:**
- [ ] Write tests for ToolResult serialization
- [ ] Define Tool trait (name, description, parameters, execute)
- [ ] Define ToolParameter (name, type, required, default)
- [ ] Define ToolResult (success, output, error, metadata)
- [ ] Define ParameterType enum
- [ ] Run tests and commit

**Core Types:**
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    
    async fn execute(&self, params: HashMap<String, Value>) 
        -> Result<ToolResult>;
}

pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default: Option<Value>,
}

pub struct ToolResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}
```

---

## Task 2: FileSystem Tool

**Files:**
- Create: `src/tools/filesystem.rs`

**Key Steps:**
- [ ] Write tests for read_file, write_file, list_directory
- [ ] Implement FileSystemTool struct
- [ ] Implement Tool trait with operations: read, write, list, search, delete, mkdir
- [ ] Add path validation (prevent access outside allowed paths)
- [ ] Handle errors gracefully (file not found, permission denied)
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct FileSystemTool;

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str { "file_system" }
    
    fn description(&self) -> &str {
        "Read, write, list, and manage files and directories"
    }
    
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "operation".to_string(),
                description: "Operation to perform".to_string(),
                param_type: ParameterType::String,
                required: true,
                default: None,
            },
            ToolParameter {
                name: "path".to_string(),
                description: "File or directory path".to_string(),
                param_type: ParameterType::String,
                required: true,
                default: None,
            },
            ToolParameter {
                name: "content".to_string(),
                description: "Content to write (for write operation)".to_string(),
                param_type: ParameterType::String,
                required: false,
                default: None,
            },
        ]
    }
    
    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let operation = params.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing operation".to_string()))?;
        
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing path".to_string()))?;
        
        match operation {
            "read" => self.read_file(path).await,
            "write" => {
                let content = params.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Internal("Missing content".to_string()))?;
                self.write_file(path, content).await
            }
            "list" => self.list_directory(path).await,
            _ => Err(Error::Internal(format!("Unknown operation: {}", operation))),
        }
    }
}

impl FileSystemTool {
    async fn read_file(&self, path: &str) -> Result<ToolResult> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(ToolResult {
            success: true,
            output: json!({ "content": content }),
            error: None,
            metadata: HashMap::new(),
        })
    }
    
    async fn write_file(&self, path: &str, content: &str) -> Result<ToolResult> {
        tokio::fs::write(path, content).await?;
        Ok(ToolResult {
            success: true,
            output: json!({ "bytes_written": content.len() }),
            error: None,
            metadata: HashMap::new(),
        })
    }
    
    async fn list_directory(&self, path: &str) -> Result<ToolResult> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        
        Ok(ToolResult {
            success: true,
            output: json!({ "entries": entries }),
            error: None,
            metadata: HashMap::new(),
        })
    }
}
```

---

## Task 3: WebScraper Tool

**Files:**
- Create: `src/tools/web_scraper.rs`

**Key Steps:**
- [ ] Add scraper and reqwest dependencies to Cargo.toml
- [ ] Write tests for fetching URL and parsing HTML
- [ ] Implement WebScraperTool
- [ ] Implement fetch_url() using reqwest
- [ ] Implement extract_text() using scraper crate
- [ ] Add timeout and retry logic
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct WebScraperTool {
    client: reqwest::Client,
}

impl WebScraperTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Tool for WebScraperTool {
    fn name(&self) -> &str { "web_scraper" }
    
    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let url = params.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing url".to_string()))?;
        
        let response = self.client.get(url).send().await?;
        let html = response.text().await?;
        
        // Parse HTML and extract text
        let document = scraper::Html::parse_document(&html);
        let text_selector = scraper::Selector::parse("body").unwrap();
        let text = document.select(&text_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        
        Ok(ToolResult {
            success: true,
            output: json!({
                "html": html,
                "text": text,
                "url": url,
            }),
            error: None,
            metadata: HashMap::new(),
        })
    }
}
```

---

## Task 4: Shell Executor Tool

**Files:**
- Create: `src/tools/shell.rs`

**Key Steps:**
- [ ] Write tests for executing safe commands
- [ ] Implement ShellExecutorTool
- [ ] Use tokio::process::Command for subprocess execution
- [ ] Add timeout enforcement
- [ ] Add command whitelist/blacklist support
- [ ] Capture stdout, stderr, exit code
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct ShellExecutorTool {
    timeout: Duration,
}

#[async_trait]
impl Tool for ShellExecutorTool {
    fn name(&self) -> &str { "shell_executor" }
    
    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let command = params.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing command".to_string()))?;
        
        let args: Vec<String> = params.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();
        
        let output = tokio::time::timeout(
            self.timeout,
            tokio::process::Command::new(command)
                .args(&args)
                .output()
        ).await??;
        
        Ok(ToolResult {
            success: output.status.success(),
            output: json!({
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
                "exit_code": output.status.code(),
            }),
            error: if !output.status.success() {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            } else {
                None
            },
            metadata: HashMap::new(),
        })
    }
}
```

---

## Task 5: PDF Parser, API Client, Clipboard Tools

**Files:**
- Create: `src/tools/pdf_parser.rs`
- Create: `src/tools/api_client.rs`
- Create: `src/tools/clipboard.rs`

**Key Steps:**
- [ ] Implement PdfParserTool (use pdf-extract or lopdf crate)
- [ ] Implement ApiClientTool (GET, POST, PUT, DELETE methods)
- [ ] Implement ClipboardTool (use arboard crate)
- [ ] Add tests for each tool
- [ ] Add dependencies to Cargo.toml
- [ ] Run tests and commit

**Brief implementations similar to above tools**

---

## Task 6: Tool Registry

**Files:**
- Create: `src/tools/registry.rs`

**Key Steps:**
- [ ] Write tests for registering and executing tools
- [ ] Implement ToolRegistry with HashMap<String, Box<dyn Tool>>
- [ ] Implement register(), get(), list() methods
- [ ] Implement execute() that validates params and calls tool
- [ ] Add tool execution logging
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register built-in tools
        registry.register(Box::new(FileSystemTool));
        registry.register(Box::new(WebScraperTool::new()));
        registry.register(Box::new(ShellExecutorTool { 
            timeout: Duration::from_secs(300) 
        }));
        
        registry
    }
    
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| b.as_ref())
    }
    
    pub async fn execute(
        &self,
        tool_name: &str,
        params: HashMap<String, Value>,
    ) -> Result<ToolResult> {
        let tool = self.get(tool_name)
            .ok_or_else(|| Error::Internal(format!("Tool not found: {}", tool_name)))?;
        
        tool.execute(params).await
    }
}
```

---

## Task 7: Permission Manager

**Files:**
- Create: `src/security/mod.rs`
- Create: `src/security/permissions.rs`

**Key Steps:**
- [ ] Write tests for permission checking (allow, deny, prompt)
- [ ] Define PermissionRule struct
- [ ] Implement PermissionManager with policy rules
- [ ] Implement check_permission() that matches rules
- [ ] Implement prompt_user() for interactive permission requests
- [ ] Store session permissions in memory
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct PermissionManager {
    policy: PermissionPolicy,
    session_permissions: Arc<RwLock<HashMap<String, PolicyAction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub tool: String,
    pub operations: Vec<String>,
    pub scope: Option<Scope>,
    pub action: PolicyAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Scope {
    FilePaths(Vec<PathBuf>),
    Domains(Vec<String>),
    Commands(Vec<String>),
}

impl PermissionManager {
    pub async fn check_permission(
        &self,
        tool: &str,
        operation: &str,
        scope_value: &str,
    ) -> Result<bool> {
        // Check rules in order
        for rule in &self.policy.rules {
            if rule.matches(tool, operation, scope_value) {
                return match rule.action {
                    PolicyAction::Allow => Ok(true),
                    PolicyAction::Deny => Ok(false),
                    PolicyAction::Prompt => {
                        self.prompt_user(tool, operation, scope_value).await
                    }
                };
            }
        }
        
        // No rule matched, use default policy
        match self.policy.default {
            PolicyAction::Allow => Ok(true),
            PolicyAction::Deny => Ok(false),
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
    ) -> Result<bool> {
        println!("\n⚠️  Permission Required\n");
        println!("Tool: {}", tool);
        println!("Operation: {}", operation);
        println!("Target: {}", scope_value);
        println!("\n[A]llow once  [T]his session  [F]orever  [D]eny\n");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim().to_lowercase().as_str() {
            "a" | "allow" => Ok(true),
            "t" | "session" => {
                let key = format!("{}:{}:{}", tool, operation, scope_value);
                self.session_permissions.write().await.insert(key, PolicyAction::Allow);
                Ok(true)
            }
            "f" | "forever" => {
                // TODO: Save to config file
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl PermissionRule {
    fn matches(&self, tool: &str, operation: &str, scope_value: &str) -> bool {
        if self.tool != tool {
            return false;
        }
        
        if !self.operations.is_empty() && !self.operations.contains(&operation.to_string()) {
            return false;
        }
        
        if let Some(scope) = &self.scope {
            match scope {
                Scope::FilePaths(paths) => {
                    let path = PathBuf::from(scope_value);
                    paths.iter().any(|p| path.starts_with(p))
                }
                Scope::Domains(domains) => {
                    domains.iter().any(|d| scope_value.contains(d))
                }
                Scope::Commands(commands) => {
                    commands.contains(&scope_value.to_string())
                }
            }
        } else {
            true
        }
    }
}
```

---

## Task 8: Audit Logging

**Files:**
- Create: `src/security/audit.rs`

**Key Steps:**
- [ ] Define AuditLog struct (timestamp, execution_id, agent_id, action, result)
- [ ] Define AuditAction enum (ToolExecuted, FileAccessed, NetworkRequest, etc.)
- [ ] Implement AuditLogger that writes to redb
- [ ] Implement log() method
- [ ] Implement query methods (by execution_id, by agent_id, by time range)
- [ ] Run tests and commit

**Implementation:**
```rust
pub struct AuditLogger {
    db: redb::Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub execution_id: Uuid,
    pub agent_id: String,
    pub action: AuditAction,
    pub result: AuditResult,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failed { error: String },
    Denied { reason: String },
}

impl AuditLogger {
    pub async fn log(&self, log: AuditLog) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(AUDIT_TABLE)?;
            let key = log.id.as_bytes();
            let value = bincode::serialize(&log)?;
            table.insert(key, value.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
```

---

## Task 9: Update Agents to Use Tools

**Files:**
- Modify: `src/agents/research.rs`
- Modify: `src/agents/analysis.rs`

**Key Steps:**
- [ ] Update ResearchAgent to accept ToolRegistry
- [ ] Implement tool calling logic in agent execution
- [ ] Parse LLM response for tool calls
- [ ] Execute tools via registry
- [ ] Add tool results to conversation context
- [ ] Update AnalysisAgent similarly
- [ ] Add integration test with tool usage
- [ ] Commit

**Updated Agent:**
```rust
impl ResearchAgent {
    async fn execute_with_tools(
        &mut self,
        task: Task,
        context: &ExecutionContext,
        tool_registry: &ToolRegistry,
        permission_manager: &PermissionManager,
    ) -> Result<AgentOutput> {
        let mut messages = vec![
            Message {
                role: MessageRole::System,
                content: self.definition.system_prompt.clone(),
                timestamp: Utc::now(),
            },
            Message {
                role: MessageRole::User,
                content: task.description.clone(),
                timestamp: Utc::now(),
            },
        ];
        
        for _ in 0..self.definition.max_iterations {
            // Get LLM response
            let response = self.llm_registry.complete(
                messages.clone(),
                CompletionOptions::default(),
            ).await?;
            
            // Check if response contains tool call
            if let Some(tool_call) = self.parse_tool_call(&response) {
                // Check permission
                let allowed = permission_manager.check_permission(
                    &tool_call.tool_name,
                    &tool_call.operation,
                    &tool_call.scope_value,
                ).await?;
                
                if !allowed {
                    return Err(Error::Internal("Permission denied".to_string()));
                }
                
                // Execute tool
                let tool_result = tool_registry.execute(
                    &tool_call.tool_name,
                    tool_call.parameters,
                ).await?;
                
                // Add tool result to conversation
                messages.push(Message {
                    role: MessageRole::Assistant,
                    content: response.clone(),
                    timestamp: Utc::now(),
                });
                messages.push(Message {
                    role: MessageRole::User,
                    content: format!("Tool result: {:?}", tool_result),
                    timestamp: Utc::now(),
                });
            } else {
                // No tool call, return response
                return Ok(AgentOutput {
                    content: response,
                    metadata: HashMap::new(),
                });
            }
        }
        
        Err(Error::Internal("Max iterations reached".to_string()))
    }
}
```

---

## Task 10: Integration Tests

**Files:**
- Create: `tests/integration/tool_execution.rs`
- Create: `tests/integration/permissions.rs`

**Key Steps:**
- [ ] Test each tool individually
- [ ] Test permission system (allow, deny, prompt)
- [ ] Test agent using tools end-to-end
- [ ] Test audit logging
- [ ] Run tests and commit

---

## Task 11: Documentation

**Files:**
- Modify: `README.md`
- Create: `docs/tool-reference.md`
- Create: `docs/security.md`

**Key Steps:**
- [ ] Document all built-in tools with examples
- [ ] Document permission system configuration
- [ ] Document audit logging
- [ ] Add security best practices
- [ ] Update README with Phase 3 features
- [ ] Commit

---

## Phase 3 Complete

**Deliverables:**
- ✅ 6 built-in tools (filesystem, web scraper, PDF, shell, API, clipboard)
- ✅ Tool registry and execution framework
- ✅ Permission system with allow/deny/prompt
- ✅ Audit logging for compliance
- ✅ Process isolation utilities
- ✅ Agents integrated with tools
- ✅ Integration tests
- ✅ Documentation

**Next:** Phase 4 - API & Execution Patterns