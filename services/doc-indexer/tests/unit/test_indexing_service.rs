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

#[tokio::test]
async fn test_indexing_service_builder_pattern() {
    // Test that the builder pattern correctly constructs an IndexingService
    let mock_vector_storage = Arc::new(MockVectorStorage::new());
    let mock_embedding_service = Arc::new(MockEmbeddingService::new());
    let mock_file_system = Arc::new(MockFileSystemService::new());
    let mock_filtering = Arc::new(MockFilteringService::new());
    let mock_progress_tracker = Arc::new(MockProgressTracker::new());
    let mock_collection_manager = Arc::new(MockCollectionManager::new());
    let mock_strategy = Arc::new(MockIndexingStrategy::new());
    
    let result = IndexingServiceBuilder::new()
        .vector_storage(mock_vector_storage)
        .embedding_service(mock_embedding_service)
        .file_system(mock_file_system)
        .filtering(mock_filtering)
        .progress_tracker(mock_progress_tracker)
        .collection_manager(mock_collection_manager)
        .indexing_strategy(mock_strategy)
        .content_processor(ContentProcessor)
        .build();
    
    assert!(result.is_ok(), "IndexingService should build successfully with all dependencies");
}

#[tokio::test]
async fn test_indexing_service_index_file_success() {
    // Test successful file indexing
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    let mut mock_file_system = MockFileSystemService::new();
    let mut mock_filtering = MockFilteringService::new();
    let mut mock_progress_tracker = MockProgressTracker::new();
    let mut mock_collection_manager = MockCollectionManager::new();
    let mut mock_strategy = MockIndexingStrategy::new();
    
    // Set up expectations
    mock_file_system
        .expect_exists()
        .returning(|_| Ok(true));
    
    mock_file_system
        .expect_is_file()
        .returning(|_| Ok(true));
    
    mock_filtering
        .expect_should_index_file()
        .returning(|_| true);
    
    mock_file_system
        .expect_read_file_content()
        .returning(|_| Ok("Test file content".to_string()));
    
    mock_strategy
        .expect_index_document()
        .returning(|_, _, _| Ok(()));
    
    mock_progress_tracker
        .expect_file_processed()
        .returning(|_, _| {});
    
    let service = IndexingService::new(
        Arc::new(mock_vector_storage),
        Arc::new(mock_embedding_service),
        Arc::new(mock_file_system),
        Arc::new(mock_filtering),
        Arc::new(mock_progress_tracker),
        Arc::new(mock_collection_manager),
        Arc::new(mock_strategy),
        ContentProcessor,
    );
    
    let result = service.index_file(Path::new("test.txt"), "test_collection").await;
    assert!(result.is_ok(), "File indexing should succeed");
    assert_eq!(result.unwrap(), true, "Should return true for successful indexing");
}

#[tokio::test]
async fn test_indexing_service_index_file_filtered_out() {
    // Test that filtered files are skipped
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    let mut mock_file_system = MockFileSystemService::new();
    let mut mock_filtering = MockFilteringService::new();
    let mut mock_progress_tracker = MockProgressTracker::new();
    let mut mock_collection_manager = MockCollectionManager::new();
    let mut mock_strategy = MockIndexingStrategy::new();
    
    // Set up expectations - file should be filtered out
    mock_file_system
        .expect_exists()
        .returning(|_| Ok(true));
    
    mock_file_system
        .expect_is_file()
        .returning(|_| Ok(true));
    
    mock_filtering
        .expect_should_index_file()
        .returning(|_| false); // File is filtered out
    
    let service = IndexingService::new(
        Arc::new(mock_vector_storage),
        Arc::new(mock_embedding_service),
        Arc::new(mock_file_system),
        Arc::new(mock_filtering),
        Arc::new(mock_progress_tracker),
        Arc::new(mock_collection_manager),
        Arc::new(mock_strategy),
        ContentProcessor,
    );
    
    let result = service.index_file(Path::new("filtered.txt"), "test_collection").await;
    assert!(result.is_ok(), "Should not error for filtered files");
    assert_eq!(result.unwrap(), false, "Should return false for filtered files");
}

#[tokio::test]
async fn test_indexing_service_index_document() {
    // Test direct document indexing
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    let mut mock_file_system = MockFileSystemService::new();
    let mut mock_filtering = MockFilteringService::new();
    let mut mock_progress_tracker = MockProgressTracker::new();
    let mut mock_collection_manager = MockCollectionManager::new();
    let mut mock_strategy = MockIndexingStrategy::new();
    
    // Set up expectations
    mock_strategy
        .expect_index_document()
        .returning(|_, _, _| Ok(()));
    
    let service = IndexingService::new(
        Arc::new(mock_vector_storage),
        Arc::new(mock_embedding_service),
        Arc::new(mock_file_system),
        Arc::new(mock_filtering),
        Arc::new(mock_progress_tracker),
        Arc::new(mock_collection_manager),
        Arc::new(mock_strategy),
        ContentProcessor,
    );
    
    let document = Document {
        id: "test_doc".to_string(),
        content: "Test document content".to_string(),
        metadata: std::collections::HashMap::new(),
        source: Some("test.txt".to_string()),
    };
    
    let result = service.index_document(document, "test_collection").await;
    assert!(result.is_ok(), "Document indexing should succeed");
}

#[tokio::test]
async fn test_indexing_service_get_collections() {
    // Test collection listing
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    let mut mock_file_system = MockFileSystemService::new();
    let mut mock_filtering = MockFilteringService::new();
    let mut mock_progress_tracker = MockProgressTracker::new();
    let mut mock_collection_manager = MockCollectionManager::new();
    let mut mock_strategy = MockIndexingStrategy::new();
    
    // Set up expectations
    mock_collection_manager
        .expect_list_collections()
        .returning(|| Ok(vec!["collection1".to_string(), "collection2".to_string()]));
    
    let service = IndexingService::new(
        Arc::new(mock_vector_storage),
        Arc::new(mock_embedding_service),
        Arc::new(mock_file_system),
        Arc::new(mock_filtering),
        Arc::new(mock_progress_tracker),
        Arc::new(mock_collection_manager),
        Arc::new(mock_strategy),
        ContentProcessor,
    );
    
    let result = service.list_collections().await;
    assert!(result.is_ok(), "Collection listing should succeed");
    let collections = result.unwrap();
    assert_eq!(collections.len(), 2, "Should return expected number of collections");
    assert!(collections.contains(&"collection1".to_string()));
    assert!(collections.contains(&"collection2".to_string()));
}

#[tokio::test]
async fn test_indexing_service_get_progress() {
    // Test progress tracking
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    let mut mock_file_system = MockFileSystemService::new();
    let mut mock_filtering = MockFilteringService::new();
    let mut mock_progress_tracker = MockProgressTracker::new();
    let mut mock_collection_manager = MockCollectionManager::new();
    let mut mock_strategy = MockIndexingStrategy::new();
    
    // Set up expectations
    mock_progress_tracker
        .expect_get_progress()
        .returning(|| Ok(ProgressStats {
            files_processed: 10,
            files_succeeded: 8,
            files_failed: 2,
            total_files: 15,
            elapsed_ms: 5000,
            current_file: Some("current.txt".to_string()),
        }));
    
    let service = IndexingService::new(
        Arc::new(mock_vector_storage),
        Arc::new(mock_embedding_service),
        Arc::new(mock_file_system),
        Arc::new(mock_filtering),
        Arc::new(mock_progress_tracker),
        Arc::new(mock_collection_manager),
        Arc::new(mock_strategy),
        ContentProcessor,
    );
    
    let result = service.get_progress().await;
    assert!(result.is_ok(), "Progress retrieval should succeed");
    let progress = result.unwrap();
    assert_eq!(progress.files_processed, 10);
    assert_eq!(progress.files_succeeded, 8);
    assert_eq!(progress.files_failed, 2);
}

// Integration test to ensure all components work together
#[tokio::test]
async fn test_indexing_service_integration() {
    // This test demonstrates that all mocked dependencies work together properly
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    let mut mock_file_system = MockFileSystemService::new();
    let mut mock_filtering = MockFilteringService::new();
    let mut mock_progress_tracker = MockProgressTracker::new();
    let mut mock_collection_manager = MockCollectionManager::new();
    let mut mock_strategy = MockIndexingStrategy::new();
    
    // Set up a complete workflow
    mock_file_system
        .expect_exists()
        .returning(|_| Ok(true));
    
    mock_file_system
        .expect_is_file()
        .returning(|_| Ok(true));
    
    mock_filtering
        .expect_should_index_file()
        .returning(|_| true);
    
    mock_file_system
        .expect_read_file_content()
        .returning(|_| Ok("Integration test content".to_string()));
    
    mock_strategy
        .expect_index_document()
        .returning(|_, _, _| Ok(()));
    
    mock_progress_tracker
        .expect_file_processed()
        .returning(|_, _| {});
    
    mock_collection_manager
        .expect_list_collections()
        .returning(|| Ok(vec!["integration_test".to_string()]));
    
    let service = IndexingService::new(
        Arc::new(mock_vector_storage),
        Arc::new(mock_embedding_service),
        Arc::new(mock_file_system),
        Arc::new(mock_filtering),
        Arc::new(mock_progress_tracker),
        Arc::new(mock_collection_manager),
        Arc::new(mock_strategy),
        ContentProcessor,
    );
    
    // Test file indexing
    let index_result = service.index_file(Path::new("integration.txt"), "integration_test").await;
    assert!(index_result.is_ok(), "Integration file indexing should succeed");
    
    // Test collection listing
    let collections_result = service.list_collections().await;
    assert!(collections_result.is_ok(), "Integration collection listing should succeed");
    let collections = collections_result.unwrap();
    assert!(collections.contains(&"integration_test".to_string()));
}
