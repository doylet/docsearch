use crate::infrastructure::ServerConfig;
/// Configuration management for doc-indexer service
///
/// This module handles loading and validating configuration from various sources
/// including environment variables, configuration files, and command line arguments.
use serde::{Deserialize, Serialize};
use zero_latency_config::AppConfig;
use zero_latency_core::{Result, ZeroLatencyError};

// For simpler compilation, we'll include all config types but make implementations conditional
// This allows configuration to be loaded regardless of features, but actual usage is gated

// Define placeholder types when features are not enabled
#[cfg(not(feature = "cloud"))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
}

#[cfg(not(feature = "cloud"))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

#[cfg(not(feature = "embedded"))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddedConfig {
    pub db_path: std::path::PathBuf,
    pub dimension: usize,
    pub cache_size: usize,
    pub enable_string_interning: bool,
    pub enable_smart_caching: bool,
}

// Import actual types when features are enabled
#[cfg(feature = "cloud")]
use crate::infrastructure::{OpenAIConfig, QdrantConfig};

#[cfg(feature = "embedded")]
use crate::infrastructure::EmbeddedConfig;

/// Settings for the `local` (bge-small-en-v1.5) and `hash` embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEmbeddingConfig {
    /// Fixed at 384 for `local`; free for `hash`
    pub dimension: usize,
    /// Seed for the `hash` provider
    pub seed: u64,
    /// Vector pooling for the `hash` provider
    pub enable_vector_pooling: bool,
    /// Directory holding pre-supplied model files. Never downloaded into
    #[serde(default)]
    pub model_path: Option<std::path::PathBuf>,
    /// Override for the model download base URL
    #[serde(default)]
    pub model_url: Option<String>,
}

impl Default for LocalEmbeddingConfig {
    fn default() -> Self {
        Self {
            dimension: LOCAL_EMBEDDING_DIMENSION,
            seed: 42,
            enable_vector_pooling: true,
            model_path: None,
            model_url: None,
        }
    }
}

/// The local model (bge-small-en-v1.5) produces 384-dimensional vectors
pub const LOCAL_EMBEDDING_DIMENSION: usize = 384;

/// Load testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestingConfig {
    pub enabled: bool,
    pub concurrency: usize,
    pub duration_seconds: u64,
    pub rate_limit: Option<u64>,
    pub validate_optimizations: bool,
    pub detailed_metrics: bool,
}

impl Default for LoadTestingConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("LOAD_TESTING_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            concurrency: std::env::var("LOAD_TEST_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            duration_seconds: std::env::var("LOAD_TEST_DURATION_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            rate_limit: std::env::var("LOAD_TEST_RATE_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok()),
            validate_optimizations: std::env::var("LOAD_TEST_VALIDATE_OPTIMIZATIONS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            detailed_metrics: std::env::var("LOAD_TEST_DETAILED_METRICS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}

/// Production deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    /// Health check configuration
    pub health_check_enabled: bool,
    pub health_check_interval_seconds: u64,
    pub health_check_timeout_seconds: u64,

    /// Monitoring configuration
    pub monitoring_enabled: bool,
    pub metrics_collection_interval_seconds: u64,
    pub performance_alerts_enabled: bool,

    /// Shutdown configuration
    pub graceful_shutdown_timeout_seconds: u64,
    pub shutdown_signal_handlers: bool,

    /// Startup validation configuration
    pub startup_validation_enabled: bool,
    pub startup_timeout_seconds: u64,
    pub dependency_check_enabled: bool,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            health_check_enabled: std::env::var("HEALTH_CHECK_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            health_check_interval_seconds: std::env::var("HEALTH_CHECK_INTERVAL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            health_check_timeout_seconds: std::env::var("HEALTH_CHECK_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            monitoring_enabled: std::env::var("MONITORING_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            metrics_collection_interval_seconds: std::env::var(
                "METRICS_COLLECTION_INTERVAL_SECONDS",
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
            performance_alerts_enabled: std::env::var("PERFORMANCE_ALERTS_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            graceful_shutdown_timeout_seconds: std::env::var("GRACEFUL_SHUTDOWN_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            shutdown_signal_handlers: std::env::var("SHUTDOWN_SIGNAL_HANDLERS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            startup_validation_enabled: std::env::var("STARTUP_VALIDATION_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            startup_timeout_seconds: std::env::var("STARTUP_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            dependency_check_enabled: std::env::var("DEPENDENCY_CHECK_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        }
    }
}

/// Main configuration structure for the doc-indexer service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// HTTP server configuration
    pub server: ServerConfig,

    /// Vector storage configuration
    pub vector: VectorConfig,

    /// Embedding generation configuration
    pub embedding: EmbeddingConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Service-specific settings
    pub service: ServiceConfig,

    /// Load testing configuration
    pub load_testing: LoadTestingConfig,

    /// Production deployment configuration
    pub production: ProductionConfig,
}

/// Vector storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    /// Vector storage backend type
    pub backend: VectorBackend,

    /// Qdrant-specific configuration
    pub qdrant: QdrantConfig,

    /// Embedded storage configuration
    pub embedded: EmbeddedConfig,
}

/// Embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding provider type
    pub provider: EmbeddingProvider,

    /// OpenAI-specific configuration
    pub openai: OpenAIConfig,

    /// Local embedding configuration
    pub local: LocalEmbeddingConfig,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log format (json, pretty)
    pub format: String,

    /// Enable structured logging
    pub structured: bool,
}

/// Service-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Maximum document size in bytes
    pub max_document_size: usize,

    /// Default search result limit
    pub default_search_limit: usize,

    /// Maximum search result limit
    pub max_search_limit: usize,

    /// Default collection name for searches when not specified
    pub default_collection: String,

    /// Enable query enhancement for advanced search
    pub enable_query_enhancement: bool,

    /// Enable result ranking for improved relevance
    pub enable_result_ranking: bool,

    /// Document chunking strategy
    pub chunking_strategy: ChunkingStrategy,

    /// Chunk size in characters
    pub chunk_size: usize,

    /// Chunk overlap in characters
    pub chunk_overlap: usize,

    /// Path to documentation directory to index
    pub docs_path: std::path::PathBuf,

    /// Maximum size in bytes of a binary document (e.g. PDF) to extract text from
    #[serde(default = "default_max_binary_file_size")]
    pub max_binary_file_size: u64,

    /// Directories `/api/browse` may list. Empty disables browsing.
    #[serde(default)]
    pub browse_roots: Vec<std::path::PathBuf>,

    /// Maximum number of items returned by one `/api/browse` response
    #[serde(default = "default_browse_max_entries")]
    pub browse_max_entries: usize,
}

const DEFAULT_MAX_BINARY_FILE_SIZE: u64 = 50 * 1024 * 1024;
const DEFAULT_BROWSE_MAX_ENTRIES: usize = 1000;

fn default_max_binary_file_size() -> u64 {
    std::env::var("DOC_INDEXER_MAX_BINARY_FILE_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_BINARY_FILE_SIZE)
}

fn default_browse_max_entries() -> usize {
    std::env::var("DOC_INDEXER_BROWSE_MAX_ENTRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_BROWSE_MAX_ENTRIES)
}

/// Read `DOC_INDEXER_BROWSE_ROOTS` (`:`-separated). Unset means browsing is disabled.
fn default_browse_roots() -> Vec<std::path::PathBuf> {
    std::env::var("DOC_INDEXER_BROWSE_ROOTS")
        .map(|v| parse_browse_roots(&v))
        .unwrap_or_default()
}

fn parse_browse_roots(value: &str) -> Vec<std::path::PathBuf> {
    value
        .split(':')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .collect()
}

/// Vector storage backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorBackend {
    Memory,
    Qdrant,
    Embedded,
}

/// Embedding provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    /// bge-small-en-v1.5 through ONNX Runtime
    Local,
    OpenAI,
    /// Deterministic, non-semantic vectors. Testing only
    Hash,
}

/// Document chunking strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingStrategy {
    /// Split by sentences
    Sentence,
    /// Split by paragraphs
    Paragraph,
    /// Split by fixed character count
    FixedSize,
    /// Split by semantic boundaries
    Semantic,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ZeroLatencyError::configuration(format!("Failed to read config file: {}", e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            ZeroLatencyError::configuration(format!("Failed to parse config file: {}", e))
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(ZeroLatencyError::configuration(
                "Server port must be greater than 0",
            ));
        }

        // Validate embedding configuration
        match self.embedding.provider {
            EmbeddingProvider::OpenAI => {
                if self.embedding.openai.api_key.is_empty() {
                    return Err(ZeroLatencyError::configuration(
                        "OpenAI API key is required",
                    ));
                }
            }
            EmbeddingProvider::Local => {
                if self.embedding.local.dimension != LOCAL_EMBEDDING_DIMENSION {
                    return Err(ZeroLatencyError::configuration(format!(
                        "Local embedding dimension is fixed at {} by the bge-small-en-v1.5 model, got {}",
                        LOCAL_EMBEDDING_DIMENSION, self.embedding.local.dimension
                    )));
                }
            }
            EmbeddingProvider::Hash => {
                if self.embedding.local.dimension == 0 {
                    return Err(ZeroLatencyError::configuration(
                        "Hash embedding dimension must be greater than 0",
                    ));
                }
            }
        }

        // Validate service configuration
        if self.service.max_document_size == 0 {
            return Err(ZeroLatencyError::configuration(
                "Max document size must be greater than 0",
            ));
        }

        if self.service.chunk_size == 0 {
            return Err(ZeroLatencyError::configuration(
                "Chunk size must be greater than 0",
            ));
        }

        Ok(())
    }

    /// Get configuration as environment variable examples
    pub fn env_example() -> String {
        r#"# Doc-Indexer Configuration
# Environment variables override zero-latency.toml (or --config <file>),
# which overrides the defaults. Each ZL_<SECTION>_<FIELD> sets <section>.<field>.

# Server
ZL_SERVER_HOST=localhost
ZL_SERVER_PORT=8081
ZL_SERVER_DOCS_PATH=~/Documents
ZL_SERVER_COLLECTION_NAME=zero_latency_docs
ZL_SERVER_TIMEOUT_MS=30000

# Vector storage: embedded (default), memory, or qdrant (needs the `cloud` feature)
ZL_VECTOR_BACKEND=embedded
ZL_VECTOR_EMBEDDED_DB_PATH=~/.zero-latency/vectors.db
ZL_VECTOR_EMBEDDED_DIMENSION=384
ZL_VECTOR_EMBEDDED_CACHE_SIZE=10000
ZL_VECTOR_QDRANT_URL=http://localhost:6333
ZL_VECTOR_QDRANT_COLLECTION=zero_latency_docs
ZL_VECTOR_QDRANT_API_KEY=your-qdrant-api-key
ZL_VECTOR_QDRANT_TIMEOUT_SECONDS=30

# Embeddings: local (default, bge-small-en-v1.5), openai (needs the `cloud`
# feature), or hash (testing only: deterministic, not semantic)
ZL_EMBEDDING_PROVIDER=local
# Fixed at 384 for the local model
ZL_EMBEDDING_LOCAL_DIMENSION=384
# Pre-supplied model.onnx + tokenizer.json; nothing is downloaded when set.
# Unset: downloaded on first run (~130 MB) to ~/.zero-latency/models/bge-small-en-v1.5
ZL_EMBEDDING_LOCAL_MODEL_PATH=/app/models/bge-small-en-v1.5
# Download source (mirror) for the model files
ZL_EMBEDDING_LOCAL_MODEL_URL=https://huggingface.co/BAAI/bge-small-en-v1.5
# OPENAI_API_KEY is used when ZL_EMBEDDING_OPENAI_API_KEY is unset
OPENAI_API_KEY=your-openai-api-key
ZL_EMBEDDING_OPENAI_MODEL=text-embedding-3-small
ZL_EMBEDDING_OPENAI_BASE_URL=https://api.openai.com/v1
ZL_EMBEDDING_OPENAI_TIMEOUT_SECONDS=30
ZL_EMBEDDING_OPENAI_MAX_RETRIES=3

# Logging
ZL_APP_LOG_LEVEL=info
"#
        .to_string()
    }
}

// String parsing implementations for enums
impl std::str::FromStr for VectorBackend {
    type Err = ZeroLatencyError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "memory" => Ok(VectorBackend::Memory),
            "qdrant" => Ok(VectorBackend::Qdrant),
            "embedded" => Ok(VectorBackend::Embedded),
            _ => Err(ZeroLatencyError::configuration(format!(
                "Unknown vector backend: {}",
                s
            ))),
        }
    }
}

impl std::str::FromStr for EmbeddingProvider {
    type Err = ZeroLatencyError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(EmbeddingProvider::Local),
            "openai" => Ok(EmbeddingProvider::OpenAI),
            "hash" => Ok(EmbeddingProvider::Hash),
            _ => Err(ZeroLatencyError::configuration(format!(
                "Unknown embedding provider: {}",
                s
            ))),
        }
    }
}

impl std::str::FromStr for ChunkingStrategy {
    type Err = ZeroLatencyError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sentence" => Ok(ChunkingStrategy::Sentence),
            "paragraph" => Ok(ChunkingStrategy::Paragraph),
            "fixed_size" => Ok(ChunkingStrategy::FixedSize),
            "semantic" => Ok(ChunkingStrategy::Semantic),
            _ => Err(ZeroLatencyError::configuration(format!(
                "Unknown chunking strategy: {}",
                s
            ))),
        }
    }
}

impl Config {
    /// Create Config from the centralized AppConfig
    pub fn from_app_config(app_config: AppConfig) -> Result<Self> {
        let mut config = Self::default();

        // Map server configuration
        config.server.port = app_config.server.port;
        config.server.host = app_config.server.host;

        // Map docs path if provided
        if let Some(docs_path) = app_config.server.docs_path {
            config.service.docs_path = std::path::PathBuf::from(docs_path);
        }

        // Map collection name
        config.service.default_collection = app_config.server.collection_name;

        // Map logging configuration
        config.logging.level = app_config.app.log_level;

        // Map timeouts
        config.server.timeout_seconds = app_config.server.timeout_ms / 1000;

        // Map vector storage
        let vector = app_config.vector;
        config.vector.backend = vector.backend.parse()?;
        config.vector.qdrant.url = vector.qdrant_url;
        config.vector.qdrant.collection_name = vector.qdrant_collection;
        config.vector.qdrant.api_key = vector.qdrant_api_key;
        config.vector.qdrant.timeout_seconds = vector.qdrant_timeout_seconds;
        if let Some(db_path) = vector.embedded_db_path {
            config.vector.embedded.db_path = expand_home(&db_path);
        }
        config.vector.embedded.dimension = vector.embedded_dimension;
        config.vector.embedded.cache_size = vector.embedded_cache_size;

        // Map embeddings
        let embedding = app_config.embedding;
        config.embedding.provider = embedding.provider.parse()?;
        config.embedding.openai.api_key = embedding.openai_api_key.unwrap_or_default();
        config.embedding.openai.model = embedding.openai_model;
        config.embedding.openai.base_url = embedding.openai_base_url;
        config.embedding.openai.timeout_seconds = embedding.openai_timeout_seconds;
        config.embedding.openai.max_retries = embedding.openai_max_retries;
        config.embedding.local.dimension = embedding.local_dimension;
        config.embedding.local.model_path = embedding.local_model_path.as_deref().map(expand_home);
        config.embedding.local.model_url = embedding.local_model_url;

        config.validate()?;
        Ok(config)
    }
}

impl Config {
    /// Defaults for tests that build a service container: in-memory vectors and
    /// the hash embedder, so no model files, disk store or network are needed.
    /// The hash embedder is not semantic: don't assert on ranking quality.
    pub fn for_tests() -> Self {
        let mut config = Self::default();
        config.vector.backend = VectorBackend::Memory;
        config.embedding.provider = EmbeddingProvider::Hash;
        config
    }
}

/// Expand a leading `~/` to the home directory
pub(crate) fn expand_home(path: &str) -> std::path::PathBuf {
    match (path.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => std::path::PathBuf::from(home).join(rest),
        _ => std::path::PathBuf::from(path),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            vector: VectorConfig {
                backend: VectorBackend::Embedded,
                qdrant: QdrantConfig::default(),
                embedded: EmbeddedConfig::default(),
            },
            embedding: EmbeddingConfig {
                provider: EmbeddingProvider::Local,
                openai: OpenAIConfig::default(),
                local: LocalEmbeddingConfig::default(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                structured: false,
            },
            service: ServiceConfig {
                name: "doc-indexer".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                max_document_size: 10 * 1024 * 1024, // 10MB
                default_search_limit: 10,
                max_search_limit: 100,
                default_collection: "zero_latency_docs".to_string(),
                enable_query_enhancement: true,
                enable_result_ranking: true,
                chunking_strategy: ChunkingStrategy::Sentence,
                chunk_size: 1000,
                chunk_overlap: 200,
                docs_path: if let Ok(home) = std::env::var("HOME") {
                    std::path::PathBuf::from(home).join("Documents")
                } else {
                    std::path::PathBuf::from("~/Documents")
                },
                max_binary_file_size: default_max_binary_file_size(),
                browse_roots: default_browse_roots(),
                browse_max_entries: default_browse_max_entries(),
            },

            load_testing: LoadTestingConfig::default(),
            production: ProductionConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_use_embedded_and_local() {
        let config = Config::from_app_config(AppConfig::default()).unwrap();
        assert!(matches!(config.vector.backend, VectorBackend::Embedded));
        assert!(matches!(
            config.embedding.provider,
            EmbeddingProvider::Local
        ));
        assert_eq!(config.embedding.local.dimension, 384);
    }

    #[test]
    fn test_maps_qdrant_and_openai_settings() {
        let mut app_config = AppConfig::default();
        app_config.vector.backend = "qdrant".to_string();
        app_config.vector.qdrant_url = "http://qdrant:6333".to_string();
        app_config.vector.qdrant_collection = "team_docs".to_string();
        app_config.vector.qdrant_api_key = Some("secret".to_string());
        app_config.vector.qdrant_timeout_seconds = 7;
        app_config.embedding.provider = "openai".to_string();
        app_config.embedding.openai_api_key = Some("sk-test".to_string());
        app_config.embedding.openai_model = "text-embedding-3-large".to_string();
        app_config.embedding.openai_base_url = Some("http://proxy/v1".to_string());
        app_config.embedding.openai_max_retries = 5;

        let config = Config::from_app_config(app_config).unwrap();

        assert!(matches!(config.vector.backend, VectorBackend::Qdrant));
        assert_eq!(config.vector.qdrant.url, "http://qdrant:6333");
        assert_eq!(config.vector.qdrant.collection_name, "team_docs");
        assert_eq!(config.vector.qdrant.api_key.as_deref(), Some("secret"));
        assert_eq!(config.vector.qdrant.timeout_seconds, 7);
        assert!(matches!(
            config.embedding.provider,
            EmbeddingProvider::OpenAI
        ));
        assert_eq!(config.embedding.openai.api_key, "sk-test");
        assert_eq!(config.embedding.openai.model, "text-embedding-3-large");
        assert_eq!(
            config.embedding.openai.base_url.as_deref(),
            Some("http://proxy/v1")
        );
        assert_eq!(config.embedding.openai.max_retries, 5);
    }

    #[test]
    fn test_unknown_backend_is_error() {
        let mut app_config = AppConfig::default();
        app_config.vector.backend = "pinecone".to_string();
        let error = Config::from_app_config(app_config).unwrap_err();
        assert!(error.to_string().contains("pinecone"), "{}", error);
    }

    #[test]
    fn test_openai_without_key_is_error() {
        let mut app_config = AppConfig::default();
        app_config.embedding.provider = "openai".to_string();
        let error = Config::from_app_config(app_config).unwrap_err();
        assert!(error.to_string().contains("API key"), "{}", error);
    }

    #[test]
    fn test_maps_local_model_settings() {
        let mut app_config = AppConfig::default();
        app_config.embedding.local_model_path = Some("~/models/bge".to_string());
        app_config.embedding.local_model_url = Some("http://mirror/bge".to_string());
        let config = Config::from_app_config(app_config).unwrap();
        let home = std::env::var_os("HOME").unwrap();
        assert_eq!(
            config.embedding.local.model_path,
            Some(std::path::PathBuf::from(home).join("models/bge"))
        );
        assert_eq!(
            config.embedding.local.model_url.as_deref(),
            Some("http://mirror/bge")
        );
    }

    #[test]
    fn test_hash_provider_parses() {
        let mut app_config = AppConfig::default();
        app_config.embedding.provider = "hash".to_string();
        let config = Config::from_app_config(app_config).unwrap();
        assert!(matches!(config.embedding.provider, EmbeddingProvider::Hash));
    }

    #[test]
    fn test_local_dimension_other_than_384_is_error() {
        let mut config = Config::default();
        config.embedding.local.dimension = 768;
        let error = config.validate().unwrap_err();
        assert!(error.to_string().contains("384"), "{}", error);

        config.embedding.provider = EmbeddingProvider::Hash;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_embedded_db_path_expands_home() {
        let mut app_config = AppConfig::default();
        app_config.vector.embedded_db_path = Some("~/data/vectors.db".to_string());
        let config = Config::from_app_config(app_config).unwrap();
        let home = std::env::var_os("HOME").unwrap();
        assert_eq!(
            config.vector.embedded.db_path,
            std::path::PathBuf::from(home).join("data/vectors.db")
        );
    }
}
