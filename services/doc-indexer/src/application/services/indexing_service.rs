use std::collections::HashMap;
use std::path::Path;
/// Document indexing service with dependency injection
///
/// This service provides a clean, testable interface for document indexing
/// with proper separation of concerns and dependency injection patterns.
use std::sync::Arc;
use zero_latency_core::{
    models::{Document, DocumentMetadata},
    Result, Uuid,
};

use crate::application::content_processing::ContentProcessor;
use crate::application::indexing_strategies::{IndexingStrategy, StandardIndexingStrategy};
use crate::application::interfaces::{
    CollectionManager, EmbeddingService, FileSystemService, FilteringService, ProgressTracker,
    VectorStorage,
};

/// Document indexing service with dependency injection
///
/// This service demonstrates clean architecture principles:
/// - Single Responsibility: Only coordinates document indexing workflow
/// - Open-Closed: Extensible via strategy injection
/// - Liskov Substitution: All dependencies are substitutable interfaces
/// - Interface Segregation: Depends only on focused, minimal interfaces
/// - Dependency Inversion: Depends on abstractions, not implementations
pub struct IndexingService {
    // Core dependencies injected via focused interfaces
    vector_storage: Arc<dyn VectorStorage>,
    embedding_service: Arc<dyn EmbeddingService>,
    file_system: Arc<dyn FileSystemService>,
    filtering: Arc<dyn FilteringService>,
    progress_tracker: Arc<dyn ProgressTracker>,
    collection_manager: Arc<dyn CollectionManager>,

    // Strategy for indexing behavior (OCP)
    indexing_strategy: Arc<dyn IndexingStrategy>,

    // Content processing (already SOLID-compliant)
    content_processor: ContentProcessor,
}

impl IndexingService {
    /// Create a new service with injected dependencies
    ///
    /// This constructor follows DIP by accepting abstractions rather than concretions
    pub fn new(
        vector_storage: Arc<dyn VectorStorage>,
        embedding_service: Arc<dyn EmbeddingService>,
        file_system: Arc<dyn FileSystemService>,
        filtering: Arc<dyn FilteringService>,
        progress_tracker: Arc<dyn ProgressTracker>,
        collection_manager: Arc<dyn CollectionManager>,
        indexing_strategy: Option<Arc<dyn IndexingStrategy>>,
        content_processor: Option<ContentProcessor>,
    ) -> Self {
        Self {
            vector_storage,
            embedding_service,
            file_system,
            filtering,
            progress_tracker,
            collection_manager,
            indexing_strategy: indexing_strategy
                .unwrap_or_else(|| Arc::new(StandardIndexingStrategy::new())),
            content_processor: content_processor.unwrap_or_else(ContentProcessor::new),
        }
    }

    /// Index a single document
    ///
    /// This method orchestrates the indexing workflow while delegating
    /// specific responsibilities to injected services (SRP)
    pub async fn index_document(&self, document: Document, collection: &str) -> Result<()> {
        // Use strategy pattern for indexing behavior (OCP)
        self.indexing_strategy
            .index_document(
                document,
                collection,
                &*self.vector_storage,
                &*self.embedding_service,
                &self.content_processor,
            )
            .await?;

        Ok(())
    }

    /// Index a file by path
    ///
    /// Orchestrates file reading, content processing, and indexing
    pub async fn index_file(&self, path: &Path, collection: &str) -> Result<bool> {
        // Check if file should be indexed (delegation to filtering service)
        if !self.filtering.should_index_file(path) {
            return Ok(false);
        }

        // Check if path is actually a file
        if !self.file_system.is_file(path).await? {
            return Ok(false);
        }

        // Read file content (delegation to file system service)
        let content = match self.file_system.read_file_content(path).await {
            Ok(content) => content,
            Err(_) => {
                self.progress_tracker.file_processed(path, false).await;
                return Ok(false);
            }
        };

        // Get file metadata
        let file_metadata = self.file_system.get_file_metadata(path).await?;

        // Process content using SOLID-compliant content processor
        let processed_content = match self.content_processor.process_document(path, &content)? {
            Some(content) => content,
            None => {
                // Content type not supported for indexing
                return Ok(false);
            }
        };

        // Detect content type from file extension
        let content_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("text/{}", ext.to_lowercase()))
            .unwrap_or("text/plain".to_string());

        // Extract title from path
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();

        // Create document
        let document = Document {
            id: Uuid::new_v4(),
            title,
            content: processed_content,
            path: path.to_path_buf(),
            last_modified: file_metadata.modified,
            size: file_metadata.size,
            metadata: DocumentMetadata {
                content_type: Some(content_type),
                custom: HashMap::new(),
                ..Default::default()
            },
        };

        // Delegate indexing to strategy
        self.indexing_strategy
            .index_document(
                document,
                collection,
                &*self.vector_storage,
                &*self.embedding_service,
                &self.content_processor,
            )
            .await?;

        // Track successful processing
        self.progress_tracker.file_processed(path, true).await;

        Ok(true)
    }

    /// Index a directory recursively
    ///
    /// Orchestrates directory traversal and file indexing
    pub fn index_directory<'a>(
        &'a self,
        dir: &'a Path,
        collection: &'a str,
        recursive: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + Send + 'a>> {
        Box::pin(async move {
            let mut files_indexed = 0u64;

            if !self.file_system.is_directory(dir).await? {
                return Ok(0);
            }

            if !self.filtering.should_traverse_directory(dir) {
                return Ok(0);
            }

            // List directory contents
            let entries = self.file_system.list_directory(dir).await?;

            for entry in entries {
                if self.file_system.is_file(&entry).await? {
                    if self.index_file(&entry, collection).await? {
                        files_indexed += 1;
                    }
                } else if recursive && self.file_system.is_directory(&entry).await? {
                    let sub_count = self.index_directory(&entry, collection, recursive).await?;
                    files_indexed += sub_count;
                }
            }

            Ok(files_indexed)
        })
    }

    /// Get indexing progress
    pub async fn get_progress(&self) -> Result<crate::application::interfaces::ProgressStats> {
        self.progress_tracker.get_progress().await
    }

    /// List collections
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        self.collection_manager.list_collections().await
    }

    /// Get collection statistics
    pub async fn get_collection_stats(
        &self,
        name: &str,
    ) -> Result<crate::application::interfaces::CollectionStats> {
        self.collection_manager.get_collection_stats(name).await
    }
}

/// Builder pattern for creating IndexingService
///
/// This builder follows the Builder pattern and makes dependency injection easier
pub struct IndexingServiceBuilder {
    vector_storage: Option<Arc<dyn VectorStorage>>,
    embedding_service: Option<Arc<dyn EmbeddingService>>,
    file_system: Option<Arc<dyn FileSystemService>>,
    filtering: Option<Arc<dyn FilteringService>>,
    progress_tracker: Option<Arc<dyn ProgressTracker>>,
    collection_manager: Option<Arc<dyn CollectionManager>>,
    indexing_strategy: Option<Arc<dyn IndexingStrategy>>,
    content_processor: Option<ContentProcessor>,
}

impl IndexingServiceBuilder {
    pub fn new() -> Self {
        Self {
            vector_storage: None,
            embedding_service: None,
            file_system: None,
            filtering: None,
            progress_tracker: None,
            collection_manager: None,
            indexing_strategy: None,
            content_processor: None,
        }
    }

    pub fn vector_storage(mut self, storage: Arc<dyn VectorStorage>) -> Self {
        self.vector_storage = Some(storage);
        self
    }

    pub fn embedding_service(mut self, service: Arc<dyn EmbeddingService>) -> Self {
        self.embedding_service = Some(service);
        self
    }

    pub fn file_system(mut self, fs: Arc<dyn FileSystemService>) -> Self {
        self.file_system = Some(fs);
        self
    }

    pub fn filtering(mut self, filtering: Arc<dyn FilteringService>) -> Self {
        self.filtering = Some(filtering);
        self
    }

    pub fn progress_tracker(mut self, tracker: Arc<dyn ProgressTracker>) -> Self {
        self.progress_tracker = Some(tracker);
        self
    }

    pub fn collection_manager(mut self, manager: Arc<dyn CollectionManager>) -> Self {
        self.collection_manager = Some(manager);
        self
    }

    pub fn indexing_strategy(mut self, strategy: Arc<dyn IndexingStrategy>) -> Self {
        self.indexing_strategy = Some(strategy);
        self
    }

    pub fn content_processor(mut self, processor: ContentProcessor) -> Self {
        self.content_processor = Some(processor);
        self
    }

    pub fn build(self) -> Result<IndexingService> {
        Ok(IndexingService::new(
            self.vector_storage.ok_or_else(|| {
                zero_latency_core::ZeroLatencyError::configuration("VectorStorage is required")
            })?,
            self.embedding_service.ok_or_else(|| {
                zero_latency_core::ZeroLatencyError::configuration("EmbeddingService is required")
            })?,
            self.file_system.ok_or_else(|| {
                zero_latency_core::ZeroLatencyError::configuration("FileSystemService is required")
            })?,
            self.filtering.ok_or_else(|| {
                zero_latency_core::ZeroLatencyError::configuration("FilteringService is required")
            })?,
            self.progress_tracker.ok_or_else(|| {
                zero_latency_core::ZeroLatencyError::configuration("ProgressTracker is required")
            })?,
            self.collection_manager.ok_or_else(|| {
                zero_latency_core::ZeroLatencyError::configuration("CollectionManager is required")
            })?,
            self.indexing_strategy,
            self.content_processor,
        ))
    }
}
