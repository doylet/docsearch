use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU16, Ordering};
use uuid::Uuid;

/// Main configuration structure for Zero-Latency applications
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Client configuration
    pub client: ClientConfig,

    /// Test configuration
    pub test: TestConfig,

    /// Global application settings
    pub app: GlobalConfig,

    /// Vector storage backend settings
    #[serde(default)]
    pub vector: VectorSettings,

    /// Embedding provider settings
    #[serde(default)]
    pub embedding: EmbeddingSettings,
}

/// Server configuration for doc-indexer and other services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host (default: localhost)
    pub host: String,

    /// Server port (default: 8081)
    pub port: u16,

    /// Documents path for indexing
    pub docs_path: Option<String>,

    /// Collection name for vector storage
    pub collection_name: String,

    /// Server timeout in milliseconds
    pub timeout_ms: u64,
}

/// Client configuration for CLI and API clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Base server URL (constructed from server config by default)
    pub server_url: Option<String>,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
}

/// Test-specific configuration for reliable test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Base port for test servers (incremented for each test)
    pub port_base: u16,

    /// Test fixture path
    pub fixture_path: String,

    /// Binary path for doc-indexer executable
    pub binary_path: Option<String>,

    /// Test collection name prefix (appended with timestamp)
    pub collection_prefix: String,

    /// Test timeout in milliseconds
    pub timeout_ms: u64,
}

/// Global application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Logging level (error, warn, info, debug, trace)
    pub log_level: String,

    /// Default output format (table, json, simple)
    pub output_format: String,

    /// Enable debug mode
    pub debug: bool,
}

/// Vector storage settings. Fields are flat so each maps to one env var,
/// e.g. `ZL_VECTOR_QDRANT_URL` → `vector.qdrant_url`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VectorSettings {
    /// Backend: `embedded`, `memory` or `qdrant` (default: embedded)
    pub backend: String,

    /// Qdrant REST URL
    pub qdrant_url: String,

    /// Qdrant collection name
    pub qdrant_collection: String,

    /// Qdrant API key, sent as the `api-key` header
    pub qdrant_api_key: Option<String>,

    /// Qdrant request timeout in seconds
    pub qdrant_timeout_seconds: u64,

    /// Embedded store database path (default: ~/.zero-latency/vectors.db)
    pub embedded_db_path: Option<String>,

    /// Embedded store vector dimension
    pub embedded_dimension: usize,

    /// Embedded store cache size, in documents
    pub embedded_cache_size: usize,
}

/// Embedding provider settings, e.g. `ZL_EMBEDDING_OPENAI_MODEL` → `embedding.openai_model`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbeddingSettings {
    /// Provider: `local` (bge-small-en-v1.5), `openai`, or `hash` (testing only,
    /// not semantic). Default: local
    pub provider: String,

    /// OpenAI API key; falls back to the `OPENAI_API_KEY` environment variable
    pub openai_api_key: Option<String>,

    /// OpenAI embedding model
    pub openai_model: String,

    /// OpenAI-compatible API base URL (default: https://api.openai.com/v1)
    pub openai_base_url: Option<String>,

    /// OpenAI request timeout in seconds
    pub openai_timeout_seconds: u64,

    /// Retries on rate limits and server errors
    pub openai_max_retries: u32,

    /// Local embedding dimension. Fixed by the model: must be 384
    pub local_dimension: usize,

    /// Directory holding pre-supplied `model.onnx` and `tokenizer.json`.
    /// When set, nothing is downloaded. Default: `~/.zero-latency/models/bge-small-en-v1.5`
    pub local_model_path: Option<String>,

    /// Base URL to download the local model from (default: https://huggingface.co/BAAI/bge-small-en-v1.5)
    pub local_model_url: Option<String>,
}

impl Default for VectorSettings {
    fn default() -> Self {
        Self {
            backend: "embedded".to_string(),
            qdrant_url: "http://localhost:6333".to_string(),
            qdrant_collection: "zero_latency_docs".to_string(),
            qdrant_api_key: None,
            qdrant_timeout_seconds: 30,
            embedded_db_path: None,
            embedded_dimension: 384,
            embedded_cache_size: 10000,
        }
    }
}

impl Default for EmbeddingSettings {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            openai_api_key: None,
            openai_model: "text-embedding-3-small".to_string(),
            openai_base_url: None,
            openai_timeout_seconds: 30,
            openai_max_retries: 3,
            local_dimension: 384,
            local_model_path: None,
            local_model_url: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8081,
            docs_path: None,
            collection_name: "zero_latency_docs".to_string(),
            timeout_ms: 30000,
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: None, // Will be constructed from server config
            timeout_ms: 30000,
            max_retries: 3,
            connect_timeout_ms: 5000,
        }
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            port_base: 19000,
            fixture_path: "./test-fixtures".to_string(),
            binary_path: None, // Will be resolved at runtime
            collection_prefix: "test_collection".to_string(),
            timeout_ms: 60000, // Longer timeout for tests
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            output_format: "table".to_string(),
            debug: false,
        }
    }
}

impl ServerConfig {
    /// Get the full server URL
    pub fn server_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl ClientConfig {
    /// Get the server URL, using the provided server config if none is set
    pub fn get_server_url(&self, server_config: &ServerConfig) -> String {
        self.server_url
            .clone()
            .unwrap_or_else(|| server_config.server_url())
    }
}

/// Process-wide port offset, shared so separate helper instances never hand out the same port
static NEXT_PORT_OFFSET: AtomicU16 = AtomicU16::new(0);

/// Test configuration helper for unique port allocation and collection names
pub struct TestConfigHelper {
    base_config: TestConfig,
}

impl TestConfigHelper {
    /// Create a new test configuration helper
    pub fn new() -> Self {
        Self {
            base_config: TestConfig::default(),
        }
    }

    /// Create with custom base configuration
    pub fn with_config(config: TestConfig) -> Self {
        Self {
            base_config: config,
        }
    }

    /// Get a unique port for testing
    pub fn get_unique_port(&self) -> u16 {
        // Skip ports still held by another process, e.g. a server leaked by an earlier run
        for _ in 0..1000 {
            let offset = NEXT_PORT_OFFSET.fetch_add(1, Ordering::SeqCst);
            let port = self.base_config.port_base.wrapping_add(offset);
            if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
                return port;
            }
        }
        panic!(
            "no free test port found above {}",
            self.base_config.port_base
        );
    }

    /// Get a unique collection name for testing
    pub fn get_unique_collection_name(&self) -> String {
        let uuid = Uuid::new_v4();
        format!("{}_{}", self.base_config.collection_prefix, uuid.simple())
    }

    /// Resolve binary path with multiple fallback locations
    pub fn resolve_binary_path(&self) -> Option<String> {
        if let Some(path) = &self.base_config.binary_path {
            if std::path::Path::new(path).exists() {
                return Some(path.clone());
            }
        }

        // Fallback paths to search for doc-indexer binary
        let fallback_paths = [
            "./target/debug/doc-indexer",
            "./target/release/doc-indexer",
            "./services/doc-indexer/target/debug/doc-indexer",
            "./services/doc-indexer/target/release/doc-indexer",
            "doc-indexer", // In PATH
        ];

        for path in &fallback_paths {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }

        None
    }

    /// Create a complete test configuration with unique values
    pub fn create_test_config(&self) -> TestConfig {
        TestConfig {
            port_base: self.get_unique_port(),
            fixture_path: self.base_config.fixture_path.clone(),
            binary_path: self.resolve_binary_path(),
            collection_prefix: self.get_unique_collection_name(),
            timeout_ms: self.base_config.timeout_ms,
        }
    }
}

impl Default for TestConfigHelper {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy Config struct for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server URL for API communication
    pub server_url: String,

    /// Collection name for vector storage (CLI override)
    pub collection_name: String,

    /// Timeout for API requests in seconds
    pub timeout_seconds: u64,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Logging level (error, warn, info, debug, trace)
    pub log_level: String,

    /// Default output format (table, json, simple)
    pub output_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8081".to_string(),
            collection_name: "zero_latency_docs".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            log_level: "info".to_string(),
            output_format: "table".to_string(),
        }
    }
}

impl From<AppConfig> for Config {
    fn from(app_config: AppConfig) -> Self {
        Self {
            server_url: app_config.client.get_server_url(&app_config.server),
            collection_name: app_config.server.collection_name,
            timeout_seconds: app_config.client.timeout_ms / 1000,
            max_retries: app_config.client.max_retries,
            log_level: app_config.app.log_level,
            output_format: app_config.app.output_format,
        }
    }
}
