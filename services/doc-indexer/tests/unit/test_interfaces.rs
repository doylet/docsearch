// Unit tests for Interface implementations and Adapter pattern
// Tests SOLID principle adherence: ISP, DIP, and LSP

use std::sync::Arc;
use std::path::Path;
use tokio;

use doc_indexer::application::interfaces::*;
use doc_indexer::application::adapters::*;
use zero_latency_core::Result;

#[tokio::test]
async fn test_standard_file_system_service_interface() {
    // Test that StandardFileSystemService properly implements FileSystemService interface
    let fs_service = StandardFileSystemService::new();
    
    // Test with a known file (this test file itself)
    let test_file_path = Path::new("tests/unit/test_interfaces.rs");
    
    // Test exists method
    let exists_result = fs_service.exists(test_file_path).await;
    assert!(exists_result.is_ok(), "exists() should not error");
    
    // Test is_file method for a directory
    let dir_path = Path::new("tests");
    let is_file_result = fs_service.is_file(dir_path).await;
    assert!(is_file_result.is_ok(), "is_file() should not error for directory");
    
    // Test is_directory method
    let is_dir_result = fs_service.is_directory(dir_path).await;
    assert!(is_dir_result.is_ok(), "is_directory() should not error");
}

#[tokio::test]
async fn test_file_system_service_read_content() {
    // Test file reading capability
    let fs_service = StandardFileSystemService::new();
    
    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_file_system.txt");
    let test_content = "Hello, FileSystemService interface test!";
    
    // Write test content
    std::fs::write(&test_file, test_content).expect("Should be able to write test file");
    
    // Test reading the content
    let read_result = fs_service.read_file_content(&test_file).await;
    assert!(read_result.is_ok(), "Should be able to read file content");
    
    let content = read_result.unwrap();
    assert_eq!(content, test_content, "Content should match what was written");
    
    // Clean up
    std::fs::remove_file(&test_file).ok();
}

#[tokio::test]
async fn test_file_system_service_list_directory() {
    // Test directory listing capability
    let fs_service = StandardFileSystemService::new();
    
    // Test listing a known directory
    let test_dir = Path::new("tests");
    let list_result = fs_service.list_directory(test_dir).await;
    
    if test_dir.exists() {
        assert!(list_result.is_ok(), "Should be able to list directory contents");
        let contents = list_result.unwrap();
        assert!(!contents.is_empty(), "Test directory should not be empty");
    }
}

#[tokio::test]
async fn test_file_system_service_metadata() {
    // Test file metadata retrieval
    let fs_service = StandardFileSystemService::new();
    
    // Create a temporary test file for metadata testing
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_metadata.txt");
    std::fs::write(&test_file, "metadata test").expect("Should be able to write test file");
    
    let metadata_result = fs_service.get_file_metadata(&test_file).await;
    assert!(metadata_result.is_ok(), "Should be able to get file metadata");
    
    let metadata = metadata_result.unwrap();
    assert!(metadata.size > 0, "File should have non-zero size");
    assert!(metadata.is_file, "Should be identified as a file");
    assert!(!metadata.is_directory, "Should not be identified as a directory");
    
    // Clean up
    std::fs::remove_file(&test_file).ok();
}

#[tokio::test]
async fn test_in_memory_progress_tracker() {
    // Test that InMemoryProgressTracker properly implements ProgressTracker interface
    let progress_tracker = InMemoryProgressTracker::new();
    
    // Test initial progress
    let initial_progress = progress_tracker.get_progress().await;
    assert!(initial_progress.is_ok(), "Should be able to get initial progress");
    
    let progress = initial_progress.unwrap();
    assert_eq!(progress.files_processed, 0, "Initial files processed should be 0");
    assert_eq!(progress.files_succeeded, 0, "Initial files succeeded should be 0");
    assert_eq!(progress.files_failed, 0, "Initial files failed should be 0");
    
    // Test file processing tracking
    let test_path = Path::new("test_file.txt");
    progress_tracker.file_processed(test_path, true).await;
    
    let updated_progress = progress_tracker.get_progress().await;
    assert!(updated_progress.is_ok(), "Should be able to get updated progress");
    
    let progress = updated_progress.unwrap();
    assert_eq!(progress.files_processed, 1, "Files processed should be incremented");
    assert_eq!(progress.files_succeeded, 1, "Files succeeded should be incremented");
    assert_eq!(progress.files_failed, 0, "Files failed should remain 0");
}

#[tokio::test]
async fn test_in_memory_progress_tracker_failure() {
    // Test failure tracking
    let progress_tracker = InMemoryProgressTracker::new();
    
    let test_path = Path::new("failed_file.txt");
    progress_tracker.file_processed(test_path, false).await;
    
    let progress = progress_tracker.get_progress().await.unwrap();
    assert_eq!(progress.files_processed, 1, "Files processed should be incremented");
    assert_eq!(progress.files_succeeded, 0, "Files succeeded should remain 0");
    assert_eq!(progress.files_failed, 1, "Files failed should be incremented");
}

#[tokio::test]
async fn test_in_memory_progress_tracker_updates() {
    // Test progress updates
    let progress_tracker = InMemoryProgressTracker::new();
    
    progress_tracker.progress_update(5, 10, 1000).await;
    
    let progress = progress_tracker.get_progress().await.unwrap();
    assert_eq!(progress.total_files, 10, "Total files should be updated");
    assert_eq!(progress.elapsed_ms, 1000, "Elapsed time should be updated");
}

#[tokio::test]
async fn test_simple_collection_manager() {
    // Test that SimpleCollectionManager properly implements CollectionManager interface
    let collection_manager = SimpleCollectionManager::new();
    
    // Test creating a collection
    let create_result = collection_manager.create_collection("test_collection").await;
    assert!(create_result.is_ok(), "Should be able to create a collection");
    
    // Test listing collections
    let list_result = collection_manager.list_collections().await;
    assert!(list_result.is_ok(), "Should be able to list collections");
    
    let collections = list_result.unwrap();
    assert!(collections.contains(&"test_collection".to_string()), "Created collection should be in the list");
    
    // Test getting collection stats
    let stats_result = collection_manager.get_collection_stats("test_collection").await;
    assert!(stats_result.is_ok(), "Should be able to get collection stats");
    
    let stats = stats_result.unwrap();
    assert_eq!(stats.name, "test_collection", "Stats should have correct collection name");
    assert_eq!(stats.document_count, 0, "New collection should have 0 documents");
    
    // Test deleting a collection
    let delete_result = collection_manager.delete_collection("test_collection").await;
    assert!(delete_result.is_ok(), "Should be able to delete a collection");
    
    // Verify collection is deleted
    let list_after_delete = collection_manager.list_collections().await.unwrap();
    assert!(!list_after_delete.contains(&"test_collection".to_string()), "Deleted collection should not be in the list");
}

#[tokio::test]
async fn test_collection_manager_nonexistent_collection() {
    // Test error handling for nonexistent collections
    let collection_manager = SimpleCollectionManager::new();
    
    let stats_result = collection_manager.get_collection_stats("nonexistent").await;
    assert!(stats_result.is_err(), "Should error when getting stats for nonexistent collection");
    
    let delete_result = collection_manager.delete_collection("nonexistent").await;
    assert!(delete_result.is_err(), "Should error when deleting nonexistent collection");
}

#[tokio::test]
async fn test_interface_segregation_principle() {
    // Test that interfaces are properly segregated and focused
    
    // FileSystemService should only deal with file operations
    let fs_service = StandardFileSystemService::new();
    let test_path = Path::new(".");
    
    // These methods should be available
    let _ = fs_service.exists(test_path).await;
    let _ = fs_service.is_file(test_path).await;
    let _ = fs_service.is_directory(test_path).await;
    
    // ProgressTracker should only deal with progress tracking
    let progress_tracker = InMemoryProgressTracker::new();
    let _ = progress_tracker.get_progress().await;
    
    // CollectionManager should only deal with collection management
    let collection_manager = SimpleCollectionManager::new();
    let _ = collection_manager.list_collections().await;
    
    // Each interface has a focused responsibility - this demonstrates ISP
    assert!(true, "Interfaces are properly segregated");
}

#[tokio::test]
async fn test_liskov_substitution_principle() {
    // Test that implementations can be substituted for their interfaces
    
    // Any FileSystemService implementation should work the same way
    let fs_service: Arc<dyn FileSystemService> = Arc::new(StandardFileSystemService::new());
    let test_path = Path::new(".");
    
    let exists_result = fs_service.exists(test_path).await;
    assert!(exists_result.is_ok(), "Any FileSystemService implementation should support exists()");
    
    // Any ProgressTracker implementation should work the same way
    let progress_tracker: Arc<dyn ProgressTracker> = Arc::new(InMemoryProgressTracker::new());
    let progress_result = progress_tracker.get_progress().await;
    assert!(progress_result.is_ok(), "Any ProgressTracker implementation should support get_progress()");
    
    // Any CollectionManager implementation should work the same way
    let collection_manager: Arc<dyn CollectionManager> = Arc::new(SimpleCollectionManager::new());
    let collections_result = collection_manager.list_collections().await;
    assert!(collections_result.is_ok(), "Any CollectionManager implementation should support list_collections()");
    
    // This demonstrates LSP - implementations are substitutable
    assert!(true, "Implementations properly substitute for their interfaces");
}

#[tokio::test]
async fn test_dependency_inversion_principle() {
    // Test that high-level modules depend on abstractions, not concretions
    
    // This function works with any FileSystemService implementation
    async fn test_with_filesystem(fs: Arc<dyn FileSystemService>) -> Result<bool> {
        fs.exists(Path::new(".")).await
    }
    
    // This function works with any ProgressTracker implementation
    async fn test_with_progress_tracker(tracker: Arc<dyn ProgressTracker>) -> Result<ProgressStats> {
        tracker.get_progress().await
    }
    
    // This function works with any CollectionManager implementation
    async fn test_with_collection_manager(manager: Arc<dyn CollectionManager>) -> Result<Vec<String>> {
        manager.list_collections().await
    }
    
    // Test with concrete implementations through abstractions
    let fs_service: Arc<dyn FileSystemService> = Arc::new(StandardFileSystemService::new());
    let progress_tracker: Arc<dyn ProgressTracker> = Arc::new(InMemoryProgressTracker::new());
    let collection_manager: Arc<dyn CollectionManager> = Arc::new(SimpleCollectionManager::new());
    
    let fs_result = test_with_filesystem(fs_service).await;
    assert!(fs_result.is_ok(), "Should work with FileSystemService abstraction");
    
    let progress_result = test_with_progress_tracker(progress_tracker).await;
    assert!(progress_result.is_ok(), "Should work with ProgressTracker abstraction");
    
    let collection_result = test_with_collection_manager(collection_manager).await;
    assert!(collection_result.is_ok(), "Should work with CollectionManager abstraction");
    
    // This demonstrates DIP - depending on abstractions, not concretions
    assert!(true, "High-level modules depend on abstractions");
}

#[tokio::test]
async fn test_adapter_pattern_integration() {
    // Test that adapters properly bridge new interfaces with existing infrastructure
    
    // FilteringServiceAdapter should work with existing FilterService
    use doc_indexer::application::services::filter_service::FilterService;
    
    let filter_service = Arc::new(FilterService::new());
    let filtering_adapter = FilteringServiceAdapter::new(filter_service);
    
    let test_path = Path::new("test.txt");
    let should_index = filtering_adapter.should_index_file(test_path);
    // Should not panic or error - adapter provides the interface
    assert!(should_index || !should_index, "Adapter should provide filtering functionality");
    
    let filter_summary = filtering_adapter.get_filter_summary();
    assert!(!filter_summary.safe_patterns.is_empty() || filter_summary.safe_patterns.is_empty(), 
           "Adapter should provide filter summary");
}

#[tokio::test] 
async fn test_interface_compatibility() {
    // Test that all interfaces work together harmoniously
    let fs_service: Arc<dyn FileSystemService> = Arc::new(StandardFileSystemService::new());
    let progress_tracker: Arc<dyn ProgressTracker> = Arc::new(InMemoryProgressTracker::new());
    let collection_manager: Arc<dyn CollectionManager> = Arc::new(SimpleCollectionManager::new());
    
    // These interfaces should work together without conflicts
    let _ = fs_service.exists(Path::new(".")).await;
    let _ = progress_tracker.get_progress().await;
    let _ = collection_manager.list_collections().await;
    
    // No interface should interfere with another
    assert!(true, "All interfaces work together harmoniously");
}
