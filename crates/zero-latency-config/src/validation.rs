use crate::models::{AppConfig, ServerConfig, ClientConfig, TestConfig, GlobalConfig};
use crate::loader::ConfigError;

/// Configuration validation trait
pub trait ConfigValidator<T> {
    /// Validate the configuration and return errors if any
    fn validate(&self, config: &T) -> Result<(), ConfigError>;
}

/// Default configuration validator
pub struct DefaultValidator;

impl ConfigValidator<AppConfig> for DefaultValidator {
    fn validate(&self, config: &AppConfig) -> Result<(), ConfigError> {
        ServerConfigValidator.validate(&config.server)?;
        ClientConfigValidator.validate(&config.client)?;
        TestConfigValidator.validate(&config.test)?;
        GlobalConfigValidator.validate(&config.app)?;
        Ok(())
    }
}

/// Server configuration validator
pub struct ServerConfigValidator;

impl ConfigValidator<ServerConfig> for ServerConfigValidator {
    fn validate(&self, config: &ServerConfig) -> Result<(), ConfigError> {
        // Validate host
        if config.host.is_empty() {
            return Err(ConfigError::ValidationError(
                "Server host cannot be empty".to_string()
            ));
        }
        
        // Validate port range
        if config.port == 0 || config.port > 65535 {
            return Err(ConfigError::ValidationError(
                format!("Invalid server port: {}. Must be between 1-65535", config.port)
            ));
        }
        
        // Validate collection name
        if config.collection_name.is_empty() {
            return Err(ConfigError::ValidationError(
                "Collection name cannot be empty".to_string()
            ));
        }
        
        // Validate collection name format (alphanumeric, underscore, hyphen)
        if !config.collection_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ConfigError::ValidationError(
                "Collection name can only contain alphanumeric characters, underscores, and hyphens".to_string()
            ));
        }
        
        // Validate timeout
        if config.timeout_ms == 0 {
            return Err(ConfigError::ValidationError(
                "Server timeout must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Client configuration validator
pub struct ClientConfigValidator;

impl ConfigValidator<ClientConfig> for ClientConfigValidator {
    fn validate(&self, config: &ClientConfig) -> Result<(), ConfigError> {
        // Validate timeout
        if config.timeout_ms == 0 {
            return Err(ConfigError::ValidationError(
                "Client timeout must be greater than 0".to_string()
            ));
        }
        
        // Validate connect timeout
        if config.connect_timeout_ms == 0 {
            return Err(ConfigError::ValidationError(
                "Client connect timeout must be greater than 0".to_string()
            ));
        }
        
        // Validate max retries (reasonable upper bound)
        if config.max_retries > 100 {
            return Err(ConfigError::ValidationError(
                "Maximum retries cannot exceed 100".to_string()
            ));
        }
        
        // Validate server URL format if provided
        if let Some(url) = &config.server_url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(ConfigError::ValidationError(
                    "Server URL must start with http:// or https://".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

/// Test configuration validator
pub struct TestConfigValidator;

impl ConfigValidator<TestConfig> for TestConfigValidator {
    fn validate(&self, config: &TestConfig) -> Result<(), ConfigError> {
        // Validate port base range (should be in ephemeral port range)
        if config.port_base < 1024 || config.port_base > 60000 {
            return Err(ConfigError::ValidationError(
                format!("Test port base {} should be between 1024-60000", config.port_base)
            ));
        }
        
        // Validate fixture path
        if config.fixture_path.is_empty() {
            return Err(ConfigError::ValidationError(
                "Test fixture path cannot be empty".to_string()
            ));
        }
        
        // Validate collection prefix
        if config.collection_prefix.is_empty() {
            return Err(ConfigError::ValidationError(
                "Test collection prefix cannot be empty".to_string()
            ));
        }
        
        // Validate collection prefix format
        if !config.collection_prefix.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ConfigError::ValidationError(
                "Test collection prefix can only contain alphanumeric characters, underscores, and hyphens".to_string()
            ));
        }
        
        // Validate timeout
        if config.timeout_ms == 0 {
            return Err(ConfigError::ValidationError(
                "Test timeout must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Global configuration validator
pub struct GlobalConfigValidator;

impl ConfigValidator<GlobalConfig> for GlobalConfigValidator {
    fn validate(&self, config: &GlobalConfig) -> Result<(), ConfigError> {
        // Validate log level
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.log_level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log level: {}. Must be one of: {}", 
                    config.log_level, 
                    valid_log_levels.join(", "))
            ));
        }
        
        // Validate output format
        let valid_formats = ["table", "json", "simple"];
        if !valid_formats.contains(&config.output_format.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid output format: {}. Must be one of: {}", 
                    config.output_format, 
                    valid_formats.join(", "))
            ));
        }
        
        Ok(())
    }
}

/// Convenience function to validate configuration
pub fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    DefaultValidator.validate(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_default_config() {
        let config = AppConfig::default();
        assert!(validate_config(&config).is_ok());
    }
    
    #[test]
    fn test_invalid_server_port() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid server port"));
    }
    
    #[test]
    fn test_invalid_collection_name() {
        let mut config = AppConfig::default();
        config.server.collection_name = "invalid!name".to_string();
        
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("alphanumeric"));
    }
    
    #[test]
    fn test_invalid_log_level() {
        let mut config = AppConfig::default();
        config.app.log_level = "invalid".to_string();
        
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid log level"));
    }
    
    #[test]
    fn test_invalid_server_url() {
        let mut config = AppConfig::default();
        config.client.server_url = Some("invalid-url".to_string());
        
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with http"));
    }
}
