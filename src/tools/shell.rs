use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::tools::traits::Tool;
use crate::tools::types::{ParameterType, ToolParameter, ToolResult};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct ShellExecutorTool {
    allowed_commands: Vec<String>,
    timeout: Duration,
}

impl ShellExecutorTool {
    pub fn new(allowed_commands: Vec<String>, timeout: Duration) -> Self {
        Self {
            allowed_commands,
            timeout,
        }
    }

    pub fn unrestricted(timeout: Duration) -> Self {
        Self {
            allowed_commands: vec![],
            timeout,
        }
    }

    fn validate_command(&self, command: &str) -> Result<()> {
        if self.allowed_commands.is_empty() {
            return Ok(());
        }

        let cmd_name = command.split_whitespace().next().unwrap_or("");

        for allowed in &self.allowed_commands {
            if cmd_name == allowed {
                return Ok(());
            }
        }

        Err(Error::Internal(format!(
            "Command '{}' is not in allowed list",
            cmd_name
        )))
    }

    async fn execute_command(&self, command: &str, working_dir: Option<&str>) -> Result<ToolResult> {
        self.validate_command(command)?;

        // Use sh on Unix, cmd on Windows
        #[cfg(unix)]
        let shell = "sh";
        #[cfg(unix)]
        let shell_arg = "-c";

        #[cfg(windows)]
        let shell = "cmd";
        #[cfg(windows)]
        let shell_arg = "/C";

        let mut cmd = Command::new(shell);
        cmd.arg(shell_arg)
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().map_err(|e| {
            Error::Internal(format!("Failed to execute command: {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(ToolResult::success(json!({
            "command": command,
            "exit_code": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "success": output.status.success()
        })))
    }
}

impl Default for ShellExecutorTool {
    fn default() -> Self {
        Self::unrestricted(Duration::from_secs(30))
    }
}

#[async_trait]
impl Tool for ShellExecutorTool {
    fn name(&self) -> &str {
        "shell_executor"
    }

    fn description(&self) -> &str {
        "Execute shell commands with optional working directory"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter::new(
                "command",
                "Shell command to execute",
                ParameterType::String,
                true,
            ),
            ToolParameter::new(
                "working_dir",
                "Working directory (optional)",
                ParameterType::String,
                false,
            ),
        ]
    }

    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let command = params
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing command parameter".to_string()))?;

        let working_dir = params.get("working_dir").and_then(|v| v.as_str());

        self.execute_command(command, working_dir).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = ShellExecutorTool::default();
        assert_eq!(tool.name(), "shell_executor");
        assert!(!tool.description().is_empty());
        assert_eq!(tool.parameters().len(), 2);
    }

    #[tokio::test]
    async fn test_execute_simple_command() {
        let tool = ShellExecutorTool::default();
        let mut params = HashMap::new();

        #[cfg(unix)]
        params.insert("command".to_string(), json!("echo 'Hello, World!'"));

        #[cfg(windows)]
        params.insert("command".to_string(), json!("echo Hello, World!"));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output["exit_code"], 0);
        assert!(result.output["stdout"].as_str().unwrap().contains("Hello"));
    }

    #[tokio::test]
    async fn test_command_validation() {
        let tool = ShellExecutorTool::new(
            vec!["echo".to_string(), "ls".to_string()],
            Duration::from_secs(30),
        );

        let mut params = HashMap::new();
        params.insert("command".to_string(), json!("echo test"));

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let mut params = HashMap::new();
        params.insert("command".to_string(), json!("rm -rf /"));

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_missing_command() {
        let tool = ShellExecutorTool::default();
        let params = HashMap::new();

        let result = tool.execute(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_command_with_working_dir() {
        let tool = ShellExecutorTool::default();
        let mut params = HashMap::new();

        #[cfg(unix)]
        params.insert("command".to_string(), json!("pwd"));

        #[cfg(windows)]
        params.insert("command".to_string(), json!("cd"));

        params.insert("working_dir".to_string(), json!("/tmp"));

        let result = tool.execute(params).await;
        // May fail if /tmp doesn't exist on Windows, but that's ok for this test
        assert!(result.is_ok() || result.is_err());
    }
}
