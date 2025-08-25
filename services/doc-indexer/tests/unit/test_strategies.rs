// Unit tests for Strategy Pattern implementations
// Tests all indexing strategies and their behavior

use std::sync::Arc;
use std::collections::HashMap;
use tokio;
use mockall::mock;

use doc_indexer::application::indexing_strategies::*;
use doc_indexer::application::interfaces::*;
use doc_indexer::domain::Document;
use zero_latency_core::Result;

// Mock implementations for strategy testing
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

#[tokio::test]
async fn test_standard_indexing_strategy_creation() {
    // Test that StandardIndexingStrategy can be created with default and custom config
    let strategy = StandardIndexingStrategy::new();
    let config = strategy.get_config();
    assert_eq!(config.name, "Standard");
    
    let custom_config = StrategyConfig {
        name: "CustomStandard".to_string(),
        // Add other config fields as they're defined
    };
    let custom_strategy = StandardIndexingStrategy::with_config(custom_config);
    let custom_config_result = custom_strategy.get_config();
    assert_eq!(custom_config_result.name, "CustomStandard");
}

#[tokio::test]
async fn test_standard_indexing_strategy_index_document() {
    // Test that StandardIndexingStrategy properly indexes a document
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    
    // Set up expectations
    mock_embedding_service
        .expect_generate_embedding()
        .with(mockall::predicate::eq("Test document content"))
        .returning(|_| Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5]));
    
    mock_vector_storage
        .expect_store_vectors()
        .with(
            mockall::predicate::eq("test_doc"),
            mockall::predicate::eq(vec![0.1, 0.2, 0.3, 0.4, 0.5])
        )
        .returning(|_, _| Ok(()));
    
    let strategy = StandardIndexingStrategy::new();
    
    let document = Document {
        id: "test_doc".to_string(),
        content: "Test document content".to_string(),
        metadata: HashMap::new(),
        source: Some("test.txt".to_string()),
    };
    
    let result = strategy.index_document(
        &document,
        &*mock_vector_storage,
        &*mock_embedding_service
    ).await;
    
    assert!(result.is_ok(), "StandardIndexingStrategy should successfully index document");
}

#[tokio::test]
async fn test_fast_indexing_strategy_creation() {
    // Test that FastIndexingStrategy can be created
    let strategy = FastIndexingStrategy::new();
    let config = strategy.get_config();
    assert_eq!(config.name, "Fast");
}

#[tokio::test]
async fn test_fast_indexing_strategy_index_document() {
    // Test that FastIndexingStrategy properly indexes a document with optimizations
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    
    // Set up expectations - Fast strategy might do batch processing or optimization
    mock_embedding_service
        .expect_generate_embedding()
        .returning(|_| Ok(vec![0.1, 0.2, 0.3])); // Shorter vector for fast processing
    
    mock_vector_storage
        .expect_store_vectors()
        .returning(|_, _| Ok(()));
    
    let strategy = FastIndexingStrategy::new();
    
    let document = Document {
        id: "fast_test_doc".to_string(),
        content: "Fast indexing test content".to_string(),
        metadata: HashMap::new(),
        source: Some("fast_test.txt".to_string()),
    };
    
    let result = strategy.index_document(
        &document,
        &*mock_vector_storage,
        &*mock_embedding_service
    ).await;
    
    assert!(result.is_ok(), "FastIndexingStrategy should successfully index document");
}

#[tokio::test]
async fn test_precision_indexing_strategy_creation() {
    // Test that PrecisionIndexingStrategy can be created
    let strategy = PrecisionIndexingStrategy::new();
    let config = strategy.get_config();
    assert_eq!(config.name, "Precision");
}

#[tokio::test]
async fn test_precision_indexing_strategy_index_document() {
    // Test that PrecisionIndexingStrategy properly indexes with high precision
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    
    // Set up expectations - Precision strategy might do more thorough processing
    mock_embedding_service
        .expect_generate_embedding()
        .returning(|_| Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8])); // Longer, more precise vector
    
    mock_vector_storage
        .expect_store_vectors()
        .returning(|_, _| Ok(()));
    
    let strategy = PrecisionIndexingStrategy::new();
    
    let document = Document {
        id: "precision_test_doc".to_string(),
        content: "Precision indexing test content with detailed analysis".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("precision_mode".to_string(), "high".to_string());
            meta
        },
        source: Some("precision_test.txt".to_string()),
    };
    
    let result = strategy.index_document(
        &document,
        &*mock_vector_storage,
        &*mock_embedding_service
    ).await;
    
    assert!(result.is_ok(), "PrecisionIndexingStrategy should successfully index document");
}

#[tokio::test]
async fn test_strategy_config_differences() {
    // Test that different strategies have different configurations
    let standard_strategy = StandardIndexingStrategy::new();
    let fast_strategy = FastIndexingStrategy::new();
    let precision_strategy = PrecisionIndexingStrategy::new();
    
    let standard_config = standard_strategy.get_config();
    let fast_config = fast_strategy.get_config();
    let precision_config = precision_strategy.get_config();
    
    // Ensure each strategy has a unique name
    assert_ne!(standard_config.name, fast_config.name);
    assert_ne!(standard_config.name, precision_config.name);
    assert_ne!(fast_config.name, precision_config.name);
    
    // Verify specific strategy names
    assert_eq!(standard_config.name, "Standard");
    assert_eq!(fast_config.name, "Fast");
    assert_eq!(precision_config.name, "Precision");
}

#[tokio::test]
async fn test_strategy_error_handling() {
    // Test that strategies properly handle errors from dependencies
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    
    // Set up expectations for error scenarios
    mock_embedding_service
        .expect_generate_embedding()
        .returning(|_| Err(zero_latency_core::Error::Other("Embedding generation failed".into())));
    
    let strategy = StandardIndexingStrategy::new();
    
    let document = Document {
        id: "error_test_doc".to_string(),
        content: "This document should fail to index".to_string(),
        metadata: HashMap::new(),
        source: Some("error_test.txt".to_string()),
    };
    
    let result = strategy.index_document(
        &document,
        &*mock_vector_storage,
        &*mock_embedding_service
    ).await;
    
    assert!(result.is_err(), "Strategy should propagate embedding generation errors");
}

#[tokio::test]
async fn test_strategy_interchangeability() {
    // Test that strategies can be used interchangeably (Liskov Substitution Principle)
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    
    // Set up expectations that work for any strategy
    mock_embedding_service
        .expect_generate_embedding()
        .returning(|_| Ok(vec![0.1, 0.2, 0.3, 0.4]));
    
    mock_vector_storage
        .expect_store_vectors()
        .returning(|_, _| Ok(()));
    
    let document = Document {
        id: "interchangeable_test".to_string(),
        content: "Test content for strategy interchangeability".to_string(),
        metadata: HashMap::new(),
        source: Some("interchangeable.txt".to_string()),
    };
    
    // Test with different strategies using the same interface
    let strategies: Vec<Box<dyn IndexingStrategy>> = vec![
        Box::new(StandardIndexingStrategy::new()),
        Box::new(FastIndexingStrategy::new()),
        Box::new(PrecisionIndexingStrategy::new()),
    ];
    
    for strategy in strategies {
        let result = strategy.index_document(
            &document,
            &*mock_vector_storage,
            &*mock_embedding_service
        ).await;
        
        assert!(result.is_ok(), "All strategies should work interchangeably through the same interface");
    }
}

#[tokio::test]
async fn test_strategy_document_variations() {
    // Test strategies with different document types and content sizes
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    
    mock_embedding_service
        .expect_generate_embedding()
        .returning(|_| Ok(vec![0.1, 0.2, 0.3]));
    
    mock_vector_storage
        .expect_store_vectors()
        .returning(|_, _| Ok(()));
    
    let strategy = StandardIndexingStrategy::new();
    
    // Test with empty document
    let empty_document = Document {
        id: "empty_doc".to_string(),
        content: "".to_string(),
        metadata: HashMap::new(),
        source: None,
    };
    
    let result = strategy.index_document(
        &empty_document,
        &*mock_vector_storage,
        &*mock_embedding_service
    ).await;
    
    assert!(result.is_ok(), "Strategy should handle empty documents");
    
    // Test with large document
    let large_content = "Large document content. ".repeat(1000);
    let large_document = Document {
        id: "large_doc".to_string(),
        content: large_content,
        metadata: HashMap::new(),
        source: Some("large_file.txt".to_string()),
    };
    
    let result = strategy.index_document(
        &large_document,
        &*mock_vector_storage,
        &*mock_embedding_service
    ).await;
    
    assert!(result.is_ok(), "Strategy should handle large documents");
}

// Test that demonstrates the Open-Closed Principle
// New strategies can be added without modifying existing code
struct CustomTestStrategy {
    config: StrategyConfig,
}

impl CustomTestStrategy {
    fn new() -> Self {
        Self {
            config: StrategyConfig {
                name: "CustomTest".to_string(),
            },
        }
    }
}

#[async_trait::async_trait]
impl IndexingStrategy for CustomTestStrategy {
    async fn index_document(
        &self,
        document: &Document,
        vector_storage: &dyn VectorStorage,
        embedding_service: &dyn EmbeddingService,
    ) -> Result<()> {
        // Custom indexing logic here
        let embedding = embedding_service.generate_embedding(&document.content).await?;
        vector_storage.store_vectors(&document.id, embedding).await?;
        Ok(())
    }
    
    fn get_config(&self) -> StrategyConfig {
        self.config.clone()
    }
}

#[tokio::test]
async fn test_custom_strategy_extensibility() {
    // Test that new strategies can be added without modifying existing code (OCP)
    let mut mock_vector_storage = MockVectorStorage::new();
    let mut mock_embedding_service = MockEmbeddingService::new();
    
    mock_embedding_service
        .expect_generate_embedding()
        .returning(|_| Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5]));
    
    mock_vector_storage
        .expect_store_vectors()
        .returning(|_, _| Ok(()));
    
    let custom_strategy = CustomTestStrategy::new();
    assert_eq!(custom_strategy.get_config().name, "CustomTest");
    
    let document = Document {
        id: "custom_test_doc".to_string(),
        content: "Custom strategy test content".to_string(),
        metadata: HashMap::new(),
        source: Some("custom_test.txt".to_string()),
    };
    
    let result = custom_strategy.index_document(
        &document,
        &*mock_vector_storage,
        &*mock_embedding_service
    ).await;
    
    assert!(result.is_ok(), "Custom strategy should work through the same interface");
}
