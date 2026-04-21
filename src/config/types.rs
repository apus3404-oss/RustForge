use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Custom serialization for Duration as seconds
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

/// Main configuration container
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalConfig {
    #[serde(default)]
    pub llm: LLMConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
    #[serde(default)]
    pub permissions: PermissionConfig,
    #[serde(default)]
    pub ui: UIConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            execution: ExecutionConfig::default(),
            permissions: PermissionConfig::default(),
            ui: UIConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl GlobalConfig {
    /// Merge another config into this one, with the other config taking priority
    pub fn merge(&mut self, other: GlobalConfig) {
        // Merge LLM config
        if !other.llm.providers.is_empty() {
            self.llm.providers = other.llm.providers;
        }
        self.llm.default_model = other.llm.default_model;
        self.llm.default_temperature = other.llm.default_temperature;
        self.llm.default_max_tokens = other.llm.default_max_tokens;

        // Merge execution config
        self.execution.max_parallel_agents = other.execution.max_parallel_agents;
        self.execution.default_timeout = other.execution.default_timeout;
        self.execution.retry_attempts = other.execution.retry_attempts;
        self.execution.retry_delay = other.execution.retry_delay;

        // Merge permissions config
        self.permissions.require_approval = other.permissions.require_approval;
        if !other.permissions.allowed_commands.is_empty() {
            self.permissions.allowed_commands = other.permissions.allowed_commands;
        }
        if !other.permissions.blocked_commands.is_empty() {
            self.permissions.blocked_commands = other.permissions.blocked_commands;
        }
        if !other.permissions.policies.is_empty() {
            self.permissions.policies = other.permissions.policies;
        }

        // Merge UI config
        self.ui.enabled = other.ui.enabled;
        self.ui.port = other.ui.port;
        self.ui.host = other.ui.host;

        // Merge logging config
        self.logging.level = other.logging.level;
        self.logging.format = other.logging.format;
        self.logging.log_to_file = other.logging.log_to_file;
        if other.logging.log_file_path.is_some() {
            self.logging.log_file_path = other.logging.log_file_path;
        }
    }
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMConfig {
    #[serde(default = "default_provider")]
    pub default_provider: String,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default = "default_temperature")]
    pub default_temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub default_max_tokens: u32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            providers: vec![
                ProviderConfig {
                    name: "openai".to_string(),
                    api_key_env: "OPENAI_API_KEY".to_string(),
                    base_url: Some("https://api.openai.com/v1".to_string()),
                    models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                },
            ],
            default_model: default_model(),
            default_temperature: default_temperature(),
            default_max_tokens: default_max_tokens(),
        }
    }
}

fn default_provider() -> String {
    "ollama:llama3".to_string()
}

fn default_model() -> String {
    "gpt-4".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

/// Individual provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key_env: String,
    pub base_url: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionConfig {
    #[serde(default = "default_max_parallel_agents")]
    pub max_parallel_agents: usize,
    #[serde(default = "default_timeout", with = "duration_secs")]
    pub default_timeout: Duration,
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
    #[serde(default = "default_retry_delay", with = "duration_secs")]
    pub retry_delay: Duration,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_parallel_agents: default_max_parallel_agents(),
            default_timeout: default_timeout(),
            retry_attempts: default_retry_attempts(),
            retry_delay: default_retry_delay(),
        }
    }
}

fn default_max_parallel_agents() -> usize {
    10
}

fn default_timeout() -> Duration {
    Duration::from_secs(300)
}

fn default_retry_attempts() -> u32 {
    3
}

fn default_retry_delay() -> Duration {
    Duration::from_secs(1)
}

/// Permission configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionConfig {
    #[serde(default = "default_require_approval")]
    pub require_approval: bool,
    #[serde(default)]
    pub allowed_commands: Vec<String>,
    #[serde(default)]
    pub blocked_commands: Vec<String>,
    #[serde(default)]
    pub policies: HashMap<String, PolicyAction>,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            require_approval: default_require_approval(),
            allowed_commands: vec![],
            blocked_commands: vec![],
            policies: HashMap::new(),
        }
    }
}

fn default_require_approval() -> bool {
    true
}

/// Policy action for permission rules
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    Allow,
    Deny,
    Prompt,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UIConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            port: default_port(),
            host: default_host(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingConfig {
    #[serde(default = "default_level")]
    pub level: String,
    #[serde(default = "default_format")]
    pub format: LogFormat,
    #[serde(default = "default_log_to_file")]
    pub log_to_file: bool,
    #[serde(default)]
    pub log_file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            format: default_format(),
            log_to_file: default_log_to_file(),
            log_file_path: None,
        }
    }
}

fn default_level() -> String {
    "info".to_string()
}

fn default_format() -> LogFormat {
    LogFormat::Pretty
}

fn default_log_to_file() -> bool {
    false
}

/// Log output format
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
        assert_eq!(config.execution.max_parallel_agents, deserialized.execution.max_parallel_agents);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LLMConfig::default();
        assert!(!config.providers.is_empty());
        assert_eq!(config.default_model, "gpt-4");
        assert_eq!(config.default_temperature, 0.7);
    }

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.max_parallel_agents, 10);
        assert_eq!(config.default_timeout.as_secs(), 300);
        assert_eq!(config.retry_attempts, 3);
    }

    #[test]
    fn test_ui_config_default() {
        let config = UIConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
        assert!(config.enabled);
    }

    #[test]
    fn test_permission_config_default() {
        let config = PermissionConfig::default();
        assert!(config.require_approval);
        assert!(config.allowed_commands.is_empty());
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Pretty);
        assert!(!config.log_to_file);
    }

    #[test]
    fn test_duration_serialization() {
        let config = ExecutionConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("default_timeout = 300"));

        let deserialized: ExecutionConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.default_timeout, deserialized.default_timeout);
    }

    #[test]
    fn test_policy_action_serialization() {
        let action = PolicyAction::Allow;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"allow\"");

        let deserialized: PolicyAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_log_format_serialization() {
        let format = LogFormat::Json;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"json\"");

        let deserialized: LogFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(format, deserialized);
    }
}
