/// Implementation adapters for service interfaces
/// 
/// These adapters implement the focused service interfaces using existing
/// infrastructure components, following the Adapter pattern and DIP principle

use std::sync::Arc;
use std::path::Path;
use async_trait::async_trait;
use zero_latency_core::Result;
use zero_latency_vector::{VectorRepository, EmbeddingGenerator, VectorDocument};
use zero_latency_search::{SearchOrchestrator, SearchRequest, SearchResponse};

use super::interfaces::{
    VectorStorage, EmbeddingService, SearchService, FileSystemService, 
    ProgressTracker, FilteringService, CollectionManager,
    FileMetadata, ProgressStats, FilterSummary, CollectionStats
};
use super::services::filter_service::FilterService;

/// Adapter implementing VectorStorage interface using VectorRepository
pub struct VectorStorageAdapter {
    repository: Arc<dyn VectorRepository>,
}

impl VectorStorageAdapter {
    pub fn new(repository: Arc<dyn VectorRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl VectorStorage for VectorStorageAdapter {
    async fn store_vector(&self, document: VectorDocument) -> Result<()> {
        self.repository.insert(vec![document]).await
    }
    
    async fn store_vectors(&self, documents: Vec<VectorDocument>) -> Result<()> {
        self.repository.insert(documents).await
    }
    
    async fn remove_vectors(&self, document_id: &str) -> Result<()> {
        self.repository.delete(document_id).await.map(|_| ())
    }
    
    async fn has_vectors(&self, document_id: &str) -> Result<bool> {
        // This would need to be implemented in the VectorRepository trait
        // For now, we'll return false and implement this later
        Ok(false)
    }
}

/// Adapter implementing EmbeddingService interface using EmbeddingGenerator
pub struct EmbeddingServiceAdapter {
    generator: Arc<dyn EmbeddingGenerator>,
}

impl EmbeddingServiceAdapter {
    pub fn new(generator: Arc<dyn EmbeddingGenerator>) -> Self {
        Self { generator }
    }
}

#[async_trait]
impl EmbeddingService for EmbeddingServiceAdapter {
    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        self.generator.generate_embedding(text).await
    }
    
    async fn generate_batch_embeddings(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.generator.generate_embedding(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }
    
    fn embedding_dimension(&self) -> usize {
        self.generator.dimension()
    }
}

/// Adapter implementing SearchService interface using SearchOrchestrator
pub struct SearchServiceAdapter {
    orchestrator: Arc<dyn SearchOrchestrator>,
}

impl SearchServiceAdapter {
    pub fn new(orchestrator: Arc<dyn SearchOrchestrator>) -> Self {
        Self { orchestrator }
    }
}

#[async_trait]
impl SearchService for SearchServiceAdapter {
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        self.orchestrator.search(request).await
    }
    
    async fn health_check(&self) -> Result<bool> {
        // This would need to be implemented in the SearchOrchestrator trait
        // For now, we'll return true and implement this later
        Ok(true)
    }
}

/// Implementation of FileSystemService using standard library
pub struct StandardFileSystemService;

impl StandardFileSystemService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileSystemService for StandardFileSystemService {
    async fn read_file_content(&self, path: &Path) -> Result<String> {
        use tokio::fs;
        fs::read_to_string(path)
            .await
            .map_err(|e| zero_latency_core::ZeroLatencyError::internal(&format!("Failed to read file: {}", e)))
    }
    
    async fn is_file(&self, path: &Path) -> Result<bool> {
        Ok(path.is_file())
    }
    
    async fn is_directory(&self, path: &Path) -> Result<bool> {
        Ok(path.is_dir())
    }
    
    async fn list_directory(&self, path: &Path) -> Result<Vec<std::path::PathBuf>> {
        use tokio::fs;
        let mut entries = fs::read_dir(path)
            .await
                .map_err(|e| zero_latency_core::ZeroLatencyError::internal(&format!("Failed to read directory: {}", e)))?;        let mut paths = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| 
            zero_latency_core::ZeroLatencyError::internal(&format!("Failed to read directory entry: {}", e)))? 
        {
            paths.push(entry.path());
        }
        Ok(paths)
    }
    
    async fn get_file_metadata(&self, path: &Path) -> Result<FileMetadata> {
        use tokio::fs;
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| zero_latency_core::ZeroLatencyError::internal(&format!("Failed to read metadata: {}", e)))?;
        
        let modified = metadata
            .modified()
            .map_err(|e| zero_latency_core::ZeroLatencyError::internal(&format!("Failed to read modified time: {}", e)))?;
        
        Ok(FileMetadata {
            size: metadata.len(),
            modified: chrono::DateTime::<chrono::Utc>::from(modified),
            is_file: metadata.is_file(),
            is_directory: metadata.is_dir(),
        })
    }
}

/// Simple in-memory progress tracker implementation
pub struct InMemoryProgressTracker {
    stats: tokio::sync::RwLock<ProgressStats>,
    start_time: std::time::Instant,
}

impl InMemoryProgressTracker {
    pub fn new() -> Self {
        Self {
            stats: tokio::sync::RwLock::new(ProgressStats {
                files_processed: 0,
                files_failed: 0,
                total_files: 0,
                elapsed_ms: 0,
                processing_rate: 0.0,
            }),
            start_time: std::time::Instant::now(),
        }
    }
}

#[async_trait]
impl ProgressTracker for InMemoryProgressTracker {
    async fn file_processed(&self, _path: &Path, success: bool) {
        let mut stats = self.stats.write().await;
        if success {
            stats.files_processed += 1;
        } else {
            stats.files_failed += 1;
        }
        
        let elapsed = self.start_time.elapsed();
        stats.elapsed_ms = elapsed.as_millis() as u64;
        
        let total_processed = stats.files_processed + stats.files_failed;
        if total_processed > 0 && elapsed.as_secs_f64() > 0.0 {
            stats.processing_rate = total_processed as f64 / elapsed.as_secs_f64();
        }
    }
    
    async fn progress_update(&self, processed: u64, total: u64, elapsed_ms: u64) {
        let mut stats = self.stats.write().await;
        stats.files_processed = processed;
        stats.total_files = total;
        stats.elapsed_ms = elapsed_ms;
        
        if elapsed_ms > 0 {
            stats.processing_rate = processed as f64 / (elapsed_ms as f64 / 1000.0);
        }
    }
    
    async fn get_progress(&self) -> Result<ProgressStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

/// Adapter implementing FilteringService interface using FilterService
pub struct FilteringServiceAdapter {
    filter_service: Arc<FilterService>,
}

impl FilteringServiceAdapter {
    pub fn new(filter_service: Arc<FilterService>) -> Self {
        Self { filter_service }
    }
}

impl FilteringService for FilteringServiceAdapter {
    fn should_index_file(&self, path: &Path) -> bool {
        self.filter_service.should_index(path)
    }
    
    fn should_traverse_directory(&self, path: &Path) -> bool {
        // For now, we'll traverse all directories except hidden ones
        !path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }
    
    fn get_filter_summary(&self) -> FilterSummary {
        let filters = self.filter_service.filters();
        FilterSummary {
            safe_patterns: filters.safe_list.clone(),
            ignore_patterns: filters.ignore_list.clone(),
            case_sensitive: filters.case_sensitive,
            follow_symlinks: filters.follow_symlinks,
        }
    }
}

/// Placeholder collection manager implementation
pub struct SimpleCollectionManager;

impl SimpleCollectionManager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CollectionManager for SimpleCollectionManager {
    async fn create_collection(&self, _name: &str) -> Result<()> {
        // This would integrate with the existing collection service
        // For now, we'll just return success
        Ok(())
    }
    
    async fn delete_collection(&self, _name: &str) -> Result<()> {
        // This would integrate with the existing collection service
        Ok(())
    }
    
    async fn list_collections(&self) -> Result<Vec<String>> {
        // This would integrate with the existing collection service
        Ok(vec!["default".to_string(), "zero_latency_docs".to_string()])
    }
    
    async fn get_collection_stats(&self, name: &str) -> Result<CollectionStats> {
        // This would integrate with the existing collection service
        Ok(CollectionStats {
            name: name.to_string(),
            document_count: 0,
            vector_count: 0,
            storage_size_bytes: 0,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        })
    }
}
