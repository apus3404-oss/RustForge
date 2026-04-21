use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Permission policy for tool execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionPolicy {
    /// Always allow execution
    Allow,
    /// Always deny execution
    Deny,
    /// Prompt user for permission (not implemented in Phase 3)
    Prompt,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        PermissionPolicy::Prompt
    }
}

/// Permission manager for controlling tool execution
pub struct PermissionManager {
    default_policy: PermissionPolicy,
    tool_policies: RwLock<HashMap<String, PermissionPolicy>>,
}

impl PermissionManager {
    /// Create a new permission manager with a default policy
    pub fn new(default_policy: PermissionPolicy) -> Self {
        Self {
            default_policy,
            tool_policies: RwLock::new(HashMap::new()),
        }
    }

    /// Create a permission manager that allows all tools
    pub fn allow_all() -> Self {
        Self::new(PermissionPolicy::Allow)
    }

    /// Create a permission manager that denies all tools
    pub fn deny_all() -> Self {
        Self::new(PermissionPolicy::Deny)
    }

    /// Create a permission manager that prompts for all tools
    pub fn prompt_all() -> Self {
        Self::new(PermissionPolicy::Prompt)
    }

    /// Set permission policy for a specific tool
    pub fn set_tool_policy(&self, tool_name: &str, policy: PermissionPolicy) {
        let mut policies = self.tool_policies.write().unwrap();
        policies.insert(tool_name.to_string(), policy);
    }

    /// Get permission policy for a specific tool
    pub fn get_tool_policy(&self, tool_name: &str) -> PermissionPolicy {
        let policies = self.tool_policies.read().unwrap();
        policies.get(tool_name).copied().unwrap_or(self.default_policy)
    }

    /// Check if a tool is allowed to execute
    pub fn check_permission(&self, tool_name: &str) -> Result<()> {
        let policy = self.get_tool_policy(tool_name);

        match policy {
            PermissionPolicy::Allow => Ok(()),
            PermissionPolicy::Deny => Err(Error::Internal(format!(
                "Permission denied: tool '{}' is not allowed",
                tool_name
            ))),
            PermissionPolicy::Prompt => {
                // In Phase 3, Prompt is treated as Allow
                // In future phases, this would trigger a user prompt
                Ok(())
            }
        }
    }

    /// Remove permission policy for a specific tool (reverts to default)
    pub fn remove_tool_policy(&self, tool_name: &str) -> bool {
        let mut policies = self.tool_policies.write().unwrap();
        policies.remove(tool_name).is_some()
    }

    /// List all tools with explicit policies
    pub fn list_tool_policies(&self) -> HashMap<String, PermissionPolicy> {
        let policies = self.tool_policies.read().unwrap();
        policies.clone()
    }

    /// Clear all tool-specific policies
    pub fn clear_tool_policies(&self) {
        let mut policies = self.tool_policies.write().unwrap();
        policies.clear();
    }

    /// Get the default policy
    pub fn default_policy(&self) -> PermissionPolicy {
        self.default_policy
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::prompt_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all() {
        let manager = PermissionManager::allow_all();
        assert_eq!(manager.default_policy(), PermissionPolicy::Allow);
        assert!(manager.check_permission("any_tool").is_ok());
    }

    #[test]
    fn test_deny_all() {
        let manager = PermissionManager::deny_all();
        assert_eq!(manager.default_policy(), PermissionPolicy::Deny);
        assert!(manager.check_permission("any_tool").is_err());
    }

    #[test]
    fn test_prompt_all() {
        let manager = PermissionManager::prompt_all();
        assert_eq!(manager.default_policy(), PermissionPolicy::Prompt);
        // Prompt is treated as Allow in Phase 3
        assert!(manager.check_permission("any_tool").is_ok());
    }

    #[test]
    fn test_set_tool_policy() {
        let manager = PermissionManager::deny_all();

        // Default is deny
        assert!(manager.check_permission("test_tool").is_err());

        // Allow specific tool
        manager.set_tool_policy("test_tool", PermissionPolicy::Allow);
        assert!(manager.check_permission("test_tool").is_ok());

        // Other tools still denied
        assert!(manager.check_permission("other_tool").is_err());
    }

    #[test]
    fn test_get_tool_policy() {
        let manager = PermissionManager::allow_all();

        assert_eq!(manager.get_tool_policy("any_tool"), PermissionPolicy::Allow);

        manager.set_tool_policy("special_tool", PermissionPolicy::Deny);
        assert_eq!(manager.get_tool_policy("special_tool"), PermissionPolicy::Deny);
    }

    #[test]
    fn test_remove_tool_policy() {
        let manager = PermissionManager::allow_all();

        manager.set_tool_policy("test_tool", PermissionPolicy::Deny);
        assert_eq!(manager.get_tool_policy("test_tool"), PermissionPolicy::Deny);

        assert!(manager.remove_tool_policy("test_tool"));
        assert_eq!(manager.get_tool_policy("test_tool"), PermissionPolicy::Allow);

        // Removing non-existent policy returns false
        assert!(!manager.remove_tool_policy("nonexistent"));
    }

    #[test]
    fn test_list_tool_policies() {
        let manager = PermissionManager::allow_all();

        manager.set_tool_policy("tool1", PermissionPolicy::Deny);
        manager.set_tool_policy("tool2", PermissionPolicy::Allow);

        let policies = manager.list_tool_policies();
        assert_eq!(policies.len(), 2);
        assert_eq!(policies.get("tool1"), Some(&PermissionPolicy::Deny));
        assert_eq!(policies.get("tool2"), Some(&PermissionPolicy::Allow));
    }

    #[test]
    fn test_clear_tool_policies() {
        let manager = PermissionManager::allow_all();

        manager.set_tool_policy("tool1", PermissionPolicy::Deny);
        manager.set_tool_policy("tool2", PermissionPolicy::Deny);

        assert_eq!(manager.list_tool_policies().len(), 2);

        manager.clear_tool_policies();
        assert_eq!(manager.list_tool_policies().len(), 0);
    }

    #[test]
    fn test_default_permission_manager() {
        let manager = PermissionManager::default();
        assert_eq!(manager.default_policy(), PermissionPolicy::Prompt);
    }
}
