/// Document indexing application service
/// 
/// This service coordinates document indexing operations using the domain
/// models and infrastructure services. It implements the use cases for
/// document processing and indexing.

use std::sync::Arc;
use std::collections::HashMap;
use zero_latency_core::{Result, models::{Document, DocumentChunk}};
use zero_latency_vector::{VectorRepository, EmbeddingGenerator, VectorDocument};
use zero_latency_search::{SearchOrchestrator, SearchRequest, SearchResponse};

use crate::application::container::ServiceContainer;
use crate::application::ContentProcessor;

/// Application service for document indexing operations
#[derive(Clone)]
pub struct DocumentIndexingService {
    vector_repository: Arc<dyn VectorRepository>,
    embedding_generator: Arc<dyn EmbeddingGenerator>,
    search_orchestrator: Arc<dyn SearchOrchestrator>,
}

impl DocumentIndexingService {
    /// Create a new document indexing service
    pub fn new(container: &ServiceContainer) -> Self {
        Self {
            vector_repository: container.vector_repository(),
            embedding_generator: container.embedding_generator(),
            search_orchestrator: container.search_orchestrator(),
        }
    }
    
    /// Index a document by chunking it and creating embeddings
    pub async fn index_document(&self, document: Document) -> Result<()> {
        // Create chunks from the document
        let chunks = self.create_document_chunks(&document).await?;
        
        // Generate embeddings for each chunk
        let mut vector_documents = Vec::new();
        for chunk in chunks {
            let embedding = self.embedding_generator
                .generate_embedding(&chunk.content)
                .await?;
            
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
                    custom: chunk.metadata.custom.clone(),
                },
            };
            
            vector_documents.push(vector_doc);
        }
        
        // Store in vector repository
        self.vector_repository
            .insert(vector_documents)
            .await?;
        
        Ok(())
    }
    
    /// Delete a document from the index
    pub async fn delete_document(&self, document_id: &str) -> Result<()> {
        let _deleted = self.vector_repository
            .delete(document_id)
            .await?;
        Ok(())
    }
    
    /// Search for documents similar to a query
    pub async fn search_documents(&self, query: &str, limit: usize) -> Result<SearchResponse> {
        let search_request = SearchRequest::new(query).with_limit(limit);
        
        self.search_orchestrator
            .search(search_request)
            .await
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
    
    /// Index all documents from a directory path
    pub async fn index_documents_from_path(&self, path: &str, recursive: bool) -> Result<(u64, f64)> {
        use std::time::Instant;
        use std::fs;
        
        let start_time = Instant::now();
        let mut documents_processed = 0u64;
        
        let path = std::path::Path::new(path);
        if !path.exists() {
            return Err(zero_latency_core::ZeroLatencyError::validation("path", "Path does not exist"));
        }
        
        if path.is_file() {
            // Index single file
            if let Ok(content) = fs::read_to_string(path) {
                let document = Document {
                    id: zero_latency_core::Uuid::new_v4(),
                    title: path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    content,
                    path: path.to_path_buf(),
                    last_modified: chrono::Utc::now(),
                    size: fs::metadata(path).map(|m| m.len()).unwrap_or(0),
                    metadata: zero_latency_core::models::DocumentMetadata::default(),
                };
                
                self.index_document(document).await?;
                documents_processed += 1;
            }
        } else if path.is_dir() {
            // Index directory
            documents_processed = self.index_directory(path, recursive).await?;
        }
        
        let processing_time = start_time.elapsed().as_millis() as f64;
        Ok((documents_processed, processing_time))
    }
    
    /// Recursively index documents in a directory
    fn index_directory<'a>(
        &'a self, 
        dir: &'a std::path::Path, 
        recursive: bool
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + Send + 'a>> {
        Box::pin(async move {
            use std::fs;
            
            let mut documents_processed = 0u64;
            let mut files_scanned = 0u64;
            let start_time = std::time::Instant::now();
            
            tracing::info!("Starting directory indexing: {}", dir.display());
            
            if let Ok(entries) = fs::read_dir(dir) {
                let entries: Vec<_> = entries.flatten().collect();
                let total_entries = entries.len();
                
                tracing::info!("Found {} entries to process in {}", total_entries, dir.display());
                
                for (index, entry) in entries.into_iter().enumerate() {
                    let path = entry.path();
                    files_scanned += 1;
                    
                    // Log progress every 100 files or every 10 seconds
                    if files_scanned % 100 == 0 || start_time.elapsed().as_secs() % 10 == 0 {
                        tracing::info!(
                            "Progress: {}/{} files scanned, {} documents indexed ({:.1}%)",
                            files_scanned,
                            total_entries,
                            documents_processed,
                            (index as f64 / total_entries as f64) * 100.0
                        );
                    }
                    
                    if path.is_file() {
                        // Read file content first
                        if let Ok(raw_content) = fs::read_to_string(&path) {
                            // Detect content type
                            let content_type = ContentProcessor::detect_content_type(&path, &raw_content);
                            
                            // Check if this content type should be indexed
                            if ContentProcessor::should_index(&content_type) {
                                // Process content based on type
                                match ContentProcessor::process_content(&raw_content, &content_type) {
                                    Ok(processed_content) => {
                                        let document = Document {
                                            id: zero_latency_core::Uuid::new_v4(),
                                            title: path.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Unknown")
                                                .to_string(),
                                            content: processed_content,
                                            path: path.clone(),
                                            last_modified: chrono::Utc::now(),
                                            size: fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                                            metadata: {
                                                let mut metadata = zero_latency_core::models::DocumentMetadata::default();
                                                metadata.content_type = Some(format!("{:?}", content_type));
                                                metadata
                                            },
                                        };
                                        
                                        if let Err(e) = self.index_document(document).await {
                                            tracing::warn!("Failed to index {}: {}", path.display(), e);
                                        } else {
                                            documents_processed += 1;
                                            tracing::debug!("Indexed {} as {:?}", path.display(), content_type);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to process content for {}: {}", path.display(), e);
                                    }
                                }
                            } else {
                                tracing::debug!("Skipping {:?} file: {}", content_type, path.display());
                            }
                        } else {
                            tracing::debug!("Could not read file as UTF-8: {}", path.display());
                        }
                    } else if path.is_dir() && recursive {
                        // Recursively index subdirectories
                        tracing::debug!("Recursing into directory: {}", path.display());
                        documents_processed += self.index_directory(&path, recursive).await?;
                    }
                }
            }
            
            let elapsed = start_time.elapsed();
            tracing::info!(
                "Completed directory indexing: {} - {} documents processed in {:.2}s",
                dir.display(),
                documents_processed,
                elapsed.as_secs_f64()
            );
            
            Ok(documents_processed)
        })
    }
    
    /// Create document chunks from a document
    async fn create_document_chunks(&self, document: &Document) -> Result<Vec<DocumentChunk>> {
        // Simple chunking strategy - split by sentences
        // In a real implementation, this might use more sophisticated chunking
        let sentences: Vec<&str> = document.content
            .split('.')
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        let mut chunks = Vec::new();
        let chunk_size = 3; // 3 sentences per chunk
        
        for (i, chunk_sentences) in sentences.chunks(chunk_size).enumerate() {
            let content = chunk_sentences.join(". ") + ".";
            
            let chunk = DocumentChunk {
                id: zero_latency_core::Uuid::new_v4(),
                document_id: document.id,
                content,
                chunk_index: i,
                heading_path: vec![], // Would be extracted in real implementation
                start_offset: 0, // Would be calculated in real implementation
                end_offset: 0,   // Would be calculated in real implementation
                metadata: zero_latency_core::models::ChunkMetadata {
                    custom: {
                        let mut custom = HashMap::new();
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

/// Health information about the document index
#[derive(Debug, Clone)]
pub struct IndexHealth {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub index_size_mb: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
