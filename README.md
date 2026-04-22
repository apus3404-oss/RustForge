# RustForge

**Local AI Agent Orchestrator** - Ultra-fast, privacy-focused agent workflow execution built in Rust.

## Overview

RustForge is a high-performance orchestration engine for AI agent workflows. Define multi-agent workflows in YAML, execute them locally with full control over your data, and leverage powerful features like variable interpolation, checkpointing, and real-time event streaming.

**Current Status:** Phase 4 - API & Execution Patterns ✅

Phase 4 adds REST API with WebSocket support, parallel execution, merge strategies, timeout/cancellation, and comprehensive integration tests.

## Features

### Core Foundation (Phase 1)
- **YAML-based Workflow Definitions** - Simple, declarative workflow syntax
- **Sequential Execution Engine** - Reliable step-by-step agent orchestration
- **Variable Interpolation** - Dynamic context passing between agents with `{agent_id.output}` syntax
- **State Persistence** - Embedded redb database for execution history and checkpoints
- **Event Bus** - Real-time workflow execution events
- **Flexible Configuration** - Multi-layer config system (defaults → user → project → env vars)
- **CLI Interface** - Intuitive commands for workflow management

### LLM + Agent Layer (Phase 2)
- **Ollama Integration** - Local LLM support for privacy-focused execution
- **OpenAI Fallback** - Automatic cloud fallback when local LLM unavailable
- **Agent System** - BaseAgent with LLM provider integration
- **Memory Store** - Conversation history persistence with redb
- **Real Agent Execution** - Workflows now execute with actual LLM calls
- **Thread-Safe Registries** - Concurrent agent and LLM provider management

### Tool Layer & Security (Phase 3)
- **6 Built-in Tools** - FileSystem, WebScraper, PDF Parser, Shell Executor, API Client, Clipboard
- **Tool Registry** - Thread-safe tool management and execution
- **Permission System** - Allow/Deny/Prompt policies for tool execution
- **Process Isolation** - Sandboxed subprocess execution with timeout enforcement
- **Audit Logging** - Security event tracking for compliance
- **Path & Command Validation** - Prevent unauthorized access and dangerous operations

### API & Execution Patterns (Phase 4) 🆕
- **REST API** - Full HTTP API with Axum for workflow and execution management
- **WebSocket Support** - Real-time execution event streaming with bidirectional control
- **Parallel Execution** - Concurrent agent execution with tokio::spawn
- **Merge Strategies** - Concat, Vote, and LLM-based result merging
- **Timeout & Cancellation** - Graceful shutdown with CancellationToken
- **Unified Executor** - Single executor supporting sequential, parallel, and DAG modes
- **Integration Tests** - Comprehensive end-to-end workflow testing

## Installation

### Prerequisites

- Rust 1.70+ and Cargo
- Git
- **Ollama** (optional, for local LLM) - [Install Ollama](https://ollama.ai)
- **OpenAI API Key** (optional, for cloud fallback)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/apus3404-oss/RustForge.git
cd RustForge

# Build release binary
cargo build --release

# Binary will be at target/release/rustforge
./target/release/rustforge --version
```

### Setup LLM Providers

**Option 1: Ollama (Local, Recommended)**
```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model
ollama pull llama2
```

**Option 2: OpenAI (Cloud Fallback)**
```bash
# Set API key
export OPENAI_API_KEY="your-api-key"
```

### Add to PATH (Optional)

```bash
# Linux/macOS
export PATH="$PATH:$(pwd)/target/release"

# Or copy to system location
sudo cp target/release/rustforge /usr/local/bin/
```

## Quick Start

### 1. Initialize a Project

```bash
rustforge init
```

This creates:
- `.rustforge/` - Configuration and state database
- `workflows/` - Directory for workflow definitions
- `workflows/example.yaml` - Example workflow to get started

### 2. Create a Workflow

Create `workflows/hello.yaml`:

```yaml
name: "Hello Workflow"
mode: sequential
agents:
  - id: greeter
    type: base
    task: "Say hello to the user"
  
  - id: responder
    type: base
    task: "Respond to: {greeter.output}"
```

### 3. Run the Workflow

```bash
rustforge run workflows/hello.yaml
```

With inputs:

```bash
rustforge run workflows/hello.yaml --inputs '{"user": "Alice"}'
```

## CLI Commands

### `rustforge init [PATH]`

Initialize a new RustForge project.

```bash
rustforge init              # Initialize in current directory
rustforge init ./my-project # Initialize in specific directory
```

### `rustforge run <WORKFLOW> [OPTIONS]`

Execute a workflow.

```bash
rustforge run workflows/my-workflow.yaml
rustforge run workflows/my-workflow.yaml --inputs '{"key": "value"}'
rustforge run workflows/my-workflow.yaml --resume  # Resume from checkpoint
```

**Options:**
- `-i, --inputs <JSON>` - Provide workflow inputs as JSON string
- `-r, --resume` - Resume execution from last checkpoint

### `rustforge validate <WORKFLOW>`

Validate a workflow definition without executing it.

```bash
rustforge validate workflows/my-workflow.yaml
```

Checks for:
- Valid YAML syntax
- Required fields (name, mode, agents)
- Unique agent IDs
- Valid dependency references
- Circular dependency detection

### `rustforge list`

List all available workflows in the `workflows/` directory.

```bash
rustforge list
```

### `rustforge config <SUBCOMMAND>`

Manage configuration.

```bash
rustforge config show                                    # Show full config
rustforge config get execution.max_parallel_agents       # Get specific value
rustforge config set execution.max_parallel_agents 20    # Set value
```

**Common config keys:**
- `execution.max_parallel_agents` - Max concurrent agents (default: 10)
- `execution.default_timeout` - Timeout in seconds (default: 300)
- `llm.default_provider` - Default LLM provider (default: "ollama:llama3")
- `logging.level` - Log level: debug, info, warn, error (default: "info")
- `ui.port` - UI server port (default: 3000)

## Workflow Syntax

### Basic Structure

```yaml
name: "Workflow Name"
mode: sequential  # Execution mode (sequential only in Phase 1)

agents:
  - id: agent1           # Unique identifier
    type: base           # Agent type (currently 'base', more types in Phase 3)
    task: "Task description"
    
  - id: agent2
    type: base
    task: "Use output from agent1: {agent1.output}"
```

### Variable Interpolation

Reference outputs from previous agents or workflow inputs:

```yaml
agents:
  - id: analyzer
    type: base
    task: "Analyze: {input.document}"
  
  - id: summarizer
    type: base
    task: "Summarize: {analyzer.output}"
```

Variables are resolved at runtime using the execution context.

### Parallel Execution (Phase 4)

Execute multiple agents concurrently:

```yaml
name: "Parallel Analysis"
mode: parallel  # Agents run concurrently

agents:
  - id: analyzer1
    type: base
    task: "Analyze dataset A"
  
  - id: analyzer2
    type: base
    task: "Analyze dataset B"
  
  - id: analyzer3
    type: base
    task: "Analyze dataset C"
```

Results can be merged using different strategies:
- **concat**: Concatenate all results with newlines
- **vote**: Return most common result
- **llm_merge**: Intelligently synthesize results using LLM

## REST API (Phase 4)

RustForge provides a full REST API for workflow management and execution.

### Start API Server

```bash
rustforge serve --port 8080
```

### API Endpoints

**Workflow Management:**
- `POST /api/workflows` - Create workflow
- `GET /api/workflows` - List workflows
- `GET /api/workflows/:id` - Get workflow details
- `DELETE /api/workflows/:id` - Delete workflow

**Execution:**
- `POST /api/workflows/:id/execute` - Execute workflow
- `GET /api/executions/:id` - Get execution status
- `GET /api/executions` - List executions
- `POST /api/executions/:id/pause` - Pause execution
- `POST /api/executions/:id/resume` - Resume execution
- `DELETE /api/executions/:id` - Cancel execution

**WebSocket:**
- `WS /api/ws/executions/:id` - Real-time execution events

### Example: Execute Workflow via API

```bash
# Create workflow
curl -X POST http://localhost:8080/api/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "name": "API Test",
    "mode": "parallel",
    "agents": [
      {"id": "agent1", "type": "base", "task": "Task 1"},
      {"id": "agent2", "type": "base", "task": "Task 2"}
    ]
  }'

# Execute workflow
curl -X POST http://localhost:8080/api/workflows/{id}/execute

# Get execution status
curl http://localhost:8080/api/executions/{execution_id}
```

### WebSocket Events

Connect to WebSocket for real-time execution updates:

```javascript
const ws = new WebSocket('ws://localhost:8080/api/ws/executions/{id}');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data);
  // Events: TaskStarted, TaskCompleted, TaskFailed
};

// Send control commands
ws.send(JSON.stringify({ type: 'pause' }));
ws.send(JSON.stringify({ type: 'resume' }));
ws.send(JSON.stringify({ type: 'cancel' }));
```

Variables are resolved at runtime using the execution context.

## Architecture

RustForge uses a layered architecture:

```
┌─────────────────────────────────────┐
│         CLI Layer (clap)            │  User interface
├─────────────────────────────────────┤
│      Config Layer (TOML/YAML)       │  Configuration management
├─────────────────────────────────────┤
│     Storage Layer (redb)            │  State persistence
├─────────────────────────────────────┤
│   Orchestration Engine (tokio)      │  Workflow execution
│  • Parser  • Validator              │
│  • Executor  • Interpolator         │
│  • Event Bus  • Checkpoints         │
└─────────────────────────────────────┘
```

**Key Components:**

- **Config Layer** - Multi-source configuration with priority: env vars → project config → user config → defaults
- **Storage Layer** - Embedded redb database for execution state, checkpoints, and audit logs
- **Engine Layer** - Workflow parsing, validation, variable interpolation, and sequential execution
- **Event Bus** - Real-time event streaming for monitoring and UI integration

For detailed architecture documentation, see `docs/specs/design.md`.

## Configuration

Configuration is loaded from multiple sources with the following priority:

1. **Environment Variables** (highest priority)
   - `RUSTFORGE_DEFAULT_LLM`
   - `RUSTFORGE_MAX_PARALLEL_AGENTS`
   - `RUSTFORGE_LOG_LEVEL`

2. **Project Config** - `.rustforge/config.toml` in current directory

3. **User Config** - `~/.rustforge/config.toml`

4. **Defaults** (lowest priority)

### Example Configuration

```toml
[llm]
default_provider = "ollama:llama3"
fallback_enabled = true

[llm.providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama3"
timeout_secs = 300

[execution]
max_parallel_agents = 10
default_timeout = 300
enable_checkpoints = true
checkpoint_interval = 60

[permissions]
default_policy = "prompt"  # allow, deny, or prompt
audit_log_enabled = true

[logging]
level = "info"
format = "pretty"  # json, pretty, or compact
```

## Development

### Run Tests

```bash
cargo test
```

### Run with Debug Logging

```bash
RUSTFORGE_LOG_LEVEL=debug cargo run -- run workflows/example.yaml
```

### Project Structure

```
rustforge/
├── src/
│   ├── cli/          # CLI commands and handlers
│   ├── config/       # Configuration types and loader
│   ├── engine/       # Workflow execution engine
│   ├── storage/      # State persistence layer
│   ├── error.rs      # Error types
│   └── main.rs       # Entry point
├── tests/
│   ├── integration/  # Integration tests
│   └── fixtures/     # Test workflows
├── workflows/        # User workflow definitions
└── .rustforge/       # Config and state database
```

## Roadmap

- ✅ **Phase 1: Core Foundation** (Completed)
  - Config management, CLI, storage, workflow engine
  
- ✅ **Phase 2: LLM & Agent Layer** (Completed)
  - Real AI agent implementations with BaseAgent
  - LLM provider integrations (Ollama, OpenAI)
  - Memory store for conversation history
  - Thread-safe registries for agents and providers
  
- ✅ **Phase 3: Tool & Security Layer** (Completed)
  - 6 built-in tools (FileSystem, WebScraper, PDF, Shell, API, Clipboard)
  - Tool registry and execution framework
  - Permission system with allow/deny/prompt policies
  - Process isolation for secure subprocess execution
  - Audit logging for security events
  - Path and command validation
  
- 🚧 **Phase 4: API & Execution Patterns** (Next)
  - REST API for workflow execution
  - Parallel execution mode
  - Advanced execution patterns
  - Tool result caching
  
- 📋 **Phase 5+: Advanced Features**
  - Web UI dashboard
  - Plugin system
  - Additional LLM providers (Anthropic, etc.)
  - Distributed execution

## Contributing

Contributions are welcome! Please see `CONTRIBUTING.md` for guidelines.

## License

MIT License - see LICENSE file for details.

## Links

- **Repository:** https://github.com/apus3404-oss/RustForge
- **Documentation:** `docs/getting-started.md`
- **Design Specs:** `docs/specs/design.md`

---

**Note:** Phase 3 is complete with 6 built-in tools, permission system, process isolation, and audit logging. Agents can now execute tools with security controls. Phase 4 will add REST API, parallel execution, and advanced execution patterns.
