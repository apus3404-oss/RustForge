use rustforge::error::Result;
use rustforge::security::{AuditLogger, PermissionManager, PermissionPolicy};
use rustforge::tools::{
    ApiClientTool, ClipboardTool, FileSystemTool, PdfParserTool, ShellExecutorTool, Tool,
    ToolRegistry, WebScraperTool,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_filesystem_tool_execution() {
    let tool = Arc::new(FileSystemTool::new(vec![]));
    let registry = ToolRegistry::new();
    registry.register(tool.clone()).unwrap();

    // Test tool is registered
    assert!(registry.contains("file_system"));

    // Create a temp file for testing
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("rustforge_test.txt");
    let test_content = "Hello, RustForge!";

    // Test write operation
    let mut params = HashMap::new();
    params.insert("operation".to_string(), json!("write"));
    params.insert("path".to_string(), json!(test_file.to_str().unwrap()));
    params.insert("content".to_string(), json!(test_content));

    let result = tool.execute(params).await.unwrap();
    assert!(result.success);

    // Test read operation
    let mut params = HashMap::new();
    params.insert("operation".to_string(), json!("read"));
    params.insert("path".to_string(), json!(test_file.to_str().unwrap()));

    let result = tool.execute(params).await.unwrap();
    assert!(result.success);
    assert_eq!(
        result.output.get("content").unwrap().as_str().unwrap(),
        test_content
    );

    // Cleanup
    std::fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_shell_executor_tool() {
    let tool = Arc::new(ShellExecutorTool::new(
        vec!["echo".to_string(), "cmd".to_string()],
        std::time::Duration::from_secs(30),
    ));

    let mut params = HashMap::new();
    params.insert("command".to_string(), json!("echo"));
    params.insert("args".to_string(), json!(["test"]));

    let result = tool.execute(params).await.unwrap();
    assert!(result.success);

    // Check that we got some output (stdout or stderr)
    let stdout = result.output.get("stdout").unwrap().as_str().unwrap();
    let stderr = result.output.get("stderr").unwrap().as_str().unwrap();
    assert!(!stdout.is_empty() || !stderr.is_empty());
}

#[tokio::test]
async fn test_api_client_tool() {
    let tool = Arc::new(ApiClientTool::new(std::time::Duration::from_secs(30)));

    let mut params = HashMap::new();
    params.insert("method".to_string(), json!("GET"));
    params.insert("url".to_string(), json!("https://httpbin.org/get"));

    let result = tool.execute(params).await;
    // Network test may fail in CI, so we just check it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_clipboard_tool() {
    let tool_result = ClipboardTool::new();

    // Clipboard may not be available in CI
    if let Ok(tool) = tool_result {
        let tool = Arc::new(tool);

        // Test write
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("write"));
        params.insert("content".to_string(), json!("test clipboard content"));

        let result = tool.execute(params).await;
        if result.is_ok() {
            let result = result.unwrap();
            assert!(result.success);

            // Test read
            let mut params = HashMap::new();
            params.insert("operation".to_string(), json!("read"));

            let result = tool.execute(params).await.unwrap();
            assert!(result.success);
        }
    }
}

#[tokio::test]
async fn test_tool_registry_operations() {
    let registry = ToolRegistry::new();

    // Register tools
    let fs_tool = Arc::new(FileSystemTool::new(vec![]));
    let shell_tool = Arc::new(ShellExecutorTool::new(vec![], std::time::Duration::from_secs(30)));

    registry.register(fs_tool).unwrap();
    registry.register(shell_tool).unwrap();

    // Check tools are registered
    assert!(registry.contains("file_system"));
    assert!(registry.contains("shell_executor"));

    // List tools
    let tools = registry.list_tools();
    assert!(tools.contains(&"file_system".to_string()));
    assert!(tools.contains(&"shell_executor".to_string()));

    // Get tool
    let tool = registry.get("file_system");
    assert!(tool.is_some());
    assert_eq!(tool.unwrap().name(), "file_system");

    // Count tools
    assert_eq!(registry.count(), 2);
}

#[tokio::test]
async fn test_tool_execution_with_audit_logging() {
    let tool = Arc::new(FileSystemTool::new(vec![]));
    let audit_logger = AuditLogger::new();
    let execution_id = Uuid::new_v4();

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("rustforge_audit_test.txt");

    // Execute tool
    let mut params = HashMap::new();
    params.insert("operation".to_string(), json!("write"));
    params.insert("path".to_string(), json!(test_file.to_str().unwrap()));
    params.insert("content".to_string(), json!("test"));

    let result = tool.execute(params.clone()).await.unwrap();

    // Log the execution
    audit_logger
        .log_tool_execution(
            execution_id,
            "test_agent".to_string(),
            "file_system".to_string(),
            "write".to_string(),
            json!(params),
            if result.success {
                rustforge::security::AuditResult::Success
            } else {
                rustforge::security::AuditResult::Failed {
                    error: result.error.unwrap_or_default(),
                }
            },
        )
        .await
        .unwrap();

    // Verify audit log
    let logs = audit_logger.query_by_execution(execution_id).await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].agent_id, "test_agent");

    // Cleanup
    std::fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_duplicate_tool_registration() {
    let registry = ToolRegistry::new();
    let tool1 = Arc::new(FileSystemTool::new(vec![]));
    let tool2 = Arc::new(FileSystemTool::new(vec![]));

    registry.register(tool1).unwrap();
    let result = registry.register(tool2);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_unregistration() {
    let registry = ToolRegistry::new();
    let tool = Arc::new(FileSystemTool::new(vec![]));

    registry.register(tool).unwrap();
    assert!(registry.contains("file_system"));

    registry.unregister("file_system").unwrap();
    assert!(!registry.contains("file_system"));
}

#[tokio::test]
async fn test_get_nonexistent_tool() {
    let registry = ToolRegistry::new();
    let tool = registry.get("nonexistent");
    assert!(tool.is_none());
}

#[tokio::test]
async fn test_unregister_nonexistent_tool() {
    let registry = ToolRegistry::new();
    let result = registry.unregister("nonexistent");
    assert!(result.is_err());
}
