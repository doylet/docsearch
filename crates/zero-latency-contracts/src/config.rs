/// Configuration Contracts
/// 
/// Shared configuration constants and utilities to ensure consistency
/// between CLI and server configurations.

use serde::{Deserialize, Serialize};
use crate::api::{defaults, urls};

/// Standard configuration values shared across the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardConfig {
    pub server_port: u16,
    pub server_host: String,
    pub collection_name: String,
    pub request_timeout_ms: u64,
}

impl Default for StandardConfig {
    fn default() -> Self {
        Self {
            server_port: defaults::SERVER_PORT,
            server_host: defaults::SERVER_HOST.to_string(),
            collection_name: defaults::COLLECTION_NAME.to_string(),
            request_timeout_ms: defaults::REQUEST_TIMEOUT_MS,
        }
    }
}

impl StandardConfig {
    /// Generate server URL from configuration
    pub fn server_url(&self) -> String {
        urls::server_url(&self.server_host, self.server_port)
    }
    
    /// Create configuration with custom port
    pub fn with_port(port: u16) -> Self {
        Self {
            server_port: port,
            ..Default::default()
        }
    }
    
    /// Create configuration with custom host
    pub fn with_host(host: &str) -> Self {
        Self {
            server_host: host.to_string(),
            ..Default::default()
        }
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.server_port == 0 {
            return Err(ConfigValidationError::InvalidPort("Port cannot be 0".to_string()));
        }
        
        if self.server_host.is_empty() {
            return Err(ConfigValidationError::InvalidHost("Host cannot be empty".to_string()));
        }
        
        if self.collection_name.is_empty() {
            return Err(ConfigValidationError::InvalidCollection("Collection name cannot be empty".to_string()));
        }
        
        if self.request_timeout_ms == 0 {
            return Err(ConfigValidationError::InvalidTimeout("Timeout cannot be 0".to_string()));
        }
        
        Ok(())
    }
}

/// Configuration validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValidationError {
    InvalidPort(String),
    InvalidHost(String),
    InvalidCollection(String),
    InvalidTimeout(String),
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPort(msg) => write!(f, "Invalid port: {}", msg),
            Self::InvalidHost(msg) => write!(f, "Invalid host: {}", msg),
            Self::InvalidCollection(msg) => write!(f, "Invalid collection: {}", msg),
            Self::InvalidTimeout(msg) => write!(f, "Invalid timeout: {}", msg),
        }
    }
}

impl std::error::Error for ConfigValidationError {}

/// Helper functions for consistent configuration
pub mod helpers {
    use super::*;
    
    /// Create CLI configuration using standard defaults
    pub fn default_cli_config() -> StandardConfig {
        StandardConfig::default()
    }
    
    /// Create server configuration using standard defaults  
    pub fn default_server_config() -> StandardConfig {
        StandardConfig::default()
    }
    
    /// Ensure CLI and server use compatible configurations
    pub fn validate_compatibility(cli_config: &StandardConfig, server_config: &StandardConfig) -> Result<(), String> {
        if cli_config.server_port != server_config.server_port {
            return Err(format!(
                "Port mismatch: CLI expects {}, Server uses {}",
                cli_config.server_port,
                server_config.server_port
            ));
        }
        
        if cli_config.collection_name != server_config.collection_name {
            return Err(format!(
                "Collection name mismatch: CLI expects '{}', Server uses '{}'",
                cli_config.collection_name,
                server_config.collection_name
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = StandardConfig::default();
        assert_eq!(config.server_port, 8081);
        assert_eq!(config.server_host, "localhost");
        assert_eq!(config.collection_name, "zero_latency_docs");
        assert_eq!(config.request_timeout_ms, 30000);
    }
    
    #[test]
    fn test_config_validation() {
        let valid_config = StandardConfig::default();
        assert!(valid_config.validate().is_ok());
        
        let invalid_port = StandardConfig {
            server_port: 0,
            ..Default::default()
        };
        assert!(invalid_port.validate().is_err());
        
        let invalid_host = StandardConfig {
            server_host: "".to_string(),
            ..Default::default()
        };
        assert!(invalid_host.validate().is_err());
    }
    
    #[test]
    fn test_server_url_generation() {
        let config = StandardConfig::default();
        assert_eq!(config.server_url(), "http://localhost:8081");
        
        let custom_config = StandardConfig::with_port(9000);
        assert_eq!(custom_config.server_url(), "http://localhost:9000");
    }
    
    #[test]
    fn test_compatibility_validation() {
        let cli_config = StandardConfig::default();
        let server_config = StandardConfig::default();
        
        assert!(helpers::validate_compatibility(&cli_config, &server_config).is_ok());
        
        let mismatched_server = StandardConfig::with_port(9000);
        assert!(helpers::validate_compatibility(&cli_config, &mismatched_server).is_err());
    }
}
