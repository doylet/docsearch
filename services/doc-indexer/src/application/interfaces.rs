/// Service abstractions following Interface Segregation Principle
/// 
/// This module defines focused interfaces that represent specific service capabilities
/// rather than large, monolithic service containers. Each interface represents
/// a single concern that can be implemented independently.

use std::path::Path;
use async_trait::async_trait;
use zero_latency_core::{Result, DateTime, Utc};
use zero_latency_vector::VectorDocument;
use zero_latency_search::{SearchRequest, SearchResponse};

/// Interface for vector storage operations
/// 
/// This interface segregates vector storage concerns from other service responsibilities
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Store a vector document in the repository
    async fn store_vector(&self, document: VectorDocument) -> Result<()>;
    
    /// Store multiple vector documents in batch
    async fn store_vectors(&self, documents: Vec<VectorDocument>) -> Result<()>;
    
    /// Remove vectors for a specific document
    async fn remove_vectors(&self, document_id: &str) -> Result<()>;
    
    /// Check if vectors exist for a document
    async fn has_vectors(&self, document_id: &str) -> Result<bool>;
}

/// Interface for embedding generation operations
/// 
/// Focuses solely on converting text to vector embeddings
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embeddings for text content
    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Generate embeddings for multiple text chunks in batch
    async fn generate_batch_embeddings(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    
    /// Get the dimensionality of embeddings produced by this service
    fn embedding_dimension(&self) -> usize;
}

/// Interface for search operations
/// 
/// Segregates search concerns from indexing concerns
#[async_trait]
pub trait SearchService: Send + Sync {
    /// Perform a search operation
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
    
    /// Check if search index is healthy
    async fn health_check(&self) -> Result<bool>;
}

/// Interface for file system operations
/// 
/// Separates file I/O concerns from business logic
#[async_trait]
pub trait FileSystemService: Send + Sync {
    /// Read file content as string
    async fn read_file_content(&self, path: &Path) -> Result<String>;
    
    /// Check if path exists and is a file
    async fn is_file(&self, path: &Path) -> Result<bool>;
    
    /// Check if path exists and is a directory
    async fn is_directory(&self, path: &Path) -> Result<bool>;
    
    /// List entries in a directory
    async fn list_directory(&self, path: &Path) -> Result<Vec<std::path::PathBuf>>;
    
    /// Get file metadata (size, modified time, etc.)
    async fn get_file_metadata(&self, path: &Path) -> Result<FileMetadata>;
}

/// File metadata information
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub is_file: bool,
    pub is_directory: bool,
}

/// Interface for indexing progress tracking
/// 
/// Separates progress reporting from indexing logic
#[async_trait]
pub trait ProgressTracker: Send + Sync {
    /// Report that a file has been processed
    async fn file_processed(&self, path: &Path, success: bool);
    
    /// Report progress update
    async fn progress_update(&self, processed: u64, total: u64, elapsed_ms: u64);
    
    /// Get current progress statistics
    async fn get_progress(&self) -> Result<ProgressStats>;
}

/// Progress statistics
#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub files_processed: u64,
    pub files_failed: u64,
    pub total_files: u64,
    pub elapsed_ms: u64,
    pub processing_rate: f64, // files per second
}

/// Interface for filtering decisions
/// 
/// Separates filtering logic from indexing logic
pub trait FilteringService: Send + Sync {
    /// Check if a file should be indexed based on path
    fn should_index_file(&self, path: &Path) -> bool;
    
    /// Check if a directory should be traversed
    fn should_traverse_directory(&self, path: &Path) -> bool;
    
    /// Get filter configuration summary
    fn get_filter_summary(&self) -> FilterSummary;
}

/// Filter configuration summary
#[derive(Debug, Clone)]
pub struct FilterSummary {
    pub safe_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub case_sensitive: bool,
    pub follow_symlinks: bool,
}

/// Interface for collection management
/// 
/// Separates collection concerns from document indexing
#[async_trait]
pub trait CollectionManager: Send + Sync {
    /// Create a new collection
    async fn create_collection(&self, name: &str) -> Result<()>;
    
    /// Delete a collection
    async fn delete_collection(&self, name: &str) -> Result<()>;
    
    /// List all collections
    async fn list_collections(&self) -> Result<Vec<String>>;
    
    /// Get collection statistics
    async fn get_collection_stats(&self, name: &str) -> Result<CollectionStats>;
}

/// Collection statistics
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub name: String,
    pub document_count: u64,
    pub vector_count: u64,
    pub storage_size_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
