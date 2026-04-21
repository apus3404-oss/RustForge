use crate::config::types::GlobalConfig;
use crate::error::{RustForgeError, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration with priority: env vars > project config > user config > defaults
    pub fn load() -> Result<GlobalConfig> {
        let mut config = Self::load_default();

        // Load user config if it exists
        if let Some(user_path) = Self::user_config_path() {
            if user_path.exists() {
                let user_config = Self::load_from_file(&user_path)?;
                config.merge(user_config);
            }
        }

        // Load project config if it exists (overrides user config)
        let project_path = Self::project_config_path();
        if project_path.exists() {
            let project_config = Self::load_from_file(&project_path)?;
            config.merge(project_config);
        }

        // Apply environment variable overrides (highest priority)
        Self::apply_env_overrides(&mut config);

        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<GlobalConfig> {
        let content = fs::read_to_string(path).map_err(|e| {
            RustForgeError::ConfigError(format!("Failed to read config file {:?}: {}", path, e))
        })?;

        let config: GlobalConfig = toml::from_str(&content).map_err(|e| {
            RustForgeError::ConfigError(format!("Failed to parse config file {:?}: {}", path, e))
        })?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file(config: &GlobalConfig, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                RustForgeError::ConfigError(format!(
                    "Failed to create config directory {:?}: {}",
                    parent, e
                ))
            })?;
        }

        let content = toml::to_string_pretty(config).map_err(|e| {
            RustForgeError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(path, content).map_err(|e| {
            RustForgeError::ConfigError(format!("Failed to write config file {:?}: {}", path, e))
        })?;

        Ok(())
    }

    /// Load default configuration
    pub fn load_default() -> GlobalConfig {
        GlobalConfig::default()
    }

    /// Get user config path (~/.rustforge/config.toml)
    fn user_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".rustforge").join("config.toml"))
    }

    /// Get project config path (.rustforge/config.toml in current directory)
    fn project_config_path() -> PathBuf {
        PathBuf::from(".rustforge").join("config.toml")
    }

    /// Apply environment variable overrides to config
    fn apply_env_overrides(config: &mut GlobalConfig) {
        // Execution overrides
        if let Ok(val) = env::var("RUSTFORGE_MAX_PARALLEL_AGENTS") {
            if let Ok(num) = val.parse::<usize>() {
                config.execution.max_parallel_agents = num;
            }
        }

        if let Ok(val) = env::var("RUSTFORGE_DEFAULT_TIMEOUT") {
            if let Ok(secs) = val.parse::<u64>() {
                config.execution.default_timeout = std::time::Duration::from_secs(secs);
            }
        }

        // UI overrides
        if let Ok(val) = env::var("RUSTFORGE_UI_PORT") {
            if let Ok(port) = val.parse::<u16>() {
                config.ui.port = port;
            }
        }

        if let Ok(val) = env::var("RUSTFORGE_UI_HOST") {
            config.ui.host = val;
        }

        if let Ok(val) = env::var("RUSTFORGE_UI_ENABLED") {
            if let Ok(enabled) = val.parse::<bool>() {
                config.ui.enabled = enabled;
            }
        }

        // Logging overrides
        if let Ok(val) = env::var("RUSTFORGE_LOG_LEVEL") {
            config.logging.level = val;
        }

        if let Ok(val) = env::var("RUSTFORGE_LOG_FORMAT") {
            match val.to_lowercase().as_str() {
                "json" => config.logging.format = crate::config::types::LogFormat::Json,
                "pretty" => config.logging.format = crate::config::types::LogFormat::Pretty,
                "compact" => config.logging.format = crate::config::types::LogFormat::Compact,
                _ => {}
            }
        }

        // LLM overrides
        if let Ok(val) = env::var("RUSTFORGE_DEFAULT_MODEL") {
            config.llm.default_model = val;
        }

        if let Ok(val) = env::var("RUSTFORGE_DEFAULT_TEMPERATURE") {
            if let Ok(temp) = val.parse::<f32>() {
                config.llm.default_temperature = temp;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_load_default_config() {
        let config = ConfigLoader::load_default();
        assert_eq!(config.execution.max_parallel_agents, 10);
        assert_eq!(config.execution.default_timeout.as_secs(), 300);
        assert_eq!(config.ui.port, 3000);
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
[llm]
default_model = "gpt-4"
default_temperature = 0.8
default_max_tokens = 4096

[execution]
max_parallel_agents = 20
default_timeout = 600
retry_attempts = 5
retry_delay = 2

[permissions]
require_approval = false

[ui]
enabled = true
port = 8080
host = "0.0.0.0"

[logging]
level = "debug"
format = "json"
log_to_file = true
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = ConfigLoader::load_from_file(&config_path).unwrap();
        assert_eq!(config.execution.max_parallel_agents, 20);
        assert_eq!(config.execution.default_timeout.as_secs(), 600);
        assert_eq!(config.execution.retry_attempts, 5);
        assert_eq!(config.ui.port, 8080);
        assert_eq!(config.ui.host, "0.0.0.0");
        assert_eq!(config.logging.level, "debug");
        assert!(!config.permissions.require_approval);
    }

    #[test]
    fn test_load_from_file_partial() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
[execution]
max_parallel_agents = 15
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = ConfigLoader::load_from_file(&config_path).unwrap();
        assert_eq!(config.execution.max_parallel_agents, 15);
        // Other values should be defaults
        assert_eq!(config.execution.default_timeout.as_secs(), 300);
        assert_eq!(config.ui.port, 3000);
    }

    #[test]
    fn test_load_from_file_not_found() {
        let result = ConfigLoader::load_from_file(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = GlobalConfig::default();
        config.execution.max_parallel_agents = 25;
        config.ui.port = 9000;

        ConfigLoader::save_to_file(&config, &config_path).unwrap();

        // Verify file was created and can be loaded back
        assert!(config_path.exists());
        let loaded = ConfigLoader::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.execution.max_parallel_agents, 25);
        assert_eq!(loaded.ui.port, 9000);
    }

    #[test]
    fn test_merge_configs() {
        let mut base = GlobalConfig::default();
        let mut override_config = GlobalConfig::default();

        override_config.execution.max_parallel_agents = 20;
        override_config.ui.port = 8080;

        base.merge(override_config);

        assert_eq!(base.execution.max_parallel_agents, 20);
        assert_eq!(base.ui.port, 8080);
        // Other values should remain from base
        assert_eq!(base.execution.default_timeout.as_secs(), 300);
    }

    #[test]
    fn test_env_overrides() {
        env::set_var("RUSTFORGE_MAX_PARALLEL_AGENTS", "30");
        env::set_var("RUSTFORGE_UI_PORT", "7000");
        env::set_var("RUSTFORGE_LOG_LEVEL", "trace");

        let mut config = GlobalConfig::default();
        ConfigLoader::apply_env_overrides(&mut config);

        assert_eq!(config.execution.max_parallel_agents, 30);
        assert_eq!(config.ui.port, 7000);
        assert_eq!(config.logging.level, "trace");

        // Cleanup
        env::remove_var("RUSTFORGE_MAX_PARALLEL_AGENTS");
        env::remove_var("RUSTFORGE_UI_PORT");
        env::remove_var("RUSTFORGE_LOG_LEVEL");
    }
}
