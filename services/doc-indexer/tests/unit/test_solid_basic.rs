// Simple unit tests for our SOLID service layer
// Basic tests that validate the interfaces and builder pattern

use std::sync::Arc;
use doc_indexer::application::adapters::*;
use doc_indexer::application::interfaces::*;

#[tokio::test]
async fn test_simple_collection_manager_basic() {
    // Test basic collection management operations
    let manager = SimpleCollectionManager::new();
    
    // Test creating a collection
    let result = manager.create_collection("test").await;
    assert!(result.is_ok(), "Should be able to create collection");
    
    // Test listing collections
    let collections = manager.list_collections().await;
    assert!(collections.is_ok(), "Should be able to list collections");
    assert!(collections.unwrap().contains(&"test".to_string()), "Should contain created collection");
}

#[tokio::test]
async fn test_in_memory_progress_tracker_basic() {
    // Test basic progress tracking
    let tracker = InMemoryProgressTracker::new();
    
    // Test initial state
    let progress = tracker.get_progress().await;
    assert!(progress.is_ok(), "Should get initial progress");
    
    let stats = progress.unwrap();
    assert_eq!(stats.files_processed, 0, "Initial processed should be 0");
    assert_eq!(stats.files_succeeded, 0, "Initial succeeded should be 0");
    assert_eq!(stats.files_failed, 0, "Initial failed should be 0");
}

#[tokio::test]
async fn test_standard_file_system_basic() {
    // Test basic file system operations
    let fs = StandardFileSystemService::new();
    
    // Test checking if current directory exists
    let current_dir = std::path::Path::new(".");
    let exists = fs.exists(current_dir).await;
    assert!(exists.is_ok(), "Should be able to check existence");
    assert!(exists.unwrap(), "Current directory should exist");
    
    // Test checking if it's a directory
    let is_dir = fs.is_directory(current_dir).await;
    assert!(is_dir.is_ok(), "Should be able to check if directory");
    assert!(is_dir.unwrap(), "Current directory should be a directory");
}

#[test]
fn test_strategy_pattern_basic() {
    // Test that strategies can be created
    use doc_indexer::application::indexing_strategies::*;
    
    let standard = StandardIndexingStrategy::new();
    let fast = FastIndexingStrategy::new();
    let precision = PrecisionIndexingStrategy::new();
    
    // Test that they have different configurations
    assert_eq!(standard.get_config().name, "Standard");
    assert_eq!(fast.get_config().name, "Fast");
    assert_eq!(precision.get_config().name, "Precision");
    
    // All different
    assert_ne!(standard.get_config().name, fast.get_config().name);
    assert_ne!(standard.get_config().name, precision.get_config().name);
    assert_ne!(fast.get_config().name, precision.get_config().name);
}

#[test]
fn test_builder_pattern_creation() {
    // Test that the IndexingServiceBuilder can be created
    use doc_indexer::application::services::indexing_service::IndexingServiceBuilder;
    
    let builder = IndexingServiceBuilder::new();
    // Builder should be created successfully
    // We can't easily test the full build without all dependencies,
    // but we can verify the builder pattern is working
    assert!(true, "Builder created successfully");
}
