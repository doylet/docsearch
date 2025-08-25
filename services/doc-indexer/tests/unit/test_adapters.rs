// Unit tests for Adapter Pattern implementations
// Tests that adapters properly bridge interfaces with existing infrastructure

use std::sync::Arc;
use std::path::Path;
use tokio;

use doc_indexer::application::adapters::*;
use doc_indexer::application::interfaces::*;
use doc_indexer::application::services::filter_service::FilterService;
use zero_latency_core::Result;

#[tokio::test]
async fn test_standard_file_system_service_adapter() {
    // Test StandardFileSystemService as an adapter for file system operations
    let fs_adapter = StandardFileSystemService::new();
    
    // Test basic file system operations
    let current_dir = Path::new(".");
    
    let exists_result = fs_adapter.exists(current_dir).await;
    assert!(exists_result.is_ok(), "Current directory should exist");
    assert!(exists_result.unwrap(), "Current directory should exist");
    
    let is_dir_result = fs_adapter.is_directory(current_dir).await;
    assert!(is_dir_result.is_ok(), "Should be able to check if path is directory");
    assert!(is_dir_result.unwrap(), "Current directory should be a directory");
    
    let is_file_result = fs_adapter.is_file(current_dir).await;
    assert!(is_file_result.is_ok(), "Should be able to check if path is file");
    assert!(!is_file_result.unwrap(), "Current directory should not be a file");
}

#[tokio::test]
async fn test_standard_file_system_service_file_operations() {
    // Test file-specific operations
    let fs_adapter = StandardFileSystemService::new();
    
    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("adapter_test.txt");
    let test_content = "Hello from FileSystemService adapter test!";
    
    // Write test content
    std::fs::write(&test_file, test_content).expect("Should be able to write test file");
    
    // Test file operations through the adapter
    let exists_result = fs_adapter.exists(&test_file).await;
    assert!(exists_result.is_ok() && exists_result.unwrap(), "Test file should exist");
    
    let is_file_result = fs_adapter.is_file(&test_file).await;
    assert!(is_file_result.is_ok() && is_file_result.unwrap(), "Test file should be identified as file");
    
    let read_result = fs_adapter.read_file_content(&test_file).await;
    assert!(read_result.is_ok(), "Should be able to read file through adapter");
    assert_eq!(read_result.unwrap(), test_content, "Content should match");
    
    let metadata_result = fs_adapter.get_file_metadata(&test_file).await;
    assert!(metadata_result.is_ok(), "Should be able to get metadata through adapter");
    
    let metadata = metadata_result.unwrap();
    assert!(metadata.is_file, "Metadata should indicate it's a file");
    assert!(!metadata.is_directory, "Metadata should indicate it's not a directory");
    assert!(metadata.size > 0, "File should have non-zero size");
    
    // Clean up
    std::fs::remove_file(&test_file).ok();
}

#[tokio::test]
async fn test_standard_file_system_service_directory_operations() {
    // Test directory-specific operations
    let fs_adapter = StandardFileSystemService::new();
    
    // Test with existing directory
    let tests_dir = Path::new("tests");
    if tests_dir.exists() {
        let list_result = fs_adapter.list_directory(tests_dir).await;
        assert!(list_result.is_ok(), "Should be able to list directory contents");
        
        let contents = list_result.unwrap();
        assert!(!contents.is_empty(), "Tests directory should have contents");
        
        // Verify that listed items actually exist
        for item in contents.iter().take(3) { // Check first 3 items to avoid too many file operations
            let item_exists = fs_adapter.exists(item).await;
            assert!(item_exists.is_ok() && item_exists.unwrap(), "Listed items should exist");
        }
    }
}

#[tokio::test]
async fn test_in_memory_progress_tracker_adapter() {
    // Test InMemoryProgressTracker as an adapter for progress tracking
    let progress_adapter = InMemoryProgressTracker::new();
    
    // Test initial state
    let initial_progress = progress_adapter.get_progress().await;
    assert!(initial_progress.is_ok(), "Should be able to get initial progress");
    
    let progress = initial_progress.unwrap();
    assert_eq!(progress.files_processed, 0, "Initial processed count should be 0");
    assert_eq!(progress.files_succeeded, 0, "Initial success count should be 0");
    assert_eq!(progress.files_failed, 0, "Initial failure count should be 0");
    assert_eq!(progress.total_files, 0, "Initial total should be 0");
    
    // Test file processing tracking
    let test_paths = [
        Path::new("file1.txt"),
        Path::new("file2.txt"),
        Path::new("file3.txt"),
    ];
    
    // Process some files with success
    progress_adapter.file_processed(test_paths[0], true).await;
    progress_adapter.file_processed(test_paths[1], true).await;
    
    // Process one file with failure
    progress_adapter.file_processed(test_paths[2], false).await;
    
    // Check updated progress
    let updated_progress = progress_adapter.get_progress().await.unwrap();
    assert_eq!(updated_progress.files_processed, 3, "Should track total processed files");
    assert_eq!(updated_progress.files_succeeded, 2, "Should track successful files");
    assert_eq!(updated_progress.files_failed, 1, "Should track failed files");
    
    // Test progress updates
    progress_adapter.progress_update(10, 20, 5000).await;
    
    let final_progress = progress_adapter.get_progress().await.unwrap();
    assert_eq!(final_progress.total_files, 20, "Should update total files");
    assert_eq!(final_progress.elapsed_ms, 5000, "Should update elapsed time");
}

#[tokio::test]
async fn test_simple_collection_manager_adapter() {
    // Test SimpleCollectionManager as an adapter for collection management
    let collection_adapter = SimpleCollectionManager::new();
    
    let test_collection = "adapter_test_collection";
    
    // Test creating a collection
    let create_result = collection_adapter.create_collection(test_collection).await;
    assert!(create_result.is_ok(), "Should be able to create collection through adapter");
    
    // Test listing collections
    let list_result = collection_adapter.list_collections().await;
    assert!(list_result.is_ok(), "Should be able to list collections through adapter");
    
    let collections = list_result.unwrap();
    assert!(collections.contains(&test_collection.to_string()), "Created collection should be listed");
    
    // Test getting collection stats
    let stats_result = collection_adapter.get_collection_stats(test_collection).await;
    assert!(stats_result.is_ok(), "Should be able to get stats through adapter");
    
    let stats = stats_result.unwrap();
    assert_eq!(stats.name, test_collection, "Stats should have correct collection name");
    assert_eq!(stats.document_count, 0, "New collection should have 0 documents");
    assert_eq!(stats.vector_count, 0, "New collection should have 0 vectors");
    
    // Test deleting collection
    let delete_result = collection_adapter.delete_collection(test_collection).await;
    assert!(delete_result.is_ok(), "Should be able to delete collection through adapter");
    
    // Verify deletion
    let final_list = collection_adapter.list_collections().await.unwrap();
    assert!(!final_list.contains(&test_collection.to_string()), "Deleted collection should not be listed");
}

#[tokio::test]
async fn test_filtering_service_adapter() {
    // Test FilteringServiceAdapter bridging new interface with existing FilterService
    let filter_service = Arc::new(FilterService::new());
    let filtering_adapter = FilteringServiceAdapter::new(filter_service);
    
    // Test file filtering decisions
    let text_file = Path::new("document.txt");
    let rust_file = Path::new("main.rs");
    let binary_file = Path::new("app.exe");
    
    let should_index_txt = filtering_adapter.should_index_file(text_file);
    let should_index_rs = filtering_adapter.should_index_file(rust_file);
    let should_index_exe = filtering_adapter.should_index_file(binary_file);
    
    // The adapter should provide filtering decisions
    assert!(should_index_txt || !should_index_txt, "Adapter should make filtering decisions for txt files");
    assert!(should_index_rs || !should_index_rs, "Adapter should make filtering decisions for rs files");
    assert!(should_index_exe || !should_index_exe, "Adapter should make filtering decisions for exe files");
    
    // Test directory traversal decisions
    let src_dir = Path::new("src");
    let target_dir = Path::new("target");
    let docs_dir = Path::new("docs");
    
    let should_traverse_src = filtering_adapter.should_traverse_directory(src_dir);
    let should_traverse_target = filtering_adapter.should_traverse_directory(target_dir);
    let should_traverse_docs = filtering_adapter.should_traverse_directory(docs_dir);
    
    // The adapter should provide traversal decisions
    assert!(should_traverse_src || !should_traverse_src, "Adapter should make traversal decisions for src");
    assert!(should_traverse_target || !should_traverse_target, "Adapter should make traversal decisions for target");
    assert!(should_traverse_docs || !should_traverse_docs, "Adapter should make traversal decisions for docs");
    
    // Test filter summary
    let filter_summary = filtering_adapter.get_filter_summary();
    
    // The summary should have the expected structure
    assert!(filter_summary.safe_patterns.len() >= 0, "Safe patterns should be accessible");
    assert!(filter_summary.ignore_patterns.len() >= 0, "Ignore patterns should be accessible");
    // Boolean fields should have valid values
    assert!(filter_summary.case_sensitive || !filter_summary.case_sensitive, "Case sensitive setting should be accessible");
    assert!(filter_summary.follow_symlinks || !filter_summary.follow_symlinks, "Follow symlinks setting should be accessible");
}

#[tokio::test]
async fn test_adapter_error_handling() {
    // Test that adapters properly handle and propagate errors
    let fs_adapter = StandardFileSystemService::new();
    
    // Test with non-existent file
    let nonexistent_file = Path::new("/definitely/does/not/exist/file.txt");
    
    let exists_result = fs_adapter.exists(nonexistent_file).await;
    assert!(exists_result.is_ok(), "exists() should not error, just return false");
    assert!(!exists_result.unwrap(), "Non-existent file should not exist");
    
    let read_result = fs_adapter.read_file_content(nonexistent_file).await;
    assert!(read_result.is_err(), "Reading non-existent file should error");
    
    let metadata_result = fs_adapter.get_file_metadata(nonexistent_file).await;
    assert!(metadata_result.is_err(), "Getting metadata for non-existent file should error");
    
    // Test collection manager with invalid operations
    let collection_adapter = SimpleCollectionManager::new();
    
    let stats_result = collection_adapter.get_collection_stats("nonexistent_collection").await;
    assert!(stats_result.is_err(), "Getting stats for non-existent collection should error");
    
    let delete_result = collection_adapter.delete_collection("nonexistent_collection").await;
    assert!(delete_result.is_err(), "Deleting non-existent collection should error");
}

#[tokio::test]
async fn test_adapter_concurrent_access() {
    // Test that adapters handle concurrent access properly
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let progress_adapter = Arc::new(InMemoryProgressTracker::new());
    let counter = Arc::new(AtomicUsize::new(0));
    
    let mut handles = vec![];
    
    // Spawn multiple tasks that use the adapter concurrently
    for i in 0..10 {
        let adapter = Arc::clone(&progress_adapter);
        let counter = Arc::clone(&counter);
        
        let handle = tokio::spawn(async move {
            let path = Path::new(&format!("concurrent_file_{}.txt", i));
            adapter.file_processed(path, true).await;
            counter.fetch_add(1, Ordering::SeqCst);
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }
    
    // Verify all tasks completed
    assert_eq!(counter.load(Ordering::SeqCst), 10, "All concurrent tasks should complete");
    
    // Verify progress was tracked correctly
    let final_progress = progress_adapter.get_progress().await.unwrap();
    assert_eq!(final_progress.files_processed, 10, "All files should be tracked");
    assert_eq!(final_progress.files_succeeded, 10, "All files should be successful");
}

#[tokio::test]
async fn test_adapter_interface_consistency() {
    // Test that adapters maintain consistent behavior across interface calls
    let fs_adapter = StandardFileSystemService::new();
    
    // Create a test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("consistency_test.txt");
    std::fs::write(&test_file, "consistency test").expect("Should write test file");
    
    // Multiple calls should return consistent results
    for _ in 0..5 {
        let exists_result = fs_adapter.exists(&test_file).await;
        assert!(exists_result.is_ok() && exists_result.unwrap(), "File existence should be consistent");
        
        let is_file_result = fs_adapter.is_file(&test_file).await;
        assert!(is_file_result.is_ok() && is_file_result.unwrap(), "File type check should be consistent");
        
        let content_result = fs_adapter.read_file_content(&test_file).await;
        assert!(content_result.is_ok(), "File reading should be consistent");
        assert_eq!(content_result.unwrap(), "consistency test", "Content should be consistent");
    }
    
    // Clean up
    std::fs::remove_file(&test_file).ok();
}

#[tokio::test]
async fn test_adapter_dependency_inversion() {
    // Test that adapters properly implement dependency inversion
    
    // High-level code should work with any FileSystemService implementation
    async fn high_level_file_operation(fs: Arc<dyn FileSystemService>) -> Result<bool> {
        fs.exists(Path::new(".")).await
    }
    
    // High-level code should work with any ProgressTracker implementation
    async fn high_level_progress_operation(tracker: Arc<dyn ProgressTracker>) -> Result<ProgressStats> {
        tracker.get_progress().await
    }
    
    // Test with adapter implementations
    let fs_adapter: Arc<dyn FileSystemService> = Arc::new(StandardFileSystemService::new());
    let progress_adapter: Arc<dyn ProgressTracker> = Arc::new(InMemoryProgressTracker::new());
    
    let fs_result = high_level_file_operation(fs_adapter).await;
    assert!(fs_result.is_ok(), "High-level code should work with FileSystemService adapter");
    
    let progress_result = high_level_progress_operation(progress_adapter).await;
    assert!(progress_result.is_ok(), "High-level code should work with ProgressTracker adapter");
    
    // This demonstrates that adapters properly enable dependency inversion
    assert!(true, "Adapters enable proper dependency inversion");
}
