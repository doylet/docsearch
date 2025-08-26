// Unit tests for IndexingService
// Tests the core SOLID service layer functionality with mocked dependencies

use std::sync::Arc;
use std::path::Path;
use tokio;
use mockall::mock;

// Import our service and interfaces
use doc_indexer::application::services::indexing_service::{IndexingService, IndexingServiceBuilder};
use doc_indexer::application::interfaces::*;
use doc_indexer::application::content_processor::ContentProcessor;
use doc_indexer::domain::Document;
use zero_latency_core::Result;

// Mock implementations for testing
mock! {
    VectorStorage {}
    
    #[async_trait::async_trait]
    impl VectorStorage for VectorStorage {
        async fn store_vectors(&self, document_id: &str, vectors: Vec<f32>) -> Result<()>;
        async fn search_similar(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>>;
        async fn remove_vectors(&self, document_id: &str) -> Result<()>;
        async fn get_document_count(&self) -> Result<u64>;
        async fn has_vectors(&self, document_id: &str) -> Result<bool>;
    }
}

mock! {
    EmbeddingService {}
    
    #[async_trait::async_trait]
    impl EmbeddingService for EmbeddingService {
        async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
        async fn generate_batch_embeddings(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
        fn embedding_dimension(&self) -> usize;
    }
}

mock! {
    FileSystemService {}
    
    #[async_trait::async_trait]
    impl FileSystemService for FileSystemService {
        async fn read_file_content(&self, path: &Path) -> Result<String>;
        async fn exists(&self, path: &Path) -> Result<bool>;
        async fn is_file(&self, path: &Path) -> Result<bool>;
        async fn is_directory(&self, path: &Path) -> Result<bool>;
        async fn list_directory(&self, path: &Path) -> Result<Vec<std::path::PathBuf>>;
        async fn get_file_metadata(&self, path: &Path) -> Result<FileMetadata>;
    }
}

mock! {
    FilteringService {}
    
    impl FilteringService for FilteringService {
        fn should_index_file(&self, path: &Path) -> bool;
        fn should_traverse_directory(&self, path: &Path) -> bool;
        fn get_filter_summary(&self) -> FilterSummary;
    }
}

mock! {
    ProgressTracker {}
    
    #[async_trait::async_trait]
    impl ProgressTracker for ProgressTracker {
        async fn file_processed(&self, path: &Path, success: bool);
        async fn progress_update(&self, processed: u64, total: u64, elapsed_ms: u64);
        async fn get_progress(&self) -> Result<ProgressStats>;
    }
}

mock! {
    CollectionManager {}
    
    #[async_trait::async_trait]
    impl CollectionManager for CollectionManager {
        async fn create_collection(&self, name: &str) -> Result<()>;
        async fn delete_collection(&self, name: &str) -> Result<()>;
        async fn list_collections(&self) -> Result<Vec<String>>;
        async fn get_collection_stats(&self, name: &str) -> Result<CollectionStats>;
    }
}

mock! {
    IndexingStrategy {}
    
    #[async_trait::async_trait]
    impl IndexingStrategy for IndexingStrategy {
        async fn index_document(
            &self,
            document: &Document,
            vector_storage: &dyn VectorStorage,
            embedding_service: &dyn EmbeddingService,
        ) -> Result<()>;
        fn get_config(&self) -> super::super::super::application::indexing_strategies::StrategyConfig;
    }
}


