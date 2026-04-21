use crate::error::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;

/// Process isolation configuration
#[derive(Debug, Clone)]
pub struct IsolationConfig {
    /// Working directory for the process
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Environment variables to clear
    pub clear_env: bool,
    /// Maximum execution time
    pub timeout: Duration,
    /// Resource limits (not implemented in Phase 3)
    pub resource_limits: ResourceLimits,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            working_dir: None,
            env_vars: HashMap::new(),
            clear_env: false,
            timeout: Duration::from_secs(300),
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Resource limits for isolated processes
#[derive(Debug, Clone, Default)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (not enforced in Phase 3)
    pub max_memory: Option<u64>,
    /// Maximum CPU time in seconds (not enforced in Phase 3)
    pub max_cpu_time: Option<u64>,
}

/// Result of isolated process execution
#[derive(Debug, Clone)]
pub struct IsolationResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

/// Process isolation manager
pub struct ProcessIsolation;

impl ProcessIsolation {
    /// Execute a command in an isolated environment
    pub async fn execute(
        command: &str,
        args: &[String],
        config: IsolationConfig,
    ) -> Result<IsolationResult> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        // Set working directory
        if let Some(working_dir) = config.working_dir {
            cmd.current_dir(working_dir);
        }

        // Configure environment
        if config.clear_env {
            cmd.env_clear();
        }
        for (key, value) in config.env_vars {
            cmd.env(key, value);
        }

        // Execute with timeout
        let output_future = cmd.output();
        let timeout_result = tokio::time::timeout(config.timeout, output_future).await;

        match timeout_result {
            Ok(Ok(output)) => Ok(IsolationResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code(),
                timed_out: false,
            }),
            Ok(Err(e)) => Err(Error::Internal(format!("Process execution failed: {}", e))),
            Err(_) => {
                // Timeout occurred
                Ok(IsolationResult {
                    stdout: String::new(),
                    stderr: "Process execution timed out".to_string(),
                    exit_code: None,
                    timed_out: true,
                })
            }
        }
    }

    /// Execute a shell command in an isolated environment
    pub async fn execute_shell(
        script: &str,
        config: IsolationConfig,
    ) -> Result<IsolationResult> {
        #[cfg(unix)]
        let (shell, flag) = ("sh", "-c");
        #[cfg(windows)]
        let (shell, flag) = ("cmd", "/C");

        Self::execute(shell, &[flag.to_string(), script.to_string()], config).await
    }

    /// Validate command against allowed list
    pub fn validate_command(command: &str, allowed_commands: &[String]) -> Result<()> {
        if allowed_commands.is_empty() {
            return Ok(());
        }

        if allowed_commands.contains(&command.to_string()) {
            Ok(())
        } else {
            Err(Error::Internal(format!(
                "Command '{}' is not in the allowed list",
                command
            )))
        }
    }

    /// Check if a path is within allowed directories
    pub fn validate_path(path: &PathBuf, allowed_paths: &[PathBuf]) -> Result<()> {
        if allowed_paths.is_empty() {
            return Ok(());
        }

        let canonical_path = path
            .canonicalize()
            .map_err(|e| Error::Internal(format!("Failed to canonicalize path: {}", e)))?;

        for allowed in allowed_paths {
            if let Ok(canonical_allowed) = allowed.canonicalize() {
                if canonical_path.starts_with(&canonical_allowed) {
                    return Ok(());
                }
            }
        }

        Err(Error::Internal(format!(
            "Path {:?} is outside allowed directories",
            path
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_simple_command() {
        let config = IsolationConfig::default();

        #[cfg(unix)]
        let result = ProcessIsolation::execute("echo", &["hello".to_string()], config)
            .await
            .unwrap();

        #[cfg(windows)]
        let result = ProcessIsolation::execute("cmd", &["/C".to_string(), "echo hello".to_string()], config)
            .await
            .unwrap();

        assert!(!result.timed_out);
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let config = IsolationConfig {
            timeout: Duration::from_millis(500),
            ..Default::default()
        };

        #[cfg(unix)]
        let result = ProcessIsolation::execute("sleep", &["10".to_string()], config)
            .await
            .unwrap();

        #[cfg(windows)]
        let result = ProcessIsolation::execute("ping", &["127.0.0.1".to_string(), "-n".to_string(), "10".to_string()], config)
            .await
            .unwrap();

        assert!(result.timed_out);
    }

    #[tokio::test]
    async fn test_execute_with_env_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let config = IsolationConfig {
            env_vars,
            ..Default::default()
        };

        #[cfg(unix)]
        let result = ProcessIsolation::execute("sh", &["-c".to_string(), "echo $TEST_VAR".to_string()], config)
            .await
            .unwrap();

        #[cfg(windows)]
        let result = ProcessIsolation::execute("cmd", &["/C".to_string(), "echo %TEST_VAR%".to_string()], config)
            .await
            .unwrap();

        assert!(result.stdout.contains("test_value"));
    }

    #[tokio::test]
    async fn test_execute_shell() {
        let config = IsolationConfig::default();

        #[cfg(unix)]
        let result = ProcessIsolation::execute_shell("echo hello && echo world", config)
            .await
            .unwrap();

        #[cfg(windows)]
        let result = ProcessIsolation::execute_shell("echo hello & echo world", config)
            .await
            .unwrap();

        assert!(result.stdout.contains("hello"));
        assert!(result.stdout.contains("world"));
    }

    #[test]
    fn test_validate_command_allowed() {
        let allowed = vec!["ls".to_string(), "cat".to_string()];
        assert!(ProcessIsolation::validate_command("ls", &allowed).is_ok());
        assert!(ProcessIsolation::validate_command("cat", &allowed).is_ok());
    }

    #[test]
    fn test_validate_command_denied() {
        let allowed = vec!["ls".to_string(), "cat".to_string()];
        assert!(ProcessIsolation::validate_command("rm", &allowed).is_err());
    }

    #[test]
    fn test_validate_command_empty_list() {
        let allowed = vec![];
        assert!(ProcessIsolation::validate_command("anything", &allowed).is_ok());
    }

    #[test]
    fn test_validate_path_allowed() {
        let temp_dir = std::env::temp_dir();
        // Use the temp_dir itself which definitely exists
        let allowed = vec![temp_dir.parent().unwrap().to_path_buf()];

        assert!(ProcessIsolation::validate_path(&temp_dir, &allowed).is_ok());
    }

    #[test]
    fn test_validate_path_denied() {
        let temp_dir = std::env::temp_dir();
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let allowed = vec![temp_dir];

        // Path outside allowed directory should fail
        assert!(ProcessIsolation::validate_path(&home_dir, &allowed).is_err());
    }

    #[test]
    fn test_validate_path_empty_list() {
        let any_path = PathBuf::from(".");
        let allowed = vec![];
        assert!(ProcessIsolation::validate_path(&any_path, &allowed).is_ok());
    }
}
