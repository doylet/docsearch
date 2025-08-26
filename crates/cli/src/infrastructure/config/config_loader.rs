use std::fs;
use std::path::PathBuf;

use zero_latency_config::Config;
use zero_latency_core::{Result as ZeroLatencyResult, ZeroLatencyError};

/// File-based configuration loader for CLI configuration.
///
/// This loader handles loading and saving configuration files,
/// providing a concrete implementation for configuration management.
pub struct FileConfigLoader;

impl FileConfigLoader {
    /// Creates a new file configuration loader
    pub fn new() -> Self {
        Self
    }

    /// Load configuration from file
    #[allow(dead_code)]
    pub fn load_config(&self, config_path: Option<PathBuf>) -> ZeroLatencyResult<Config> {
        let config_path = config_path.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("zero-latency");
            path.push("config.toml");
            path
        });

        if config_path.exists() {
            let content =
                fs::read_to_string(&config_path).map_err(|e| ZeroLatencyError::Configuration {
                    message: format!("Failed to read config file: {}", e),
                })?;

            toml::from_str(&content).map_err(|e| ZeroLatencyError::Configuration {
                message: format!("Failed to parse config file: {}", e),
            })
        } else {
            // Return default configuration if file doesn't exist
            Ok(Config::default())
        }
    }

    /// Save configuration to file
    #[allow(dead_code)]
    pub fn save_config(&self, config: &Config, config_path: PathBuf) -> ZeroLatencyResult<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ZeroLatencyError::Configuration {
                message: format!("Failed to create config directory: {}", e),
            })?;
        }

        let content =
            toml::to_string_pretty(config).map_err(|e| ZeroLatencyError::Configuration {
                message: format!("Failed to serialize config: {}", e),
            })?;

        fs::write(&config_path, content).map_err(|e| ZeroLatencyError::Configuration {
            message: format!("Failed to write config file: {}", e),
        })?;

        Ok(())
    }
}
