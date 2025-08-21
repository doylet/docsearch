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
