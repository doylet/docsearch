use crate::models::AppConfig;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::path::PathBuf;

/// Configuration loading errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Configuration parsing error: {0}")]
    ParseError(String),

    #[error("Environment variable error: {0}")]
    EnvError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// `ZL_*` environment provider. Splits each key at its first `_` only, so
/// `ZL_SERVER_DOCS_PATH` sets `server.docs_path` and `ZL_VECTOR_QDRANT_URL`
/// sets `vector.qdrant_url`.
fn env_provider(prefix: &str) -> Env {
    Env::prefixed(prefix).map(|key| key.as_str().replacen('_', ".", 1).into())
}

/// Apply settings that come from outside the `ZL_` namespace: an unset
/// `embedding.openai_api_key` falls back to `OPENAI_API_KEY`.
fn apply_env_fallbacks(mut config: AppConfig) -> AppConfig {
    if config
        .embedding
        .openai_api_key
        .as_deref()
        .unwrap_or("")
        .is_empty()
    {
        config.embedding.openai_api_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|key| !key.is_empty());
    }
    config
}

/// Trait for configuration loaders
pub trait ConfigLoader<T> {
    /// Load configuration from the source
    fn load(&self) -> Result<T, ConfigError>;
}

/// Environment variable configuration loader
pub struct EnvConfigLoader {
    prefix: String,
}

impl EnvConfigLoader {
    /// Create a new environment loader with ZL_ prefix
    pub fn new() -> Self {
        Self {
            prefix: "ZL_".to_string(),
        }
    }

    /// Create with custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Default for EnvConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader<AppConfig> for EnvConfigLoader {
    fn load(&self) -> Result<AppConfig, ConfigError> {
        let figment = Figment::from(figment::providers::Serialized::defaults(
            AppConfig::default(),
        ))
        .merge(env_provider(&self.prefix));

        figment
            .extract()
            .map(apply_env_fallbacks)
            .map_err(|e| ConfigError::EnvError(e.to_string()))
    }
}

/// File-based configuration loader
pub struct FileConfigLoader {
    file_path: PathBuf,
}

impl FileConfigLoader {
    /// Create a new file loader with specified path
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }

    /// Create with default configuration file locations
    pub fn with_default_locations() -> Self {
        // Try multiple default locations in order of preference
        let default_paths = [
            "zero-latency.toml",
            "config/zero-latency.toml",
            "./zero-latency.toml",
        ];

        for path in &default_paths {
            if std::path::Path::new(path).exists() {
                return Self::new(path);
            }
        }

        // Try user config directory
        if let Some(config_dir) = dirs::config_dir() {
            let user_config = config_dir.join("zero-latency.toml");
            if user_config.exists() {
                return Self::new(user_config);
            }
        }

        // Fallback to first default path (may not exist)
        Self::new(default_paths[0])
    }
}

impl ConfigLoader<AppConfig> for FileConfigLoader {
    fn load(&self) -> Result<AppConfig, ConfigError> {
        if !self.file_path.exists() {
            return Err(ConfigError::FileNotFound(
                self.file_path.display().to_string(),
            ));
        }

        let figment = Figment::new().merge(Toml::file(&self.file_path));

        figment
            .extract()
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

/// Configuration resolver with precedence handling
pub struct ConfigResolver {
    file_loader: Option<FileConfigLoader>,
    env_loader: EnvConfigLoader,
}

impl ConfigResolver {
    /// Create a new resolver with default settings
    pub fn new() -> Self {
        Self {
            file_loader: Some(FileConfigLoader::with_default_locations()),
            env_loader: EnvConfigLoader::new(),
        }
    }

    /// Create resolver with specific file path
    pub fn with_file(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_loader: Some(FileConfigLoader::new(file_path)),
            env_loader: EnvConfigLoader::new(),
        }
    }

    /// Create resolver without file loading (env + defaults only)
    pub fn env_only() -> Self {
        Self {
            file_loader: None,
            env_loader: EnvConfigLoader::new(),
        }
    }

    /// Load configuration with precedence: env > file > defaults
    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        let mut figment = Figment::from(figment::providers::Serialized::defaults(
            AppConfig::default(),
        ));

        // Layer 1: File configuration (if available)
        // Merged as TOML rather than a full AppConfig, so a file that sets only
        // some sections (e.g. just [vector]) still applies
        if let Some(file_loader) = &self.file_loader {
            if file_loader.file_path.exists() {
                figment = figment.merge(Toml::file(&file_loader.file_path));
            }
            // Don't fail if file doesn't exist, just skip it
        }

        // Layer 2: Environment variables (highest precedence)
        figment = figment.merge(env_provider("ZL_"));

        figment
            .extract()
            .map(apply_env_fallbacks)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Load configuration from a specific file path, with env overrides
    pub fn load_from_file(&self, file_path: impl Into<PathBuf>) -> Result<AppConfig, ConfigError> {
        let file_path = file_path.into();

        let mut figment = Figment::from(figment::providers::Serialized::defaults(
            AppConfig::default(),
        ));

        // Layer 1: File configuration
        if file_path.exists() {
            figment = figment.merge(Toml::file(&file_path));
        }

        // Layer 2: Environment variables
        figment = figment.merge(env_provider("ZL_"));

        figment
            .extract()
            .map(apply_env_fallbacks)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

impl Default for ConfigResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to load configuration with default resolver
pub fn load_config() -> Result<AppConfig, ConfigError> {
    ConfigResolver::new().load()
}

/// Convenience function to load configuration from specific file
pub fn load_config_from_file(file_path: impl Into<PathBuf>) -> Result<AppConfig, ConfigError> {
    ConfigResolver::new().load_from_file(file_path)
}

/// Convenience function to load configuration from environment only
pub fn load_config_from_env() -> Result<AppConfig, ConfigError> {
    ConfigResolver::env_only().load()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::validate_config;
    use figment::Jail;

    /// Load from env and defaults only, inside a Jail with a clean environment
    fn load_env(jail: &mut Jail, vars: &[(&str, &str)]) -> AppConfig {
        jail.clear_env();
        for (key, value) in vars {
            jail.set_env(key, value);
        }
        load_config_from_env().unwrap()
    }

    #[test]
    fn test_default_config_loading() {
        Jail::expect_with(|jail| {
            let config = load_env(jail, &[]);
            assert_eq!(config.server.host, "localhost");
            assert_eq!(config.server.port, 8081);
            assert_eq!(config.vector.backend, "embedded");
            assert_eq!(config.embedding.provider, "local");
            assert_eq!(config.embedding.openai_api_key, None);
            Ok(())
        });
    }

    #[test]
    fn test_env_config_override() {
        Jail::expect_with(|jail| {
            let config = load_env(
                jail,
                &[("ZL_SERVER_HOST", "0.0.0.0"), ("ZL_SERVER_PORT", "9090")],
            );
            assert_eq!(config.server.host, "0.0.0.0");
            assert_eq!(config.server.port, 9090);
            Ok(())
        });
    }

    #[test]
    fn test_env_multi_word_fields() {
        Jail::expect_with(|jail| {
            let config = load_env(
                jail,
                &[
                    ("ZL_SERVER_DOCS_PATH", "/srv/docs"),
                    ("ZL_TEST_PORT_BASE", "20000"),
                    ("ZL_VECTOR_BACKEND", "qdrant"),
                    ("ZL_VECTOR_QDRANT_URL", "http://qdrant:6333"),
                    ("ZL_VECTOR_QDRANT_COLLECTION", "team_docs"),
                    ("ZL_VECTOR_QDRANT_API_KEY", "secret"),
                    ("ZL_EMBEDDING_PROVIDER", "openai"),
                    ("ZL_EMBEDDING_OPENAI_MODEL", "text-embedding-3-large"),
                    ("ZL_EMBEDDING_OPENAI_MAX_RETRIES", "5"),
                ],
            );
            assert_eq!(config.server.docs_path.as_deref(), Some("/srv/docs"));
            assert_eq!(config.test.port_base, 20000);
            assert_eq!(config.vector.backend, "qdrant");
            assert_eq!(config.vector.qdrant_url, "http://qdrant:6333");
            assert_eq!(config.vector.qdrant_collection, "team_docs");
            assert_eq!(config.vector.qdrant_api_key.as_deref(), Some("secret"));
            assert_eq!(config.embedding.provider, "openai");
            assert_eq!(config.embedding.openai_model, "text-embedding-3-large");
            assert_eq!(config.embedding.openai_max_retries, 5);
            Ok(())
        });
    }

    #[test]
    fn test_openai_key_fallback() {
        Jail::expect_with(|jail| {
            let config = load_env(jail, &[("OPENAI_API_KEY", "sk-from-env")]);
            assert_eq!(
                config.embedding.openai_api_key.as_deref(),
                Some("sk-from-env")
            );

            // An explicit ZL_ key wins over OPENAI_API_KEY
            let config = load_env(
                jail,
                &[
                    ("OPENAI_API_KEY", "sk-from-env"),
                    ("ZL_EMBEDDING_OPENAI_API_KEY", "sk-explicit"),
                ],
            );
            assert_eq!(
                config.embedding.openai_api_key.as_deref(),
                Some("sk-explicit")
            );
            Ok(())
        });
    }

    #[test]
    fn test_partial_file_with_env_override() {
        Jail::expect_with(|jail| {
            jail.clear_env();
            jail.create_file(
                "zero-latency.toml",
                r#"
                [vector]
                backend = "qdrant"
                qdrant_collection = "from_file"
                "#,
            )?;
            jail.set_env("ZL_VECTOR_QDRANT_COLLECTION", "from_env");

            // Default file location (the Jail's working directory)
            let config = load_config().unwrap();
            assert_eq!(config.vector.backend, "qdrant");
            assert_eq!(config.vector.qdrant_collection, "from_env");
            assert_eq!(config.vector.qdrant_url, "http://localhost:6333");
            assert_eq!(config.server.port, 8081);

            // Explicit file path
            let config = load_config_from_file("zero-latency.toml").unwrap();
            assert_eq!(config.vector.backend, "qdrant");
            assert_eq!(config.vector.qdrant_collection, "from_env");
            Ok(())
        });
    }

    #[test]
    fn test_invalid_selection_fails_validation() {
        Jail::expect_with(|jail| {
            let config = load_env(jail, &[("ZL_VECTOR_BACKEND", "pinecone")]);
            let error = validate_config(&config).unwrap_err().to_string();
            assert!(error.contains("pinecone"), "{}", error);

            let config = load_env(jail, &[("ZL_EMBEDDING_PROVIDER", "openai")]);
            let error = validate_config(&config).unwrap_err().to_string();
            assert!(error.contains("API key"), "{}", error);

            let config = load_env(
                jail,
                &[
                    ("ZL_EMBEDDING_PROVIDER", "OpenAI"),
                    ("OPENAI_API_KEY", "sk-test"),
                ],
            );
            assert!(validate_config(&config).is_ok());
            Ok(())
        });
    }

    #[test]
    fn test_test_config_helper() {
        use crate::models::TestConfigHelper;

        let helper = TestConfigHelper::new();

        let port1 = helper.get_unique_port();
        let port2 = helper.get_unique_port();
        assert_ne!(port1, port2);

        let collection1 = helper.get_unique_collection_name();
        let collection2 = helper.get_unique_collection_name();
        assert_ne!(collection1, collection2);
        assert!(collection1.starts_with("test_collection_"));
    }
}
