// src/cli/handlers.rs
use crate::agents::AgentRegistry;
use crate::cli::commands::{Commands, ConfigCommands};
use crate::config::{ConfigLoader, GlobalConfig};
use crate::engine::{EventBus, ExecutionContext, SequentialExecutor, WorkflowParser};
use crate::error::Result;
use crate::llm::{LLMRegistry, OllamaProvider, OpenAIProvider};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Init { path } => handle_init(path).await,
        Commands::Run {
            workflow,
            inputs,
            resume,
        } => handle_run(workflow, inputs, resume).await,
        Commands::Validate { workflow } => handle_validate(workflow).await,
        Commands::List => handle_list().await,
        Commands::Config { command } => handle_config(command).await,
    }
}

async fn handle_init(path: Option<PathBuf>) -> Result<()> {
    let project_dir = path.unwrap_or_else(|| env::current_dir().unwrap());

    // Create .rustforge directory
    let rustforge_dir = project_dir.join(".rustforge");
    fs::create_dir_all(&rustforge_dir)?;

    // Create workflows directory
    let workflows_dir = project_dir.join("workflows");
    fs::create_dir_all(&workflows_dir)?;

    // Create example config
    let config_path = rustforge_dir.join("config.toml");
    if !config_path.exists() {
        let default_config = GlobalConfig::default();
        ConfigLoader::save_to_file(&default_config, &config_path)?;
        println!("✓ Created config at {}", config_path.display());
    }

    // Create example workflow
    let example_workflow_path = workflows_dir.join("example.yaml");
    if !example_workflow_path.exists() {
        let example_workflow = r#"name: "Example Workflow"
mode: sequential
agents:
  - id: agent1
    type: ExampleAgent
    task: "Perform example task"
"#;
        fs::write(&example_workflow_path, example_workflow)?;
        println!("✓ Created example workflow at {}", example_workflow_path.display());
    }

    println!("\n✓ RustForge project initialized at {}", project_dir.display());
    println!("\nNext steps:");
    println!("  1. Edit workflows/example.yaml");
    println!("  2. Run: rustforge run workflows/example.yaml");

    Ok(())
}

async fn handle_run(
    workflow_path: PathBuf,
    inputs: Option<String>,
    _resume: bool,
) -> Result<()> {
    println!("Running workflow: {}", workflow_path.display());

    // Load config
    let _config = ConfigLoader::load()?;

    // Parse workflow
    let workflow = WorkflowParser::parse_file(&workflow_path)?;
    println!("✓ Loaded workflow: {}", workflow.name);

    // Create execution context
    let mut context = ExecutionContext::new(workflow.name.clone());

    // Parse and set inputs if provided
    if let Some(inputs_json) = inputs {
        let inputs_value: serde_json::Value = serde_json::from_str(&inputs_json)
            .map_err(|e| crate::error::Error::Config(format!("Invalid inputs JSON: {}", e)))?;
        context.set_value("input", inputs_value);
    }

    // Create event bus and executor
    let event_bus = Arc::new(EventBus::new());

    // Create LLM registry with Ollama and OpenAI providers
    let ollama = Arc::new(OllamaProvider::new(
        "http://localhost:11434".to_string(),
        "llama3".to_string(),
    ));
    let openai = Arc::new(OpenAIProvider::new(
        env::var("OPENAI_API_KEY").unwrap_or_default(),
        "gpt-4o-mini".to_string(),
    ));

    let llm_registry = Arc::new(LLMRegistry::with_fallback(ollama.clone(), openai.clone()));

    // Create agent registry
    let agent_registry = Arc::new(AgentRegistry::new());

    let executor = SequentialExecutor::new(event_bus.clone(), llm_registry, agent_registry);

    // Execute workflow
    println!("✓ Starting execution...");
    let result = executor.execute(&workflow, &mut context).await?;

    println!("\n✓ Workflow completed successfully");
    println!("Result: {}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

async fn handle_validate(workflow_path: PathBuf) -> Result<()> {
    println!("Validating workflow: {}", workflow_path.display());

    // Parse workflow
    let workflow = WorkflowParser::parse_file(&workflow_path)?;

    println!("✓ Workflow is valid");
    println!("  Name: {}", workflow.name);
    println!("  Mode: {:?}", workflow.mode);
    println!("  Agents: {}", workflow.agents.len());

    for agent in &workflow.agents {
        println!("    - {} ({})", agent.id, agent.agent_type);
    }

    Ok(())
}

async fn handle_list() -> Result<()> {
    let workflows_dir = Path::new("workflows");

    if !workflows_dir.exists() {
        println!("No workflows directory found. Run 'rustforge init' first.");
        return Ok(());
    }

    println!("Available workflows:");
    for entry in fs::read_dir(workflows_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            println!("  - {}", path.file_stem().unwrap().to_str().unwrap());
        }
    }

    Ok(())
}

async fn handle_config(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            let config = ConfigLoader::load()?;
            let toml_str = toml::to_string_pretty(&config)
                .map_err(|e| crate::error::Error::Config(format!("Failed to serialize config: {}", e)))?;
            println!("{}", toml_str);
            Ok(())
        }
        ConfigCommands::Get { key } => {
            let config = ConfigLoader::load()?;
            let value = get_config_value(&config, &key)?;
            println!("{}", value);
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            let mut config = ConfigLoader::load()?;
            set_config_value(&mut config, &key, &value)?;

            // Save to project config
            let config_path = env::current_dir()?.join(".rustforge").join("config.toml");
            ConfigLoader::save_to_file(&config, &config_path)?;

            println!("✓ Set {} = {}", key, value);
            Ok(())
        }
    }
}

fn get_config_value(config: &GlobalConfig, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["execution", "max_parallel_agents"] => Ok(config.execution.max_parallel_agents.to_string()),
        ["execution", "default_timeout"] => Ok(config.execution.default_timeout.as_secs().to_string()),
        ["llm", "default_provider"] => Ok(config.llm.default_provider.clone()),
        ["logging", "level"] => Ok(config.logging.level.clone()),
        ["ui", "port"] => Ok(config.ui.port.to_string()),
        _ => Err(crate::error::Error::Config(format!("Unknown config key: {}", key))),
    }
}

fn set_config_value(config: &mut GlobalConfig, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["execution", "max_parallel_agents"] => {
            config.execution.max_parallel_agents = value.parse()
                .map_err(|_| crate::error::Error::Config("Invalid number".to_string()))?;
        }
        ["execution", "default_timeout"] => {
            let secs: u64 = value.parse()
                .map_err(|_| crate::error::Error::Config("Invalid number".to_string()))?;
            config.execution.default_timeout = std::time::Duration::from_secs(secs);
        }
        ["llm", "default_provider"] => {
            config.llm.default_provider = value.to_string();
        }
        ["logging", "level"] => {
            config.logging.level = value.to_string();
        }
        ["ui", "port"] => {
            config.ui.port = value.parse()
                .map_err(|_| crate::error::Error::Config("Invalid port number".to_string()))?;
        }
        _ => return Err(crate::error::Error::Config(format!("Unknown config key: {}", key))),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_init() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_init(Some(temp_dir.path().to_path_buf())).await;
        assert!(result.is_ok());

        // Verify directories were created
        assert!(temp_dir.path().join(".rustforge").exists());
        assert!(temp_dir.path().join("workflows").exists());
        assert!(temp_dir.path().join(".rustforge/config.toml").exists());
        assert!(temp_dir.path().join("workflows/example.yaml").exists());
    }

    #[tokio::test]
    async fn test_handle_validate() {
        let temp_dir = TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.yaml");

        let workflow_content = r#"
name: "Test Workflow"
mode: sequential
agents:
  - id: agent1
    type: TestAgent
    task: "Do something"
"#;
        fs::write(&workflow_path, workflow_content).unwrap();

        let result = handle_validate(workflow_path).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_list() {
        let temp_dir = TempDir::new().unwrap();
        let workflows_dir = temp_dir.path().join("workflows");
        fs::create_dir_all(&workflows_dir).unwrap();

        // Create test workflow files
        fs::write(workflows_dir.join("workflow1.yaml"), "name: Test1\nmode: sequential\nagents: []").unwrap();
        fs::write(workflows_dir.join("workflow2.yaml"), "name: Test2\nmode: sequential\nagents: []").unwrap();
        fs::write(workflows_dir.join("readme.txt"), "not a workflow").unwrap();

        // Change to temp directory for the test
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        let result = handle_list().await;
        assert!(result.is_ok());

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_get_config_value() {
        let config = GlobalConfig::default();

        let value = get_config_value(&config, "execution.max_parallel_agents").unwrap();
        assert_eq!(value, "10");

        let value = get_config_value(&config, "llm.default_provider").unwrap();
        assert_eq!(value, "ollama:llama3");
    }

    #[test]
    fn test_set_config_value() {
        let mut config = GlobalConfig::default();

        set_config_value(&mut config, "execution.max_parallel_agents", "20").unwrap();
        assert_eq!(config.execution.max_parallel_agents, 20);

        set_config_value(&mut config, "llm.default_provider", "openai:gpt-4").unwrap();
        assert_eq!(config.llm.default_provider, "openai:gpt-4");
    }

    #[test]
    fn test_get_unknown_config_key() {
        let config = GlobalConfig::default();
        let result = get_config_value(&config, "unknown.key");
        assert!(result.is_err());
    }
}
