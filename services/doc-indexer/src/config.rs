/// Configuration management for doc-indexer service
/// 
/// This module handles loading and validating configuration from various sources
/// including environment variables, configuration files, and command line arguments.

use serde::{Deserialize, Serialize};
use zero_latency_core::{Result, ZeroLatencyError};
use crate::infrastructure::{ServerConfig, QdrantConfig, OpenAIConfig, LocalEmbeddingConfig, EmbeddedConfig};

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
    
    /// Document chunking strategy
    pub chunking_strategy: ChunkingStrategy,
    
    /// Chunk size in characters
    pub chunk_size: usize,
    
    /// Chunk overlap in characters
    pub chunk_overlap: usize,
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
    Local,
    OpenAI,
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
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Result<Self> {
        let config = Config {
            server: ServerConfig {
                host: std::env::var("DOC_INDEXER_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("DOC_INDEXER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|_| ZeroLatencyError::configuration("Invalid port number"))?,
                timeout_seconds: std::env::var("DOC_INDEXER_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                enable_cors: std::env::var("DOC_INDEXER_ENABLE_CORS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                cors_origins: std::env::var("DOC_INDEXER_CORS_ORIGINS")
                    .unwrap_or_default()
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            
            vector: VectorConfig {
                backend: std::env::var("DOC_INDEXER_VECTOR_BACKEND")
                    .unwrap_or_else(|_| "embedded".to_string())
                    .parse()
                    .unwrap_or(VectorBackend::Embedded),
                qdrant: QdrantConfig {
                    url: std::env::var("DOC_INDEXER_QDRANT_URL")
                        .unwrap_or_else(|_| "http://localhost:6333".to_string()),
                    collection_name: std::env::var("DOC_INDEXER_QDRANT_COLLECTION")
                        .unwrap_or_else(|_| "zero_latency_docs".to_string()),
                    api_key: std::env::var("DOC_INDEXER_QDRANT_API_KEY").ok(),
                    timeout_seconds: std::env::var("DOC_INDEXER_QDRANT_TIMEOUT")
                        .unwrap_or_else(|_| "30".to_string())
                        .parse()
                        .unwrap_or(30),
                },
                embedded: EmbeddedConfig {
                    db_path: std::env::var("DOC_INDEXER_EMBEDDED_DB_PATH")
                        .map(|p| {
                            if p.starts_with("~/") {
                                dirs::home_dir()
                                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                                    .join(&p[2..])
                            } else {
                                std::path::PathBuf::from(p)
                            }
                        })
                        .unwrap_or_else(|_| {
                            dirs::home_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join(".zero-latency")
                                .join("vectors.db")
                        }),
                    dimension: std::env::var("DOC_INDEXER_EMBEDDED_DIMENSION")
                        .unwrap_or_else(|_| "384".to_string())
                        .parse()
                        .unwrap_or(384),
                    cache_size: std::env::var("DOC_INDEXER_EMBEDDED_CACHE_SIZE")
                        .unwrap_or_else(|_| "10000".to_string())
                        .parse()
                        .unwrap_or(10000),
                },
            },
            
            embedding: EmbeddingConfig {
                provider: std::env::var("DOC_INDEXER_EMBEDDING_PROVIDER")
                    .unwrap_or_else(|_| "local".to_string())
                    .parse()
                    .unwrap_or(EmbeddingProvider::Local),
                openai: OpenAIConfig {
                    api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                    model: std::env::var("DOC_INDEXER_OPENAI_MODEL")
                        .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
                    base_url: std::env::var("DOC_INDEXER_OPENAI_BASE_URL").ok(),
                    timeout_seconds: std::env::var("DOC_INDEXER_OPENAI_TIMEOUT")
                        .unwrap_or_else(|_| "30".to_string())
                        .parse()
                        .unwrap_or(30),
                    max_retries: std::env::var("DOC_INDEXER_OPENAI_MAX_RETRIES")
                        .unwrap_or_else(|_| "3".to_string())
                        .parse()
                        .unwrap_or(3),
                },
                local: LocalEmbeddingConfig {
                    dimension: std::env::var("DOC_INDEXER_LOCAL_EMBEDDING_DIMENSION")
                        .unwrap_or_else(|_| "384".to_string())
                        .parse()
                        .unwrap_or(384),
                    seed: std::env::var("DOC_INDEXER_LOCAL_EMBEDDING_SEED")
                        .unwrap_or_else(|_| "42".to_string())
                        .parse()
                        .unwrap_or(42),
                },
            },
            
            logging: LoggingConfig {
                level: std::env::var("DOC_INDEXER_LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                format: std::env::var("DOC_INDEXER_LOG_FORMAT")
                    .unwrap_or_else(|_| "pretty".to_string()),
                structured: std::env::var("DOC_INDEXER_LOG_STRUCTURED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
            
            service: ServiceConfig {
                name: "doc-indexer".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                max_document_size: std::env::var("DOC_INDEXER_MAX_DOCUMENT_SIZE")
                    .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                    .parse()
                    .unwrap_or(10 * 1024 * 1024),
                default_search_limit: std::env::var("DOC_INDEXER_DEFAULT_SEARCH_LIMIT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                max_search_limit: std::env::var("DOC_INDEXER_MAX_SEARCH_LIMIT")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                chunking_strategy: std::env::var("DOC_INDEXER_CHUNKING_STRATEGY")
                    .unwrap_or_else(|_| "sentence".to_string())
                    .parse()
                    .unwrap_or(ChunkingStrategy::Sentence),
                chunk_size: std::env::var("DOC_INDEXER_CHUNK_SIZE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                chunk_overlap: std::env::var("DOC_INDEXER_CHUNK_OVERLAP")
                    .unwrap_or_else(|_| "200".to_string())
                    .parse()
                    .unwrap_or(200),
            },
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ZeroLatencyError::configuration(&format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ZeroLatencyError::configuration(&format!("Failed to parse config file: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(ZeroLatencyError::configuration("Server port must be greater than 0"));
        }
        
        // Validate embedding configuration
        match self.embedding.provider {
            EmbeddingProvider::OpenAI => {
                if self.embedding.openai.api_key.is_empty() {
                    return Err(ZeroLatencyError::configuration("OpenAI API key is required"));
                }
            }
            EmbeddingProvider::Local => {
                if self.embedding.local.dimension == 0 {
                    return Err(ZeroLatencyError::configuration("Local embedding dimension must be greater than 0"));
                }
            }
        }
        
        // Validate service configuration
        if self.service.max_document_size == 0 {
            return Err(ZeroLatencyError::configuration("Max document size must be greater than 0"));
        }
        
        if self.service.chunk_size == 0 {
            return Err(ZeroLatencyError::configuration("Chunk size must be greater than 0"));
        }
        
        Ok(())
    }
    
    /// Get configuration as environment variable examples
    pub fn env_example() -> String {
        r#"# Doc-Indexer Configuration
DOC_INDEXER_HOST=0.0.0.0
DOC_INDEXER_PORT=8080
DOC_INDEXER_TIMEOUT=30
DOC_INDEXER_ENABLE_CORS=true
DOC_INDEXER_CORS_ORIGINS=http://localhost:3000,http://localhost:3001

# Vector Storage
DOC_INDEXER_VECTOR_BACKEND=embedded
DOC_INDEXER_QDRANT_URL=http://localhost:6333
DOC_INDEXER_QDRANT_COLLECTION=documents
DOC_INDEXER_QDRANT_API_KEY=your-api-key
DOC_INDEXER_QDRANT_TIMEOUT=30

# Embedded Vector Storage
DOC_INDEXER_EMBEDDED_DB_PATH=~/.zero-latency/vectors.db
DOC_INDEXER_EMBEDDED_DIMENSION=384
DOC_INDEXER_EMBEDDED_CACHE_SIZE=10000

# Embeddings
DOC_INDEXER_EMBEDDING_PROVIDER=local
OPENAI_API_KEY=your-openai-api-key
DOC_INDEXER_OPENAI_MODEL=text-embedding-3-small
DOC_INDEXER_OPENAI_TIMEOUT=30
DOC_INDEXER_OPENAI_MAX_RETRIES=3
DOC_INDEXER_LOCAL_EMBEDDING_DIMENSION=384
DOC_INDEXER_LOCAL_EMBEDDING_SEED=42

# Logging
DOC_INDEXER_LOG_LEVEL=info
DOC_INDEXER_LOG_FORMAT=pretty
DOC_INDEXER_LOG_STRUCTURED=false

# Service Settings
DOC_INDEXER_MAX_DOCUMENT_SIZE=10485760
DOC_INDEXER_DEFAULT_SEARCH_LIMIT=10
DOC_INDEXER_MAX_SEARCH_LIMIT=100
DOC_INDEXER_CHUNKING_STRATEGY=sentence
DOC_INDEXER_CHUNK_SIZE=1000
DOC_INDEXER_CHUNK_OVERLAP=200
"#.to_string()
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
            _ => Err(ZeroLatencyError::configuration(&format!("Unknown vector backend: {}", s))),
        }
    }
}

impl std::str::FromStr for EmbeddingProvider {
    type Err = ZeroLatencyError;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(EmbeddingProvider::Local),
            "openai" => Ok(EmbeddingProvider::OpenAI),
            _ => Err(ZeroLatencyError::configuration(&format!("Unknown embedding provider: {}", s))),
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
            _ => Err(ZeroLatencyError::configuration(&format!("Unknown chunking strategy: {}", s))),
        }
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
                chunking_strategy: ChunkingStrategy::Sentence,
                chunk_size: 1000,
                chunk_overlap: 200,
            },
        }
    }
}
