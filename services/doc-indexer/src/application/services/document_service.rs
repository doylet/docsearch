/// Document indexing application service
///
/// This service coordinates document indexing operations using the domain
/// models and infrastructure services. It implements the use cases for
/// document processing and indexing.
use std::sync::Arc;
use zero_latency_core::{
    models::{Document, DocumentChunk},
    Result,
};
use zero_latency_search::{
    QueryEnhancer, ResultRanker, SearchOrchestrator, SearchRequest, SearchResponse,
};
use zero_latency_vector::{EmbeddingGenerator, VectorDocument, VectorRepository};

use crate::application::container::ServiceContainer;
use crate::application::content_processing::extraction::{build_file_metadata, read_document_text};
use crate::application::services::filter_service::{FilterService, IndexingFilters};
use crate::application::ContentProcessor;

/// Application service for document indexing operations
#[derive(Clone)]
pub struct DocumentIndexingService {
    vector_repository: Arc<dyn VectorRepository>,
    embedding_generator: Arc<dyn EmbeddingGenerator>,
    search_orchestrator: Arc<dyn SearchOrchestrator>,
    filter_service: Arc<FilterService>,
    content_processor: ContentProcessor,
    query_enhancer: Option<Arc<dyn QueryEnhancer>>,
    result_ranker: Option<Arc<dyn ResultRanker>>,
    max_binary_file_size: u64,
}

impl DocumentIndexingService {
    /// Create a new document indexing service with default filters
    pub fn new(container: &ServiceContainer) -> Self {
        let default_filters = IndexingFilters::new();
        Self::with_filters(container, default_filters)
    }

    /// Create a new document indexing service with custom filters
    pub fn with_filters(container: &ServiceContainer, filters: IndexingFilters) -> Self {
        Self {
            vector_repository: container.vector_repository(),
            embedding_generator: container.embedding_generator(),
            search_orchestrator: container.search_orchestrator(),
            filter_service: Arc::new(FilterService::new(filters)),
            content_processor: ContentProcessor::new(),
            query_enhancer: None,
            result_ranker: None,
            max_binary_file_size: container.config().service.max_binary_file_size,
        }
    }

    /// Create a new document indexing service with enhanced search capabilities
    pub fn with_enhanced_search(
        container: &ServiceContainer,
        filters: IndexingFilters,
        query_enhancer: Option<Arc<dyn QueryEnhancer>>,
        result_ranker: Option<Arc<dyn ResultRanker>>,
    ) -> Self {
        Self {
            vector_repository: container.vector_repository(),
            embedding_generator: container.embedding_generator(),
            search_orchestrator: container.search_orchestrator(),
            filter_service: Arc::new(FilterService::new(filters)),
            content_processor: ContentProcessor::new(),
            query_enhancer,
            result_ranker,
            max_binary_file_size: container.config().service.max_binary_file_size,
        }
    }

    /// Get a reference to the vector repository for advanced operations
    pub fn vector_repository(&self) -> &Arc<dyn VectorRepository> {
        &self.vector_repository
    }

    /// Index a document by chunking it and creating embeddings
    pub async fn index_document(&self, document: Document) -> Result<()> {
        self.index_document_with_collection(document, "zero_latency_docs")
            .await
    }

    /// Index a document with specific collection information
    pub async fn index_document_with_collection(
        &self,
        document: Document,
        collection_name: &str,
    ) -> Result<()> {
        // Create chunks from the document
        let chunks = self.create_document_chunks(&document).await?;

        // Generate embeddings for each chunk
        let mut vector_documents = Vec::new();
        for chunk in chunks {
            let embedding = self
                .embedding_generator
                .generate_embedding(&chunk.content)
                .await?;

            let mut custom_metadata = chunk.metadata.custom.clone();
            custom_metadata.insert("collection".to_string(), collection_name.to_string());

            let vector_doc = VectorDocument {
                id: chunk.id,
                embedding,
                metadata: zero_latency_vector::VectorMetadata {
                    document_id: chunk.document_id,
                    chunk_index: chunk.chunk_index,
                    content: chunk.content.clone(),
                    title: document.title.clone(),
                    heading_path: chunk.heading_path.clone(),
                    url: None,
                    collection: Some(collection_name.to_string()),
                    custom: custom_metadata,
                },
            };

            vector_documents.push(vector_doc);
        }

        // Store in vector repository
        self.vector_repository.insert(vector_documents).await?;

        Ok(())
    }

    /// Delete a document from the index
    pub async fn delete_document(&self, document_id: &str) -> Result<()> {
        let _deleted = self.vector_repository.delete(document_id).await?;
        Ok(())
    }

    /// Search for documents similar to a query
    pub async fn search_documents(&self, query: &str, limit: usize) -> Result<SearchResponse> {
        let search_request = SearchRequest::new(query).with_limit(limit);

        self.search_orchestrator.search(search_request).await
    }

    /// Search for documents similar to a query in a specific collection
    pub async fn search_documents_in_collection(
        &self,
        query: &str,
        collection_name: &str,
        limit: usize,
    ) -> Result<SearchResponse> {
        tracing::info!(
            "[AdvancedSearch] Starting search with query: '{}', collection: '{}', limit: {}",
            query,
            collection_name,
            limit
        );
        tracing::info!(
            "[AdvancedSearch] Components available - Query Enhancer: {}, Result Ranker: {}",
            self.query_enhancer.is_some(),
            self.result_ranker.is_some()
        );

        // Use the SearchOrchestrator (which includes analytics) instead of direct vector search
        let mut filters = zero_latency_search::SearchFilters::default();
        filters
            .custom
            .insert("collection".to_string(), collection_name.to_string());

        let search_request = zero_latency_search::SearchRequest::new(query)
            .with_limit(limit)
            .with_filters(filters);

        // This will go through the full pipeline including analytics
        self.search_orchestrator.search(search_request).await
    }

    /// Update an existing document in the index
    pub async fn update_document(&self, document: Document) -> Result<()> {
        // Delete from vector store
        self.delete_document(&document.id.to_string()).await?;

        // Re-index the updated document
        self.index_document(document).await
    }

    /// Get document health/status information
    pub async fn get_index_health(&self) -> Result<IndexHealth> {
        // This would query the vector repository for health metrics
        // For now, return a placeholder
        Ok(IndexHealth {
            total_documents: 0,
            total_chunks: 0,
            index_size_mb: 0.0,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Get total document count
    pub async fn get_document_count(&self) -> Result<u64> {
        // Query the vector repository for document count
        self.vector_repository.count().await.map(|c| c as u64)
    }

    /// Get approximate index size in bytes
    pub async fn get_index_size(&self) -> Result<u64> {
        // This would calculate storage size from vector database
        // For now, return an estimate based on document count
        let count = self.get_document_count().await?;
        Ok(count * 1024) // Rough estimate: 1KB per document
    }

    /// Get current filtering configuration
    pub fn get_filters(&self) -> &IndexingFilters {
        self.filter_service.filters()
    }

    /// Update the filtering configuration
    ///
    /// This creates a new FilterService with the updated configuration.
    /// Note: This updates the current service instance.
    pub fn update_filters(&mut self, filters: IndexingFilters) {
        self.filter_service = Arc::new(FilterService::new(filters));
    }

    /// Create a new DocumentIndexingService with updated filters
    ///
    /// This is useful when you need to preserve immutability or when
    /// the service is shared across multiple contexts.
    pub fn with_updated_filters(&self, filters: IndexingFilters) -> Self {
        Self {
            vector_repository: Arc::clone(&self.vector_repository),
            embedding_generator: Arc::clone(&self.embedding_generator),
            search_orchestrator: Arc::clone(&self.search_orchestrator),
            filter_service: Arc::new(FilterService::new(filters)),
            content_processor: self.content_processor.clone(),
            query_enhancer: self.query_enhancer.clone(),
            result_ranker: self.result_ranker.clone(),
            max_binary_file_size: self.max_binary_file_size,
        }
    }

    /// Index all documents from a directory path
    pub async fn index_documents_from_path(
        &self,
        path: &str,
        recursive: bool,
    ) -> Result<IndexingStats> {
        self.index_documents_from_path_with_filters(path, recursive, None)
            .await
    }

    /// Index all documents from a directory path with optional filtering
    pub async fn index_documents_from_path_with_filters(
        &self,
        path: &str,
        recursive: bool,
        filters: Option<IndexingFilters>,
    ) -> Result<IndexingStats> {
        self.index_documents_from_path_with_filters_and_collection(
            path,
            recursive,
            filters,
            "zero_latency_docs",
        )
        .await
    }

    /// Index all documents from a directory path with optional filtering and collection
    pub async fn index_documents_from_path_with_filters_and_collection(
        &self,
        path: &str,
        recursive: bool,
        filters: Option<IndexingFilters>,
        collection_name: &str,
    ) -> Result<IndexingStats> {
        let start_time = std::time::Instant::now();

        // Use a temporary service if request-specific filters were provided
        let service = match filters {
            Some(filters) => self.with_updated_filters(filters),
            None => self.clone(),
        };

        let path = std::path::Path::new(path);
        if !path.exists() {
            return Err(zero_latency_core::ZeroLatencyError::validation(
                "path",
                "Path does not exist",
            ));
        }

        let mut counts = IndexCounts::default();
        if path.is_file() {
            if service.filter_service.should_index(path) {
                counts.record(service.index_file(path, collection_name).await);
            }
        } else if path.is_dir() {
            counts = service
                .index_directory_with_collection(path, recursive, collection_name)
                .await?;
        }

        Ok(IndexingStats {
            documents_processed: counts.indexed,
            documents_skipped: counts.skipped,
            processing_time_ms: start_time.elapsed().as_millis() as f64,
        })
    }

    /// Read, process and index one file.
    ///
    /// Files that cannot be read or extracted, or whose content fails processing
    /// or indexing, are reported as `Skipped` so one bad file never aborts a run.
    async fn index_file(&self, path: &std::path::Path, collection_name: &str) -> FileOutcome {
        let raw_content = match read_document_text(path, self.max_binary_file_size).await {
            Ok(Some(content)) => content,
            Ok(None) => return FileOutcome::Skipped,
            Err(e) => {
                tracing::debug!("Could not read {}: {}", path.display(), e);
                return FileOutcome::Skipped;
            }
        };

        let content_type = self
            .content_processor
            .detect_content_type(path, &raw_content);
        if !self.content_processor.should_index(&content_type) {
            tracing::debug!("Skipping {:?} file: {}", content_type, path.display());
            return FileOutcome::NotIndexable;
        }

        let processed_content = match self
            .content_processor
            .process_content(&raw_content, &content_type)
        {
            Ok(content) => content,
            Err(e) => {
                tracing::warn!("Failed to process content for {}: {}", path.display(), e);
                return FileOutcome::Skipped;
            }
        };

        let (size, last_modified) = match std::fs::metadata(path) {
            Ok(metadata) => (
                metadata.len(),
                metadata
                    .modified()
                    .map(chrono::DateTime::<chrono::Utc>::from)
                    .unwrap_or_else(|_| chrono::Utc::now()),
            ),
            Err(_) => (0, chrono::Utc::now()),
        };

        let mut metadata = zero_latency_core::models::DocumentMetadata {
            content_type: Some(format!("{:?}", content_type)),
            custom: build_file_metadata(path, size, last_modified),
            ..Default::default()
        };
        metadata
            .custom
            .insert("collection".to_string(), collection_name.to_string());

        let document = Document {
            id: zero_latency_core::Uuid::new_v4(),
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            content: processed_content,
            path: path.to_path_buf(),
            last_modified,
            size,
            metadata,
        };

        match self
            .index_document_with_collection(document, collection_name)
            .await
        {
            Ok(()) => {
                tracing::debug!("Indexed {} as {:?}", path.display(), content_type);
                FileOutcome::Indexed
            }
            Err(e) => {
                tracing::warn!("Failed to index {}: {}", path.display(), e);
                FileOutcome::Skipped
            }
        }
    }

    /// Recursively index documents in a directory
    fn index_directory<'a>(
        &'a self,
        dir: &'a std::path::Path,
        recursive: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IndexCounts>> + Send + 'a>> {
        self.index_directory_with_collection(dir, recursive, "zero_latency_docs")
    }

    /// Recursively index documents in a directory with collection awareness
    fn index_directory_with_collection<'a>(
        &'a self,
        dir: &'a std::path::Path,
        recursive: bool,
        collection_name: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<IndexCounts>> + Send + 'a>> {
        Box::pin(async move {
            use std::fs;

            let mut counts = IndexCounts::default();
            let mut files_scanned = 0u64;
            let start_time = std::time::Instant::now();

            tracing::info!("Starting directory indexing: {}", dir.display());

            if let Ok(entries) = fs::read_dir(dir) {
                let entries: Vec<_> = entries.flatten().collect();
                let total_entries = entries.len();

                tracing::info!(
                    "Found {} entries to process in {}",
                    total_entries,
                    dir.display()
                );

                for (index, entry) in entries.into_iter().enumerate() {
                    let path = entry.path();
                    files_scanned += 1;

                    // Apply filtering rules early to skip unwanted files/directories
                    if path.is_file() && !self.filter_service.should_index(&path) {
                        tracing::debug!("Skipping file (filtered): {}", path.display());
                        continue;
                    }

                    if path.is_dir() && !self.filter_service.should_traverse(&path) {
                        tracing::debug!("Skipping directory (filtered): {}", path.display());
                        continue;
                    }

                    // Log progress every 100 files or every 10 seconds
                    if files_scanned.is_multiple_of(100)
                        || start_time.elapsed().as_secs().is_multiple_of(10)
                    {
                        tracing::info!(
                            "Progress: {}/{} files scanned, {} documents indexed ({:.1}%)",
                            files_scanned,
                            total_entries,
                            counts.indexed,
                            (index as f64 / total_entries as f64) * 100.0
                        );
                    }

                    if path.is_file() {
                        counts.record(self.index_file(&path, collection_name).await);
                    } else if path.is_dir() && recursive {
                        // Recursively index subdirectories
                        tracing::debug!("Recursing into directory: {}", path.display());
                        counts.add(
                            self.index_directory_with_collection(&path, recursive, collection_name)
                                .await?,
                        );
                    }
                }
            }

            let elapsed = start_time.elapsed();
            tracing::info!(
                "Completed directory indexing: {} - {} documents processed, {} skipped in {:.2}s",
                dir.display(),
                counts.indexed,
                counts.skipped,
                elapsed.as_secs_f64()
            );

            Ok(counts)
        })
    }

    /// Create document chunks from a document
    async fn create_document_chunks(&self, document: &Document) -> Result<Vec<DocumentChunk>> {
        // Simple chunking strategy - split by sentences
        // In a real implementation, this might use more sophisticated chunking
        let sentences: Vec<&str> = document
            .content
            .split('.')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let chunk_size = 50; // 50 sentences per chunk (much more reasonable)

        for (i, chunk_sentences) in sentences.chunks(chunk_size).enumerate() {
            let content = chunk_sentences.join(". ") + ".";

            let chunk = DocumentChunk {
                id: zero_latency_core::Uuid::new_v4(),
                document_id: document.id,
                content,
                chunk_index: i,
                heading_path: vec![], // Would be extracted in real implementation
                start_offset: 0,      // Would be calculated in real implementation
                end_offset: 0,        // Would be calculated in real implementation
                metadata: zero_latency_core::models::ChunkMetadata {
                    custom: {
                        let mut custom = document.metadata.custom.clone(); // Start with document metadata
                        custom.insert("chunk_index".to_string(), i.to_string());
                        custom.insert("parent_document_id".to_string(), document.id.to_string());
                        custom
                    },
                    ..Default::default()
                },
            };

            chunks.push(chunk);
        }

        Ok(chunks)
    }
}

/// Result of indexing a path
#[derive(Debug, Clone, Copy)]
pub struct IndexingStats {
    pub documents_processed: u64,
    /// Files that passed filtering but could not be read, extracted or indexed
    pub documents_skipped: u64,
    pub processing_time_ms: f64,
}

/// What happened to a single file during indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileOutcome {
    Indexed,
    Skipped,
    NotIndexable,
}

/// Running totals while walking a directory tree
#[derive(Debug, Default, Clone, Copy)]
struct IndexCounts {
    indexed: u64,
    skipped: u64,
}

impl IndexCounts {
    fn record(&mut self, outcome: FileOutcome) {
        match outcome {
            FileOutcome::Indexed => self.indexed += 1,
            FileOutcome::Skipped => self.skipped += 1,
            FileOutcome::NotIndexable => {}
        }
    }

    fn add(&mut self, other: IndexCounts) {
        self.indexed += other.indexed;
        self.skipped += other.skipped;
    }
}

/// Health information about the document index
#[derive(Debug, Clone)]
pub struct IndexHealth {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub index_size_mb: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
