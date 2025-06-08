use serde::{Deserialize, Serialize};
use std::env;

/// Application configuration that can be loaded from YAML and overridden by environment variables
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub persistence: PersistenceSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PersistenceSettings {
    /// Enable/disable database persistence
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Auto-save interval in seconds
    #[serde(default = "default_auto_save_interval")]
    pub auto_save_interval: u64,

    /// Shutdown timeout in seconds
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout: u64,

    /// Enable write coalescing to batch database operations
    #[serde(default = "default_true")]
    pub write_coalescing: bool,

    /// Write coalescing delay in milliseconds
    #[serde(default = "default_coalescing_delay")]
    pub coalescing_delay_ms: u64,
}

fn default_true() -> bool {
    true
}
fn default_auto_save_interval() -> u64 {
    30
}
fn default_shutdown_timeout() -> u64 {
    10
}
fn default_coalescing_delay() -> u64 {
    1000
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            persistence: PersistenceSettings::default(),
        }
    }
}

impl Default for PersistenceSettings {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            auto_save_interval: default_auto_save_interval(),
            shutdown_timeout: default_shutdown_timeout(),
            write_coalescing: default_true(),
            coalescing_delay_ms: default_coalescing_delay(),
        }
    }
}

impl AppConfig {
    /// Load configuration from YAML file and override with environment variables
    pub fn load_from_file_and_env(
        file_path: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = if let Some(path) = file_path {
            // Try to load from YAML file
            match std::fs::read_to_string(path) {
                Ok(content) => serde_yaml::from_str::<AppConfig>(&content)?,
                Err(_) => {
                    log::warn!("Could not read config file {}, using defaults", path);
                    AppConfig::default()
                }
            }
        } else {
            AppConfig::default()
        };

        // Override with environment variables using GWAR prefix
        config.apply_env_overrides();

        Ok(config)
    }

    /// Load configuration only from environment variables
    pub fn load_from_env() -> Self {
        let mut config = AppConfig::default();
        config.apply_env_overrides();
        config
    }

    /// Apply environment variable overrides with GWAR prefix
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = env::var("GWAR_PERSISTENCE_ENABLED") {
            if let Ok(enabled) = val.parse::<bool>() {
                self.persistence.enabled = enabled;
            }
        }

        if let Ok(val) = env::var("GWAR_PERSISTENCE_AUTO_SAVE_INTERVAL") {
            if let Ok(interval) = val.parse::<u64>() {
                self.persistence.auto_save_interval = interval;
            }
        }

        if let Ok(val) = env::var("GWAR_PERSISTENCE_SHUTDOWN_TIMEOUT") {
            if let Ok(timeout) = val.parse::<u64>() {
                self.persistence.shutdown_timeout = timeout;
            }
        }

        if let Ok(val) = env::var("GWAR_PERSISTENCE_WRITE_COALESCING") {
            if let Ok(coalescing) = val.parse::<bool>() {
                self.persistence.write_coalescing = coalescing;
            }
        }

        if let Ok(val) = env::var("GWAR_PERSISTENCE_COALESCING_DELAY_MS") {
            if let Ok(delay) = val.parse::<u64>() {
                self.persistence.coalescing_delay_ms = delay;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.persistence.enabled);
        assert_eq!(config.persistence.auto_save_interval, 30);
        assert_eq!(config.persistence.shutdown_timeout, 10);
        assert!(config.persistence.write_coalescing);
        assert_eq!(config.persistence.coalescing_delay_ms, 1000);
    }

    #[test]
    fn test_env_overrides() {
        // Set test environment variables
        env::set_var("GWAR_PERSISTENCE_ENABLED", "false");
        env::set_var("GWAR_PERSISTENCE_AUTO_SAVE_INTERVAL", "60");
        env::set_var("GWAR_PERSISTENCE_SHUTDOWN_TIMEOUT", "20");
        env::set_var("GWAR_PERSISTENCE_WRITE_COALESCING", "false");
        env::set_var("GWAR_PERSISTENCE_COALESCING_DELAY_MS", "2000");

        let config = AppConfig::load_from_env();

        assert!(!config.persistence.enabled);
        assert_eq!(config.persistence.auto_save_interval, 60);
        assert_eq!(config.persistence.shutdown_timeout, 20);
        assert!(!config.persistence.write_coalescing);
        assert_eq!(config.persistence.coalescing_delay_ms, 2000);

        // Clean up
        env::remove_var("GWAR_PERSISTENCE_ENABLED");
        env::remove_var("GWAR_PERSISTENCE_AUTO_SAVE_INTERVAL");
        env::remove_var("GWAR_PERSISTENCE_SHUTDOWN_TIMEOUT");
        env::remove_var("GWAR_PERSISTENCE_WRITE_COALESCING");
        env::remove_var("GWAR_PERSISTENCE_COALESCING_DELAY_MS");
    }

    #[test]
    fn test_yaml_config() {
        let yaml_content = r#"
persistence:
  enabled: false
  auto_save_interval: 45
  shutdown_timeout: 15
  write_coalescing: false
  coalescing_delay_ms: 500
"#;

        let config: AppConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert!(!config.persistence.enabled);
        assert_eq!(config.persistence.auto_save_interval, 45);
        assert_eq!(config.persistence.shutdown_timeout, 15);
        assert!(!config.persistence.write_coalescing);
        assert_eq!(config.persistence.coalescing_delay_ms, 500);
    }
}
