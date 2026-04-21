// src/cli/commands.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rustforge")]
#[command(version, about = "Local AI Agent Orchestrator", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new RustForge project
    Init {
        /// Project directory (defaults to current directory)
        path: Option<PathBuf>,
    },

    /// Run a workflow
    Run {
        /// Path to workflow YAML file
        workflow: PathBuf,

        /// Workflow inputs as JSON string
        #[arg(short, long)]
        inputs: Option<String>,

        /// Resume from checkpoint
        #[arg(short, long)]
        resume: bool,
    },

    /// Validate a workflow definition
    Validate {
        /// Path to workflow YAML file
        workflow: PathBuf,
    },

    /// List available workflows
    List,

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Get a configuration value
    Get {
        /// Configuration key (e.g., "execution.max_parallel_agents")
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_run_command() {
        let cli = Cli::parse_from(["rustforge", "run", "workflow.yaml"]);
        match cli.command {
            Commands::Run { workflow, inputs, resume } => {
                assert_eq!(workflow, PathBuf::from("workflow.yaml"));
                assert_eq!(inputs, None);
                assert!(!resume);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parse_run_with_inputs() {
        let cli = Cli::parse_from([
            "rustforge",
            "run",
            "workflow.yaml",
            "--inputs",
            r#"{"key":"value"}"#,
        ]);
        match cli.command {
            Commands::Run { workflow, inputs, resume } => {
                assert_eq!(workflow, PathBuf::from("workflow.yaml"));
                assert_eq!(inputs, Some(r#"{"key":"value"}"#.to_string()));
                assert!(!resume);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parse_validate_command() {
        let cli = Cli::parse_from(["rustforge", "validate", "workflow.yaml"]);
        match cli.command {
            Commands::Validate { workflow } => {
                assert_eq!(workflow, PathBuf::from("workflow.yaml"));
            }
            _ => panic!("Expected Validate command"),
        }
    }

    #[test]
    fn test_cli_parse_init_command() {
        let cli = Cli::parse_from(["rustforge", "init"]);
        match cli.command {
            Commands::Init { path } => {
                assert_eq!(path, None);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_parse_config_show() {
        let cli = Cli::parse_from(["rustforge", "config", "show"]);
        match cli.command {
            Commands::Config { command: ConfigCommands::Show } => {}
            _ => panic!("Expected Config Show command"),
        }
    }

    #[test]
    fn test_cli_parse_list_command() {
        let cli = Cli::parse_from(["rustforge", "list"]);
        match cli.command {
            Commands::List => {}
            _ => panic!("Expected List command"),
        }
    }
}
