# Getting Started with RustForge

This guide will walk you through setting up RustForge, creating your first workflow, and understanding the core concepts.

**Current Version:** Phase 2 - LLM + Agent Layer

## Table of Contents

1. [Installation](#installation)
2. [LLM Setup](#llm-setup)
3. [Your First Project](#your-first-project)
4. [Understanding Workflows](#understanding-workflows)
5. [Variable Interpolation](#variable-interpolation)
6. [Configuration](#configuration)
7. [Common Patterns](#common-patterns)
8. [Troubleshooting](#troubleshooting)

## Installation

### Step 1: Install Rust

If you don't have Rust installed:

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source $HOME/.cargo/env
```

Verify installation:

```bash
rustc --version
cargo --version
```

### Step 2: Clone and Build RustForge

```bash
# Clone the repository
git clone https://github.com/apus3404-oss/RustForge.git
cd RustForge

# Build in release mode (optimized)
cargo build --release

# Test the binary
./target/release/rustforge --version
```

### Step 3: Add to PATH (Optional but Recommended)

**Linux/macOS:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:/path/to/RustForge/target/release"

# Or create a symlink
sudo ln -s /path/to/RustForge/target/release/rustforge /usr/local/bin/rustforge
```

**Windows:**
```powershell
# Add to PATH via System Properties > Environment Variables
# Or copy rustforge.exe to a directory already in PATH
```

## LLM Setup

RustForge supports two LLM providers: **Ollama** (local) and **OpenAI** (cloud). You need at least one configured.

### Option 1: Ollama (Recommended for Privacy)

**Install Ollama:**
```bash
# Linux/macOS
curl -fsSL https://ollama.ai/install.sh | sh

# Or visit https://ollama.ai for other platforms
```

**Pull a model:**
```bash
# Recommended: Llama 2 (7B)
ollama pull llama2

# Or other models
ollama pull mistral
ollama pull codellama
```

**Verify:**
```bash
ollama list
```

### Option 2: OpenAI (Cloud Fallback)

**Set API key:**
```bash
# Linux/macOS
export OPENAI_API_KEY="sk-your-api-key-here"

# Add to ~/.bashrc or ~/.zshrc for persistence
echo 'export OPENAI_API_KEY="sk-your-api-key-here"' >> ~/.bashrc
```

**Windows:**
```powershell
$env:OPENAI_API_KEY="sk-your-api-key-here"
```

### Hybrid Setup (Best of Both Worlds)

Configure both providers - RustForge will use Ollama by default and automatically fall back to OpenAI if Ollama is unavailable.

## Your First Project

### Initialize a Project

Create a new directory for your workflows:

```bash
mkdir my-rustforge-project
cd my-rustforge-project
rustforge init
```

You'll see output like:

```
✓ Created config at .rustforge/config.toml
✓ Created example workflow at workflows/example.yaml

✓ RustForge project initialized at /path/to/my-rustforge-project

Next steps:
  1. Edit workflows/example.yaml
  2. Run: rustforge run workflows/example.yaml
```

### Project Structure

After initialization, your project looks like:

```
my-rustforge-project/
├── .rustforge/
│   ├── config.toml      # Project configuration
│   └── state.db         # State database (created on first run)
└── workflows/
    └── example.yaml     # Example workflow
```

### Run the Example Workflow

```bash
rustforge run workflows/example.yaml
```

You'll see:

```
Running workflow: workflows/example.yaml
✓ Loaded workflow: Example Workflow
✓ Starting execution...
✓ Workflow completed successfully
```

## Understanding Workflows

### Workflow Anatomy

A workflow is a YAML file that defines a sequence of agent tasks:

```yaml
name: "My First Workflow"
mode: sequential

agents:
  - id: step1
    type: base
    task: "First task description"
  
  - id: step2
    type: base
    task: "Second task description"
```

**Key Fields:**

- `name` - Human-readable workflow name
- `mode` - Execution mode (`sequential` in Phase 1)
- `agents` - List of agent configurations
  - `id` - Unique identifier for the agent
  - `type` - Agent type (Phase 2 will add real agent types)
  - `task` - Task description or prompt

### Creating a Simple Workflow

Let's create a document processing workflow:

**workflows/process-document.yaml:**

```yaml
name: "Document Processing Pipeline"
mode: sequential

agents:
  - id: reader
    type: base
    task: "Read the document from {input.file_path}"
  
  - id: analyzer
    type: base
    task: "Analyze the content: {reader.output}"
  
  - id: summarizer
    type: base
    task: "Create a summary of: {analyzer.output}"
  
  - id: formatter
    type: base
    task: "Format the summary as markdown: {summarizer.output}"
```

### Validate Your Workflow

Before running, validate the syntax:

```bash
rustforge validate workflows/process-document.yaml
```

If valid, you'll see:

```
✓ Workflow is valid
  - Name: Document Processing Pipeline
  - Mode: sequential
  - Agents: 4
```

If there are errors:

```
✗ Validation failed:
  - Duplicate agent ID: 'analyzer'
  - Invalid dependency: 'unknown_agent.output'
```

## Variable Interpolation

Variables allow agents to reference outputs from previous agents or workflow inputs.

### Syntax

Variables use curly braces: `{source.field}`

**Sources:**
- `{input.field}` - Workflow input data
- `{agent_id.output}` - Output from a specific agent
- `{agent_id.field}` - Specific field from agent output (if structured)

### Example: Using Inputs

**workflows/greet-user.yaml:**

```yaml
name: "Personalized Greeting"
mode: sequential

agents:
  - id: greeter
    type: base
    task: "Greet {input.name} who is a {input.role}"
  
  - id: follow_up
    type: base
    task: "Based on this greeting: {greeter.output}, ask about their day"
```

Run with inputs:

```bash
rustforge run workflows/greet-user.yaml --inputs '{"name": "Alice", "role": "developer"}'
```

### Example: Chaining Outputs

**workflows/data-pipeline.yaml:**

```yaml
name: "Data Processing Pipeline"
mode: sequential

agents:
  - id: fetch
    type: base
    task: "Fetch data from {input.source}"
  
  - id: clean
    type: base
    task: "Clean this data: {fetch.output}"
  
  - id: transform
    type: base
    task: "Transform cleaned data: {clean.output}"
  
  - id: analyze
    type: base
    task: "Analyze: {transform.output}"
  
  - id: report
    type: base
    task: "Generate report from analysis: {analyze.output}"
```

### Error Handling

If you reference a non-existent variable:

```yaml
task: "Process {unknown_agent.output}"
```

You'll get a helpful error:

```
✗ Variable 'unknown_agent' not found
  Did you mean: 'known_agent'?
```

RustForge uses fuzzy matching to suggest similar variable names.

## Configuration

### Configuration Layers

RustForge loads configuration from multiple sources:

1. **Environment Variables** (highest priority)
2. **Project Config** - `.rustforge/config.toml`
3. **User Config** - `~/.rustforge/config.toml`
4. **Defaults** (lowest priority)

### View Current Configuration

```bash
rustforge config show
```

### Get Specific Values

```bash
rustforge config get execution.max_parallel_agents
# Output: 10

rustforge config get llm.default_provider
# Output: ollama:llama3
```

### Set Configuration Values

```bash
# Set max parallel agents
rustforge config set execution.max_parallel_agents 20

# Set default LLM provider
rustforge config set llm.default_provider "openai:gpt-4"

# Set log level
rustforge config set logging.level "debug"
```

### Configuration File

Edit `.rustforge/config.toml` directly:

```toml
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

### Environment Variables

Override config with environment variables:

```bash
# Set default LLM
export RUSTFORGE_DEFAULT_LLM="openai:gpt-4"

# Set max parallel agents
export RUSTFORGE_MAX_PARALLEL_AGENTS=20

# Set log level
export RUSTFORGE_LOG_LEVEL=debug

# Run with overrides
rustforge run workflows/my-workflow.yaml
```

## Common Patterns

### Pattern 1: Multi-Step Analysis

```yaml
name: "Research and Analysis"
mode: sequential

agents:
  - id: researcher
    type: base
    task: "Research topic: {input.topic}"
  
  - id: fact_checker
    type: base
    task: "Verify facts in: {researcher.output}"
  
  - id: analyst
    type: base
    task: "Analyze verified research: {fact_checker.output}"
  
  - id: writer
    type: base
    task: "Write article based on: {analyst.output}"
```

### Pattern 2: Content Generation Pipeline

```yaml
name: "Content Creation"
mode: sequential

agents:
  - id: ideator
    type: base
    task: "Generate content ideas for: {input.theme}"
  
  - id: outliner
    type: base
    task: "Create outline from ideas: {ideator.output}"
  
  - id: writer
    type: base
    task: "Write content following outline: {outliner.output}"
  
  - id: editor
    type: base
    task: "Edit and improve: {writer.output}"
  
  - id: formatter
    type: base
    task: "Format as {input.format}: {editor.output}"
```

### Pattern 3: Code Review Workflow

```yaml
name: "Code Review Assistant"
mode: sequential

agents:
  - id: analyzer
    type: base
    task: "Analyze code: {input.code}"
  
  - id: security_checker
    type: base
    task: "Check for security issues: {analyzer.output}"
  
  - id: performance_reviewer
    type: base
    task: "Review performance: {analyzer.output}"
  
  - id: style_checker
    type: base
    task: "Check code style: {input.code}"
  
  - id: report_generator
    type: base
    task: "Generate review report from: {security_checker.output}, {performance_reviewer.output}, {style_checker.output}"
```

## Troubleshooting

### Workflow Not Found

**Error:**
```
Error: No such file or directory (os error 2)
```

**Solution:**
- Check the file path is correct
- Use `rustforge list` to see available workflows
- Ensure you're in the project directory

### Invalid YAML Syntax

**Error:**
```
Error: Failed to parse workflow: invalid type: string "agents", expected a sequence
```

**Solution:**
- Validate YAML syntax (use a YAML validator)
- Check indentation (use spaces, not tabs)
- Ensure `agents` is a list (starts with `-`)

### Variable Not Found

**Error:**
```
Error: Variable 'agent1.output' not found
```

**Solution:**
- Ensure the agent ID exists and is spelled correctly
- Check that the agent runs before the one referencing it
- Use `rustforge validate` to catch these errors early

### Configuration Issues

**Error:**
```
Error: Config error: Failed to parse config file
```

**Solution:**
- Check TOML syntax in `.rustforge/config.toml`
- Use `rustforge config show` to see current config
- Delete config file to regenerate defaults: `rm .rustforge/config.toml && rustforge init`

### Build Errors

**Error:**
```
error: could not compile `rustforge`
```

**Solution:**
- Ensure Rust is up to date: `rustup update`
- Clean and rebuild: `cargo clean && cargo build --release`
- Check Rust version: `rustc --version` (need 1.70+)

## Next Steps

Now that you understand the basics:

1. **Explore Examples** - Check `tests/fixtures/workflows/` for more examples
2. **Read the Design Docs** - See `docs/specs/design.md` for architecture details
3. **Experiment** - Create your own workflows and test different patterns
4. **Configure LLMs** - Set up Ollama for local execution or OpenAI for cloud fallback
5. **Build Custom Agents** - Phase 3 will add specialized agent types and tool calling

## Getting Help

- **Issues:** https://github.com/apus3404-oss/RustForge/issues
- **Discussions:** https://github.com/apus3404-oss/RustForge/discussions
- **Documentation:** Check `docs/` directory for detailed specs

---

**Remember:** Phase 2 provides real LLM integration with Ollama and OpenAI. The BaseAgent executes tasks using actual LLM calls. Phase 3 will add specialized agent types, tool calling, and advanced features.
