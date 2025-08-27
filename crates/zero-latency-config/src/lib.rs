/// Configuration management for Zero-Latency
///
/// This crate provides centralized configuration patterns including:
/// - Environment-based configuration with ZL_ prefix
/// - TOML configuration file support
/// - Configuration precedence handling (env > file > defaults)
/// - Test configuration utilities for unique ports and collections
/// - Service-specific config sections
pub mod loader;
pub mod models;
pub mod validation;

// Re-export commonly used types
pub use models::{
    AppConfig, ServerConfig, ClientConfig, TestConfig, GlobalConfig, 
    TestConfigHelper, Config // Legacy compatibility
};
pub use loader::{
    ConfigLoader, ConfigResolver, ConfigError,
    load_config, load_config_from_file, load_config_from_env
};
pub use validation::*;
