# Phase 1: Core Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the foundational infrastructure for RustForge - config management, CLI, storage, and basic workflow execution engine.

**Architecture:** Layered architecture with Config/CLI at the bottom, Storage layer for persistence (YAML configs + redb state DB), and Orchestration Engine for workflow parsing and sequential execution. Event bus enables real-time updates.

**Tech Stack:** Rust, tokio (async), clap (CLI), serde/serde_yaml/toml (serialization), redb (embedded DB), thiserror (errors)

---

## File Structure

**Core Infrastructure:**
- `Cargo.toml` - Project dependencies and metadata
- `src/main.rs` - Entry point, CLI initialization
- `src/lib.rs` - Library exports

**Config Layer:**
- `src/config/mod.rs` - Module exports
- `src/config/types.rs` - GlobalConfig, LLMConfig, etc.
- `src/config/loader.rs` - Config loading logic (env, files, defaults)

**CLI Layer:**
- `src/cli/mod.rs` - Module exports
- `src/cli/commands.rs` - Command definitions (clap)
- `src/cli/handlers.rs` - Command handlers
- `src/cli/init.rs` - Project initialization

**Storage Layer:**
- `src/storage/mod.rs` - Module exports
- `src/storage/state.rs` - StateStore (redb)
- `src/storage/workflow.rs` - Workflow definition loading

**Engine Layer:**
- `src/engine/mod.rs` - Module exports
- `src/engine/types.rs` - Core types (WorkflowDefinition, ExecutionContext, etc.)
- `src/engine/parser.rs` - YAML workflow parser
- `src/engine/validator.rs` - DAG validation
- `src/engine/executor.rs` - Sequential workflow executor
- `src/engine/events.rs` - Event bus
- `src/engine/interpolation.rs` - Variable interpolation
- `src/engine/checkpoint.rs` - Checkpoint/resume logic

**Error Handling:**
- `src/error.rs` - Error types and Result alias
- `src/retry.rs` - Retry logic with backoff

**Tests:**
- `tests/integration/workflow_execution.rs` - End-to-end workflow tests
- `tests/fixtures/workflows/` - Test workflow YAML files

---

## Task 1: Project Setup

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `.gitignore`
- Create: `README.md`

- [ ] **Step 1: Create Cargo.toml with dependencies**

```toml
[package]
name = "rustforge"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Local AI Agent Orchestrator"
license = "MIT"
repository = "https://github.com/apus3404-oss/RustForge"

[dependencies]
# Async runtime
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# CLI
clap = { version = "4.5", features = ["derive"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"

# Storage
redb = "2.1"
bincode = "1.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
regex = "1.10"

[dev-dependencies]
tempfile = "3.12"
mockall = "0.13"

[[bin]]
name = "rustforge"
path = "src/main.rs"

[lib]
name = "rustforge"
path = "src/lib.rs"
```

- [ ] **Step 2: Create minimal main.rs**

```rust
// src/main.rs
fn main() {
    println!("RustForge v{}", env!("CARGO_PKG_VERSION"));
}
```

- [ ] **Step 3: Create lib.rs with module declarations**

```rust
// src/lib.rs
pub mod config;
pub mod cli;
pub mod storage;
pub mod engine;
pub mod error;
pub mod retry;
```

- [ ] **Step 4: Create .gitignore**

```
/target
/Cargo.lock
**/*.rs.bk
*.pdb
.DS_Store
.idea/
.vscode/
*.swp
*.swo
*~

# RustForge specific
.rustforge/
!.rustforge/config.toml.example
reports/
*.log
```

- [ ] **Step 5: Create basic README.md**

```markdown
# RustForge

Local AI Agent Orchestrator - Ultra-fast, privacy-focused agent workflow execution.

## Status

🚧 **Phase 1: Core Foundation** - In Development

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/rustforge --version
```

## License

MIT
```

- [ ] **Step 6: Verify build**

Run: `cargo build`
Expected: Successful compilation

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml src/ .gitignore README.md
git commit -m "chore: initialize project structure

- Add Cargo.toml with core dependencies
- Create main.rs and lib.rs stubs
- Add .gitignore and README

Co-Authored-By: Claude Sonnet 4 <noreply@anthropic.com>"
```

---

## Task 2: Error Types

**Files:**
- Create: `src/error.rs`

- [ ] **Step 1: Write error type tests**

```rust
// src/error.rs
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
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test error::tests`
Expected: FAIL - types not defined

- [ ] **Step 3: Implement error types**

```rust
// src/error.rs
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
    Storage(#[from] redb::Error),

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
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test error::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/error.rs
git commit -m "feat(error): add core error types

- Define Error enum with thiserror
- Add workflow, execution, storage error variants
- Implement From traits for common error types
- Add unit tests for error display

Co-Authored-By: Claude Sonnet 4 <noreply@anthropic.com>"
```

---

## Task 3: Config Types

**Files:**
- Create: `src/config/mod.rs`
- Create: `src/config/types.rs`

- [ ] **Step 1: Write config type tests**

```rust
// src/config/types.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.execution.max_parallel_agents, 10);
        assert_eq!(config.execution.default_timeout.as_secs(), 300);
    }

    #[test]
    fn test_config_serialization() {
        let config = GlobalConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: GlobalConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.execution.max_parallel_agents, deserialized.execution.max_parallel_agents);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test config::types::tests`
Expected: FAIL - types not defined

- [ ] **Step 3: Create config module exports**

```rust
// src/config/mod.rs
pub mod types;
pub mod loader;

pub use types::*;
pub use loader::ConfigLoader;
```

- [ ] **Step 4: Implement config types**

```rust
// src/config/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub llm: LLMConfig,
    pub execution: ExecutionConfig,
    pub permissions: PermissionConfig,
    pub ui: UIConfig,
    pub logging: LoggingConfig,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            execution: ExecutionConfig::default(),
            permissions: PermissionConfig::default(),
            ui: UIConfig::default(),
            logging: LoggingConfig::default(),
            data_dir: default_data_dir(),
        }
    }
}

fn default_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".rustforge")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    #[serde(default = "default_llm_provider")]
    pub default_provider: String,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default = "default_true")]
    pub fallback_enabled: bool,
}

impl Default for LLMConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                api_key_env: None,
                base_url: Some("http://localhost:11434".to_string()),
                default_model: "llama3".to_string(),
                timeout_secs: 300,
            },
        );
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                base_url: None,
                default_model: "gpt-4o-mini".to_string(),
                timeout_secs: 60,
            },
        );

        Self {
            default_provider: default_llm_provider(),
            providers,
            fallback_enabled: true,
        }
    }
}

fn default_llm_provider() -> String {
    "ollama:llama3".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default = "default_max_parallel")]
    pub max_parallel_agents: usize,
    #[serde(default = "default_execution_timeout", with = "duration_secs")]
    pub default_timeout: Duration,
    #[serde(default = "default_true")]
    pub enable_checkpoints: bool,
    #[serde(default = "default_checkpoint_interval", with = "duration_secs")]
    pub checkpoint_interval: Duration,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_parallel_agents: default_max_parallel(),
            default_timeout: default_execution_timeout(),
            enable_checkpoints: true,
            checkpoint_interval: default_checkpoint_interval(),
        }
    }
}

fn default_max_parallel() -> usize {
    10
}

fn default_execution_timeout() -> Duration {
    Duration::from_secs(300)
}

fn default_checkpoint_interval() -> Duration {
    Duration::from_secs(60)
}

mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    #[serde(default = "default_policy")]
    pub default_policy: PolicyAction,
    #[serde(default = "default_true")]
    pub audit_log_enabled: bool,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            default_policy: default_policy(),
            audit_log_enabled: true,
        }
    }
}

fn default_policy() -> PolicyAction {
    PolicyAction::Prompt
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    Allow,
    Deny,
    Prompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub auto_open_browser: bool,
    #[serde(default)]
    pub enable_cors: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            auto_open_browser: true,
            enable_cors: false,
        }
    }
}

fn default_port() -> u16 {
    3000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: LogFormat::Pretty,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert_eq!(config.execution.max_parallel_agents, 10);
        assert_eq!(config.execution.default_timeout.as_secs(), 300);
    }

    #[test]
    fn test_config_serialization() {
        let config = GlobalConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: GlobalConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            config.execution.max_parallel_agents,
            deserialized.execution.max_parallel_agents
        );
    }
}
```

- [ ] **Step 5: Add dirs dependency to Cargo.toml**

Add to `[dependencies]` section:
```toml
dirs = "5.0"
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test config::types::tests`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/config/ Cargo.toml
git commit -m "feat(config): add configuration types

- Define GlobalConfig with nested config structs
- Add LLM, Execution, Permission, UI, Logging configs
- Implement Default trait with sensible defaults
- Add serde serialization support
- Add unit tests for config creation and serialization

Co-Authored-By: Claude Sonnet 4 <noreply@anthropic.com>"
```

---

## Task 4: Config Loader

**Files:**
- Create: `src/config/loader.rs`
- Create: `.rustforge/config.toml.example`

- [ ] **Step 1: Write config loader tests**

```rust
// src/config/loader.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_load_default_config() {
        let config = ConfigLoader::load_default();
        assert_eq!(config.execution.max_parallel_agents, 10);
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
[execution]
max_parallel_agents = 20
default_timeout = 600
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = ConfigLoader::load_from_file(&config_path).unwrap();
        assert_eq!(config.execution.max_parallel_agents, 20);
        assert_eq!(config.execution.default_timeout.as_secs(), 600);
    }

    #[test]
    fn test_merge_configs() {
        let mut base = GlobalConfig::default();
        let mut override_config = GlobalConfig::default();
        override_config.execution.max_parallel_agents = 20;

        base.merge(override_config);
        assert_eq!(base.execution.max_parallel_agents, 20);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test config::loader::tests`
Expected: FAIL - ConfigLoader not defined

- [ ] **Step 3: Implement config loader**

```rust
// src/config/loader.rs
use crate::config::types::GlobalConfig;
use crate::error::{Error, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration with priority:
    /// 1. Environment variables (RUSTFORGE_*)
    /// 2. Project config (.rustforge/config.toml)
    /// 3. User config (~/.rustforge/config.toml)
    /// 4. Default values
    pub fn load() -> Result<GlobalConfig> {
        let mut config = Self::load_default();

        // Load user-global config
        if let Some(user_config_path) = Self::user_config_path() {
            if user_config_path.exists() {
                let user_config = Self::load_from_file(&user_config_path)?;
                config.merge(user_config);
            }
        }

        // Load project config
        if let Some(project_config_path) = Self::project_config_path() {
            if project_config_path.exists() {
                let project_config = Self::load_from_file(&project_config_path)?;
                config.merge(project_config);
            }
        }

        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config);

        Ok(config)
    }

    pub fn load_default() -> GlobalConfig {
        GlobalConfig::default()
    }

    pub fn load_from_file(path: &Path) -> Result<GlobalConfig> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)))
    }

    pub fn save_to_file(config: &GlobalConfig, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }

    fn user_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".rustforge").join("config.toml"))
    }

    fn project_config_path() -> Option<PathBuf> {
        env::current_dir()
            .ok()
            .map(|cwd| cwd.join(".rustforge").join("config.toml"))
    }

    fn apply_env_overrides(config: &mut GlobalConfig) {
        // RUSTFORGE_DEFAULT_LLM
        if let Ok(llm) = env::var("RUSTFORGE_DEFAULT_LLM") {
            config.llm.default_provider = llm;
        }

        // RUSTFORGE_MAX_PARALLEL_AGENTS
        if let Ok(max_parallel) = env::var("RUSTFORGE_MAX_PARALLEL_AGENTS") {
            if let Ok(value) = max_parallel.parse() {
                config.execution.max_parallel_agents = value;
            }
        }

        // RUSTFORGE_LOG_LEVEL
        if let Ok(log_level) = env::var("RUSTFORGE_LOG_LEVEL") {
            config.logging.level = log_level;
        }
    }
}

impl GlobalConfig {
    pub fn merge(&mut self, other: GlobalConfig) {
        // Merge LLM config
        self.llm.default_provider = other.llm.default_provider;
        self.llm.fallback_enabled = other.llm.fallback_enabled;
        for (key, value) in other.llm.providers {
            self.llm.providers.insert(key, value);
        }

        // Merge execution config
        self.execution = other.execution;

        // Merge permissions config
        self.permissions = other.permissions;

        // Merge UI config
        self.ui = other.ui;

        // Merge logging config
        self.logging = other.logging;

        // Merge data_dir
        self.data_dir = other.data_dir;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_load_default_config() {
        let config = ConfigLoader::load_default();
        assert_eq!(config.execution.max_parallel_agents, 10);
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
[execution]
max_parallel_agents = 20
default_timeout = 600
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = ConfigLoader::load_from_file(&config_path).unwrap();
        assert_eq!(config.execution.max_parallel_agents, 20);
        assert_eq!(config.execution.default_timeout.as_secs(), 600);
    }

    #[test]
    fn test_merge_configs() {
        let mut base = GlobalConfig::default();
        let mut override_config = GlobalConfig::default();
        override_config.execution.max_parallel_agents = 20;

        base.merge(override_config);
        assert_eq!(base.execution.max_parallel_agents, 20);
    }
}
```

- [ ] **Step 4: Create example config file**

```toml
# .rustforge/config.toml.example
[llm]
default_provider = "ollama:llama3"
fallback_enabled = true

[llm.providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama3"
timeout_secs = 300

[llm.providers.openai]
api_key_env = "OPENAI_API_KEY"
default_model = "gpt-4o-mini"
timeout_secs = 60

[execution]
max_parallel_agents = 10
default_timeout = 300
enable_checkpoints = true
checkpoint_interval = 60

[permissions]
default_policy = "prompt"
audit_log_enabled = true

[ui]
port = 3000
auto_open_browser = true
enable_cors = false

[logging]
level = "info"
format = "pretty"
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test config::loader::tests`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/config/loader.rs .rustforge/config.toml.example
git commit -m "feat(config): add config loader with priority system

- Implement ConfigLoader with env/file/default priority
- Support user-global and project-specific configs
- Add environment variable overrides
- Implement config merge logic
- Add example config file
- Add unit tests for loading and merging

Co-Authored-By: Claude Sonnet 4 <noreply@anthropic.com>"
```

---

## Task 5: Storage Layer - State Store

**Files:**
- Create: `src/storage/mod.rs`
- Create: `src/storage/state.rs`

**Key Steps:**
- [ ] Write tests for StateStore (save/get execution, save/get checkpoint)
- [ ] Implement StateStore with redb
- [ ] Define database schema (executions, checkpoints, messages tables)
- [ ] Implement CRUD operations for WorkflowExecution
- [ ] Implement checkpoint save/load
- [ ] Run tests and commit

**Core Implementation:**
```rust
pub struct StateStore {
    db: redb::Database,
}

impl StateStore {
    pub async fn save_execution(&self, execution: &WorkflowExecution) -> Result<()>;
    pub async fn get_execution(&self, id: Uuid) -> Result<Option<WorkflowExecution>>;
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()>;
    pub async fn get_latest_checkpoint(&self, execution_id: Uuid) -> Result<Option<Checkpoint>>;
}
```

---

## Task 6: Engine Types

**Files:**
- Create: `src/engine/mod.rs`
- Create: `src/engine/types.rs`

**Key Steps:**
- [ ] Write tests for WorkflowDefinition deserialization
- [ ] Define WorkflowDefinition, AgentConfig, ExecutionMode structs
- [ ] Define ExecutionContext with context_store HashMap
- [ ] Define ExecutionStatus enum
- [ ] Implement serde traits
- [ ] Run tests and commit

**Core Types:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub mode: ExecutionMode,
    pub agents: Vec<AgentConfig>,
    pub inputs: Option<Vec<InputDefinition>>,
}

pub struct ExecutionContext {
    pub workflow_id: String,
    pub execution_id: Uuid,
    pub context_store: HashMap<String, serde_json::Value>,
}
```

---

## Task 7: Workflow Parser

**Files:**
- Create: `src/engine/parser.rs`
- Create: `tests/fixtures/workflows/simple.yaml`

**Key Steps:**
- [ ] Write test for parsing valid YAML workflow
- [ ] Write test for parsing invalid YAML (should fail gracefully)
- [ ] Implement WorkflowParser::parse_file()
- [ ] Implement WorkflowParser::parse_str()
- [ ] Add validation (required fields, valid agent IDs)
- [ ] Run tests and commit

**Example Test Workflow:**
```yaml
name: "Simple Test"
mode: sequential
agents:
  - id: agent1
    type: TestAgent
    task: "Do something"
```

---

## Task 8: Variable Interpolation

**Files:**
- Create: `src/engine/interpolation.rs`

**Key Steps:**
- [ ] Write tests for interpolating {input.field}, {agent_id.output}
- [ ] Write test for missing variable (should suggest similar names)
- [ ] Implement VariableInterpolator with regex-based replacement
- [ ] Implement resolve_variable() with context lookup
- [ ] Implement Levenshtein distance for suggestions
- [ ] Run tests and commit

**Core Logic:**
```rust
pub struct VariableInterpolator<'a> {
    context: &'a ExecutionContext,
}

impl<'a> VariableInterpolator<'a> {
    pub fn interpolate(&self, template: &str) -> Result<String>;
    fn resolve_variable(&self, path: &str) -> Result<String>;
}
```

---

## Task 9: Event Bus

**Files:**
- Create: `src/engine/events.rs`

**Key Steps:**
- [ ] Write test for publishing and subscribing to events
- [ ] Define AgentEvent enum (TaskStarted, TaskCompleted, etc.)
- [ ] Implement EventBus with tokio::sync::broadcast
- [ ] Implement publish() and subscribe() methods
- [ ] Run tests and commit

**Core Implementation:**
```rust
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    TaskStarted { agent_id: String, task: String },
    TaskCompleted { agent_id: String, output: String },
    TaskFailed { agent_id: String, error: String },
}
```

---

## Task 10: Sequential Executor (Stub)

**Files:**
- Create: `src/engine/executor.rs`

**Key Steps:**
- [ ] Write test for sequential execution (with mock agents)
- [ ] Implement SequentialExecutor::execute()
- [ ] Loop through agents, interpolate task variables
- [ ] Store agent outputs in context
- [ ] Publish events via event bus
- [ ] Run tests and commit

**Note:** This is a stub that will be completed in Phase 2 when we have real agents.

```rust
pub struct SequentialExecutor {
    event_bus: Arc<EventBus>,
}

impl SequentialExecutor {
    pub async fn execute(
        &self,
        workflow: &WorkflowDefinition,
        context: &mut ExecutionContext,
    ) -> Result<serde_json::Value>;
}
```

---

## Task 11: CLI Commands

**Files:**
- Create: `src/cli/mod.rs`
- Create: `src/cli/commands.rs`
- Create: `src/cli/handlers.rs`
- Create: `src/cli/init.rs`
- Modify: `src/main.rs`

**Key Steps:**
- [ ] Define CLI commands with clap (init, run, list, config)
- [ ] Implement init command (create .rustforge/, workflows/, example workflow)
- [ ] Implement config show/set/get commands
- [ ] Implement run command (parse workflow, execute, print results)
- [ ] Wire up main.rs to parse CLI and dispatch to handlers
- [ ] Test each command manually
- [ ] Commit

**CLI Structure:**
```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

enum Commands {
    Init { path: Option<PathBuf> },
    Run { workflow: PathBuf },
    Config { #[command(subcommand)] command: ConfigCommands },
}
```

---

## Task 12: Integration Test

**Files:**
- Create: `tests/integration/workflow_execution.rs`
- Create: `tests/fixtures/workflows/test-sequential.yaml`

**Key Steps:**
- [ ] Write end-to-end test: init project → create workflow → run workflow
- [ ] Test should verify execution completes successfully
- [ ] Test should verify state is saved to database
- [ ] Test should verify events are published
- [ ] Run test and commit

---

## Task 13: Documentation

**Files:**
- Modify: `README.md`
- Create: `docs/getting-started.md`

**Key Steps:**
- [ ] Update README with Phase 1 features and usage examples
- [ ] Add installation instructions
- [ ] Add quick start guide
- [ ] Document CLI commands
- [ ] Add example workflow
- [ ] Commit

---

## Phase 1 Complete

**Deliverables:**
- ✅ Config management (YAML/TOML + env vars)
- ✅ CLI with init, run, config commands
- ✅ Storage layer (redb state database)
- ✅ Workflow parser (YAML → WorkflowDefinition)
- ✅ Variable interpolation engine
- ✅ Event bus for real-time updates
- ✅ Sequential executor (stub, ready for Phase 2)
- ✅ Integration tests
- ✅ Documentation

**Next:** Phase 2 - LLM & Agent Layer