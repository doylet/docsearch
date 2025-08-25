/// Indexing strategies following Open-Closed Principle
/// 
/// This module defines strategy interfaces and implementations for different
/// indexing approaches. New strategies can be added without modifying existing code.

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zero_latency_core::{Result, models::Document, Uuid};
use zero_latency_vector::{VectorDocument, VectorMetadata};

use super::interfaces::{VectorStorage, EmbeddingService};
use super::content_processing::ContentProcessor;

/// Strategy interface for document indexing
/// 
/// This interface enables different indexing approaches to be plugged in
/// without modifying the core indexing service (Open-Closed Principle)
#[async_trait]
pub trait IndexingStrategy: Send + Sync {
    /// Index a document using this strategy
    async fn index_document(
        &self,
        document: Document,
        collection: &str,
        vector_storage: &dyn VectorStorage,
        embedding_service: &dyn EmbeddingService,
        content_processor: &ContentProcessor,
    ) -> Result<()>;

    /// Get strategy name for logging/debugging
    fn strategy_name(&self) -> &'static str;

    /// Get strategy configuration summary
    fn get_config(&self) -> StrategyConfig;
}

/// Configuration for indexing strategies
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub name: String,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_chunks_per_document: Option<usize>,
    pub min_chunk_size: usize,
}

/// Standard indexing strategy with configurable chunking
/// 
/// This is the default strategy that chunks documents and creates embeddings
pub struct StandardIndexingStrategy {
    config: StrategyConfig,
}

impl StandardIndexingStrategy {
    pub fn new() -> Self {
        Self {
            config: StrategyConfig {
                name: "standard".to_string(),
                chunk_size: 1000,
                chunk_overlap: 200,
                max_chunks_per_document: Some(50),
                min_chunk_size: 100,
            },
        }
    }

    pub fn with_config(config: StrategyConfig) -> Self {
        Self { config }
    }

    /// Chunk document content into smaller pieces
    fn chunk_content(&self, content: &str) -> Vec<String> {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut chunks = Vec::new();
        
        if words.is_empty() {
            return chunks;
        }

        let chunk_size_words = self.config.chunk_size / 6; // Approximate words per chunk
        let overlap_words = self.config.chunk_overlap / 6;
        
        let mut i = 0;
        while i < words.len() {
            let end = std::cmp::min(i + chunk_size_words, words.len());
            let chunk = words[i..end].join(" ");
            
            // Only include chunks that meet minimum size
            if chunk.len() >= self.config.min_chunk_size {
                chunks.push(chunk);
                
                // Check max chunks limit
                if let Some(max_chunks) = self.config.max_chunks_per_document {
                    if chunks.len() >= max_chunks {
                        break;
                    }
                }
            }
            
            // Move forward with overlap
            i += chunk_size_words.saturating_sub(overlap_words);
            if i == 0 {
                i = 1; // Prevent infinite loop
            }
        }
        
        chunks
    }
}

#[async_trait]
impl IndexingStrategy for StandardIndexingStrategy {
    async fn index_document(
        &self,
        document: Document,
        collection: &str,
        vector_storage: &dyn VectorStorage,
        embedding_service: &dyn EmbeddingService,
        _content_processor: &ContentProcessor,
    ) -> Result<()> {
        // Chunk the document content
        let chunks = self.chunk_content(&document.content);
        
        if chunks.is_empty() {
            tracing::warn!("No chunks generated for document: {}", document.title);
            return Ok(());
        }

        // Generate embeddings for each chunk
        let mut vector_documents = Vec::new();
        
        for (chunk_index, chunk_content) in chunks.into_iter().enumerate() {
            let embedding = embedding_service.generate_embeddings(&chunk_content).await?;
            
            let mut metadata = serde_json::Map::new();
            metadata.insert("title".to_string(), serde_json::Value::String(document.title.clone()));
            metadata.insert("path".to_string(), serde_json::Value::String(
                document.path.to_string_lossy().to_string()
            ));
            metadata.insert("chunk_index".to_string(), serde_json::Value::Number(
                serde_json::Number::from(chunk_index)
            ));
            metadata.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
            metadata.insert("content_preview".to_string(), serde_json::Value::String(
                chunk_content.chars().take(200).collect::<String>()
            ));
            
            let vector_doc = VectorDocument {
                id: Uuid::new_v4(),
                embedding,
                metadata: VectorMetadata {
                    document_id: document.id,
                    chunk_index,
                    content: chunk_content.clone(),
                    title: document.title.clone(),
                    heading_path: vec![],
                    url: None,
                    collection: Some(collection.to_string()),
                    custom: HashMap::new(),
                },
            };
            
            vector_documents.push(vector_doc);
        }

                // Store all vectors
        let vector_count = vector_documents.len();
        vector_storage.store_vectors(vector_documents).await?;
        
        tracing::info!(
            "StandardIndexingStrategy: Indexed document with {} vectors", 
            vector_count
        );
        
        Ok(())
    }

    fn strategy_name(&self) -> &'static str {
        "standard_chunking"
    }

    fn get_config(&self) -> StrategyConfig {
        self.config.clone()
    }
}

/// Fast indexing strategy with minimal chunking
/// 
/// This strategy is optimized for speed over granularity
pub struct FastIndexingStrategy {
    config: StrategyConfig,
}

impl FastIndexingStrategy {
    pub fn new() -> Self {
        Self {
            config: StrategyConfig {
                name: "fast".to_string(),
                chunk_size: 2000,
                chunk_overlap: 0,
                max_chunks_per_document: Some(10),
                min_chunk_size: 500,
            },
        }
    }
}

#[async_trait]
impl IndexingStrategy for FastIndexingStrategy {
    async fn index_document(
        &self,
        document: Document,
        collection: &str,
        vector_storage: &dyn VectorStorage,
        embedding_service: &dyn EmbeddingService,
        _content_processor: &ContentProcessor,
    ) -> Result<()> {
        // Simple strategy: one embedding per document if small enough,
        // otherwise split into large chunks
        
        if document.content.len() <= self.config.chunk_size {
            // Small document - index as single chunk
            let embedding = embedding_service.generate_embeddings(&document.content).await?;
            
            let mut metadata = serde_json::Map::new();
            metadata.insert("title".to_string(), serde_json::Value::String(document.title.clone()));
            metadata.insert("path".to_string(), serde_json::Value::String(
                document.path.to_string_lossy().to_string()
            ));
            metadata.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
            metadata.insert("strategy".to_string(), serde_json::Value::String("fast_single".to_string()));
            
            let vector_doc = VectorDocument {
                id: document.id,
                embedding,
                metadata: VectorMetadata {
                    document_id: document.id,
                    chunk_index: 0,
                    content: document.content.clone(),
                    title: document.title.clone(),
                    heading_path: vec![],
                    url: None,
                    collection: Some(collection.to_string()),
                    custom: HashMap::from([
                        ("strategy".to_string(), "fast_single".to_string()),
                    ]),
                },
            };
            
            vector_storage.store_vector(vector_doc).await?;
        } else {
            // Large document - split into large chunks without overlap
            let chunk_size = self.config.chunk_size;
            let mut chunk_index = 0;
            let mut start = 0;
            
            while start < document.content.len() && chunk_index < self.config.max_chunks_per_document.unwrap_or(10) {
                let end = std::cmp::min(start + chunk_size, document.content.len());
                let chunk = &document.content[start..end];
                
                if chunk.len() >= self.config.min_chunk_size {
                    let embedding = embedding_service.generate_embeddings(chunk).await?;
                    
                    let mut metadata = serde_json::Map::new();
                    metadata.insert("title".to_string(), serde_json::Value::String(document.title.clone()));
                    metadata.insert("path".to_string(), serde_json::Value::String(
                        document.path.to_string_lossy().to_string()
                    ));
                    metadata.insert("chunk_index".to_string(), serde_json::Value::Number(
                        serde_json::Number::from(chunk_index)
                    ));
                    metadata.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
                    metadata.insert("strategy".to_string(), serde_json::Value::String("fast_chunked".to_string()));
                    
                    let vector_doc = VectorDocument {
                        id: Uuid::new_v4(),
                        embedding,
                        metadata: VectorMetadata {
                            document_id: document.id,
                            chunk_index,
                            content: chunk.to_string(),
                            title: document.title.clone(),
                            heading_path: vec![],
                            url: None,
                            collection: Some(collection.to_string()),
                            custom: HashMap::from([
                                ("strategy".to_string(), "fast_chunked".to_string()),
                            ]),
                        },
                    };
                    
                    vector_storage.store_vector(vector_doc).await?;
                    chunk_index += 1;
                }
                
                start = end;
            }
        }

        tracing::info!(
            "Fast-indexed document '{}' using {} strategy", 
            document.title, 
            self.strategy_name()
        );
        
        Ok(())
    }

    fn strategy_name(&self) -> &'static str {
        "fast_indexing"
    }

    fn get_config(&self) -> StrategyConfig {
        self.config.clone()
    }
}

/// Precision indexing strategy with fine-grained chunking
/// 
/// This strategy prioritizes search precision over indexing speed
pub struct PrecisionIndexingStrategy {
    config: StrategyConfig,
}

impl PrecisionIndexingStrategy {
    pub fn new() -> Self {
        Self {
            config: StrategyConfig {
                name: "precision".to_string(),
                chunk_size: 500,
                chunk_overlap: 100,
                max_chunks_per_document: Some(100),
                min_chunk_size: 50,
            },
        }
    }
}

#[async_trait]
impl IndexingStrategy for PrecisionIndexingStrategy {
    async fn index_document(
        &self,
        document: Document,
        collection: &str,
        vector_storage: &dyn VectorStorage,
        embedding_service: &dyn EmbeddingService,
        _content_processor: &ContentProcessor,
    ) -> Result<()> {
        // Fine-grained chunking for maximum search precision
        // This strategy uses sentence boundaries when possible
        
        let sentences: Vec<&str> = document.content
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        
        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            
            // Check if adding this sentence would exceed chunk size
            if !current_chunk.is_empty() && 
               current_chunk.len() + sentence.len() + 1 > self.config.chunk_size {
                
                // Save current chunk if it meets minimum size
                if current_chunk.len() >= self.config.min_chunk_size {
                    chunks.push(current_chunk.clone());
                }
                
                // Start new chunk with overlap
                if self.config.chunk_overlap > 0 && !current_chunk.is_empty() {
                    let words: Vec<&str> = current_chunk.split_whitespace().collect();
                    let overlap_words = self.config.chunk_overlap / 6; // Approximate words
                    if words.len() > overlap_words {
                        let overlap_start = words.len().saturating_sub(overlap_words);
                        current_chunk = words[overlap_start..].join(" ");
                    } else {
                        current_chunk = String::new();
                    }
                } else {
                    current_chunk = String::new();
                }
            }
            
            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }
            current_chunk.push_str(sentence);
            
            // Check max chunks limit
            if let Some(max_chunks) = self.config.max_chunks_per_document {
                if chunks.len() >= max_chunks {
                    break;
                }
            }
        }
        
        // Add final chunk
        if current_chunk.len() >= self.config.min_chunk_size {
            chunks.push(current_chunk);
        }
        
        // Generate embeddings for each chunk
        let mut vector_documents = Vec::new();
        
        for (chunk_index, chunk_content) in chunks.into_iter().enumerate() {
            let embedding = embedding_service.generate_embeddings(&chunk_content).await?;
            
            let mut metadata = serde_json::Map::new();
            metadata.insert("title".to_string(), serde_json::Value::String(document.title.clone()));
            metadata.insert("path".to_string(), serde_json::Value::String(
                document.path.to_string_lossy().to_string()
            ));
            metadata.insert("chunk_index".to_string(), serde_json::Value::Number(
                serde_json::Number::from(chunk_index)
            ));
            metadata.insert("collection".to_string(), serde_json::Value::String(collection.to_string()));
            metadata.insert("strategy".to_string(), serde_json::Value::String("precision".to_string()));
            metadata.insert("content_preview".to_string(), serde_json::Value::String(
                chunk_content.chars().take(200).collect::<String>()
            ));
            
            let vector_doc = VectorDocument {
                id: Uuid::new_v4(),
                embedding,
                metadata: VectorMetadata {
                    document_id: document.id,
                    chunk_index,
                    content: chunk_content.clone(),
                    title: document.title.clone(),
                    heading_path: vec![],
                    url: None,
                    collection: Some(collection.to_string()),
                    custom: HashMap::from([
                        ("strategy".to_string(), "precision".to_string()),
                    ]),
                },
            };
            
            vector_documents.push(vector_doc);
        }

        // Store all vectors
        let vector_count = vector_documents.len();
        vector_storage.store_vectors(vector_documents).await?;
        
        tracing::info!(
            "Precision-indexed document '{}' with {} chunks using {} strategy", 
            document.title, 
            vector_count,
            self.strategy_name()
        );
        
        Ok(())
    }

    fn strategy_name(&self) -> &'static str {
        "precision_indexing"
    }

    fn get_config(&self) -> StrategyConfig {
        self.config.clone()
    }
}
