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
pub use loader::{
    load_config, load_config_from_env, load_config_from_file, ConfigError, ConfigLoader,
    ConfigResolver,
};
pub use models::{
    AppConfig,
    ClientConfig,
    Config, // Legacy compatibility
    EmbeddingSettings,
    GlobalConfig,
    ServerConfig,
    TestConfig,
    TestConfigHelper,
    VectorSettings,
};
pub use validation::*;
