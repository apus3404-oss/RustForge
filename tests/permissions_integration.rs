use rustforge::error::Result;
use rustforge::security::{AuditLogger, PermissionManager, PermissionPolicy};
use rustforge::tools::{FileSystemTool, ShellExecutorTool, Tool};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_permission_allow_all() {
    let manager = PermissionManager::allow_all();

    // All tools should be allowed
    assert!(manager.check_permission("file_system").is_ok());
    assert!(manager.check_permission("shell_executor").is_ok());
    assert!(manager.check_permission("any_tool").is_ok());
}

#[tokio::test]
async fn test_permission_deny_all() {
    let manager = PermissionManager::deny_all();

    // All tools should be denied
    assert!(manager.check_permission("file_system").is_err());
    assert!(manager.check_permission("shell_executor").is_err());
    assert!(manager.check_permission("any_tool").is_err());
}

#[tokio::test]
async fn test_permission_prompt_all() {
    let manager = PermissionManager::prompt_all();

    // In Phase 3, Prompt is treated as Allow
    assert!(manager.check_permission("file_system").is_ok());
    assert!(manager.check_permission("shell_executor").is_ok());
}

#[tokio::test]
async fn test_permission_tool_specific_policy() {
    let manager = PermissionManager::deny_all();

    // Default is deny
    assert!(manager.check_permission("file_system").is_err());

    // Allow specific tool
    manager.set_tool_policy("file_system", PermissionPolicy::Allow);
    assert!(manager.check_permission("file_system").is_ok());

    // Other tools still denied
    assert!(manager.check_permission("shell_executor").is_err());
}

#[tokio::test]
async fn test_permission_override_default() {
    let manager = PermissionManager::allow_all();

    // Default is allow
    assert!(manager.check_permission("dangerous_tool").is_ok());

    // Deny specific tool
    manager.set_tool_policy("dangerous_tool", PermissionPolicy::Deny);
    assert!(manager.check_permission("dangerous_tool").is_err());

    // Other tools still allowed
    assert!(manager.check_permission("safe_tool").is_ok());
}

#[tokio::test]
async fn test_permission_list_policies() {
    let manager = PermissionManager::allow_all();

    manager.set_tool_policy("tool1", PermissionPolicy::Deny);
    manager.set_tool_policy("tool2", PermissionPolicy::Allow);
    manager.set_tool_policy("tool3", PermissionPolicy::Prompt);

    let policies = manager.list_tool_policies();
    assert_eq!(policies.len(), 3);
    assert_eq!(policies.get("tool1"), Some(&PermissionPolicy::Deny));
    assert_eq!(policies.get("tool2"), Some(&PermissionPolicy::Allow));
    assert_eq!(policies.get("tool3"), Some(&PermissionPolicy::Prompt));
}

#[tokio::test]
async fn test_permission_remove_policy() {
    let manager = PermissionManager::deny_all();

    manager.set_tool_policy("file_system", PermissionPolicy::Allow);
    assert!(manager.check_permission("file_system").is_ok());

    // Remove policy, should revert to default (deny)
    assert!(manager.remove_tool_policy("file_system"));
    assert!(manager.check_permission("file_system").is_err());

    // Removing non-existent policy returns false
    assert!(!manager.remove_tool_policy("nonexistent"));
}

#[tokio::test]
async fn test_permission_clear_policies() {
    let manager = PermissionManager::deny_all();

    manager.set_tool_policy("tool1", PermissionPolicy::Allow);
    manager.set_tool_policy("tool2", PermissionPolicy::Allow);
    manager.set_tool_policy("tool3", PermissionPolicy::Allow);

    assert_eq!(manager.list_tool_policies().len(), 3);

    manager.clear_tool_policies();
    assert_eq!(manager.list_tool_policies().len(), 0);

    // All tools should revert to default policy (deny)
    assert!(manager.check_permission("tool1").is_err());
}

#[tokio::test]
async fn test_permission_with_audit_logging() {
    let manager = PermissionManager::deny_all();
    let audit_logger = AuditLogger::new();
    let execution_id = Uuid::new_v4();

    // Try to execute a tool without permission
    let result = manager.check_permission("file_system");
    assert!(result.is_err());

    // Log the denial
    audit_logger
        .log_permission_denied(
            execution_id,
            "test_agent".to_string(),
            "file_system".to_string(),
            "/etc/passwd".to_string(),
            "Permission denied by policy".to_string(),
        )
        .await
        .unwrap();

    // Grant permission
    manager.set_tool_policy("file_system", PermissionPolicy::Allow);
    let result = manager.check_permission("file_system");
    assert!(result.is_ok());

    // Log the grant
    audit_logger
        .log_permission_granted(
            execution_id,
            "test_agent".to_string(),
            "file_system".to_string(),
            "/home/user".to_string(),
        )
        .await
        .unwrap();

    // Verify audit logs
    let logs = audit_logger.query_by_execution(execution_id).await.unwrap();
    assert_eq!(logs.len(), 2);
}

#[tokio::test]
async fn test_tool_execution_with_permission_check() {
    let manager = PermissionManager::deny_all();
    let tool = Arc::new(FileSystemTool::new(vec![]));

    // Check permission before execution
    let permission_result = manager.check_permission("file_system");
    assert!(permission_result.is_err());

    // Don't execute if permission denied
    // (In real implementation, this check would be in the executor)

    // Grant permission
    manager.set_tool_policy("file_system", PermissionPolicy::Allow);
    let permission_result = manager.check_permission("file_system");
    assert!(permission_result.is_ok());

    // Now execute the tool
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("rustforge_permission_test.txt");

    let mut params = HashMap::new();
    params.insert("operation".to_string(), json!("write"));
    params.insert("path".to_string(), json!(test_file.to_str().unwrap()));
    params.insert("content".to_string(), json!("test"));

    let result = tool.execute(params).await.unwrap();
    assert!(result.success);

    // Cleanup
    std::fs::remove_file(test_file).ok();
}

#[tokio::test]
async fn test_permission_default_policy() {
    let allow_manager = PermissionManager::allow_all();
    assert_eq!(allow_manager.default_policy(), PermissionPolicy::Allow);

    let deny_manager = PermissionManager::deny_all();
    assert_eq!(deny_manager.default_policy(), PermissionPolicy::Deny);

    let prompt_manager = PermissionManager::prompt_all();
    assert_eq!(prompt_manager.default_policy(), PermissionPolicy::Prompt);
}

#[tokio::test]
async fn test_permission_manager_default() {
    let manager = PermissionManager::default();
    assert_eq!(manager.default_policy(), PermissionPolicy::Prompt);
}

#[tokio::test]
async fn test_multiple_permission_changes() {
    let manager = PermissionManager::allow_all();

    // Change policy multiple times
    manager.set_tool_policy("test_tool", PermissionPolicy::Deny);
    assert!(manager.check_permission("test_tool").is_err());

    manager.set_tool_policy("test_tool", PermissionPolicy::Allow);
    assert!(manager.check_permission("test_tool").is_ok());

    manager.set_tool_policy("test_tool", PermissionPolicy::Prompt);
    assert!(manager.check_permission("test_tool").is_ok()); // Prompt = Allow in Phase 3

    manager.set_tool_policy("test_tool", PermissionPolicy::Deny);
    assert!(manager.check_permission("test_tool").is_err());
}

#[tokio::test]
async fn test_permission_isolation() {
    let manager1 = PermissionManager::deny_all();
    let manager2 = PermissionManager::allow_all();

    manager1.set_tool_policy("tool1", PermissionPolicy::Allow);
    manager2.set_tool_policy("tool1", PermissionPolicy::Deny);

    // Each manager should have independent state
    assert!(manager1.check_permission("tool1").is_ok());
    assert!(manager2.check_permission("tool1").is_err());

    // Other tools should follow default policies
    assert!(manager1.check_permission("tool2").is_err()); // deny_all default
    assert!(manager2.check_permission("tool2").is_ok()); // allow_all default
}
