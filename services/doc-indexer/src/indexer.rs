use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use crate::config::Config;
use crate::document::{Document, DocumentProcessor};
use crate::vector_db_trait::{VectorDatabase, SearchResult};
use crate::vectordb_simple::VectorDB;
use crate::qdrant_client::QdrantVectorDB;
use crate::watcher_v2::{DocumentWatcher, FileEvent};
use crate::embedding_provider::EmbeddingProvider;

pub struct DocumentIndexer {
    config: Config,
    processor: DocumentProcessor,
    vectordb: Box<dyn VectorDatabase>,
    embedder: Box<dyn EmbeddingProvider>,
}

impl DocumentIndexer {
    pub async fn new(config: Config, embedder: Box<dyn EmbeddingProvider>) -> Result<Self> {
        let processor = DocumentProcessor::new(config.docs_directory.clone())?;
        
        // Choose between mock and real Qdrant based on URL
        let vectordb: Box<dyn VectorDatabase> = if config.qdrant_url.contains("mock") {
            Box::new(VectorDB::new(&config.qdrant_url, config.collection_name.clone()).await
                .context("Failed to initialize mock vector database")?)
        } else {
            Box::new(QdrantVectorDB::new(&config.qdrant_url, config.collection_name.clone()).await
                .context("Failed to initialize Qdrant vector database")?)
        };

        // Ensure collection exists
        vectordb
            .ensure_collection_exists()
            .await
            .context("Failed to ensure collection exists")?;

        Ok(Self {
            config,
            processor,
            vectordb,
            embedder,
        })
    }

    /// Create a new vector database instance for search operations
    pub async fn create_vectordb_for_search(&self) -> Result<Box<dyn VectorDatabase>> {
        if self.config.qdrant_url.contains("mock") {
            Ok(Box::new(VectorDB::new(&self.config.qdrant_url, self.config.collection_name.clone()).await
                .context("Failed to create mock vector database for search")?))
        } else {
            Ok(Box::new(QdrantVectorDB::new(&self.config.qdrant_url, self.config.collection_name.clone()).await
                .context("Failed to create Qdrant vector database for search")?))
        }
    }

    pub async fn index_all_documents(&mut self) -> Result<()> {
        println!("Starting full document indexing...");

        let mut indexed_count = 0;
        let mut error_count = 0;

        for entry in WalkDir::new(&self.config.docs_directory)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && is_markdown_file(path) {
                match self.index_document(path).await {
                    Ok(_) => {
                        indexed_count += 1;
                        debug!("Indexed document: {}", path.display());
                    }
                    Err(e) => {
                        error_count += 1;
                        error!("Failed to index document {}: {}", path.display(), e);
                    }
                }
            }
        }

        info!(
            "Initial indexing complete. Indexed: {}, Errors: {}",
            indexed_count, error_count
        );

        // Log collection stats
        match self.vectordb.get_collection_info().await {
            Ok(info) => {
                info!(
                    "Collection '{}' now contains {} active documents, {} tombstoned",
                    self.config.collection_name, info.active_documents, info.tombstoned_documents
                );
            }
            Err(e) => warn!("Failed to get collection info: {}", e),
        }

        Ok(())
    }

    pub async fn start_watching(&self) -> Result<()> {
        info!("Starting file watcher for: {}", self.config.docs_directory.display());

        let (_watcher, mut event_rx) = DocumentWatcher::new(self.config.docs_directory.clone())?;

        // Process file events
        while let Some(event) = event_rx.recv().await {
            match event {
                FileEvent::Created(path) | FileEvent::Modified(path) => {
                    if let Err(e) = self.index_document(&path).await {
                        error!("Failed to index document {}: {}", path.display(), e);
                    } else {
                        info!("Reindexed document: {}", path.display());
                    }
                }
                FileEvent::Deleted(path) => {
                    if let Err(e) = self.remove_document(&path).await {
                        error!("Failed to remove document {}: {}", path.display(), e);
                    } else {
                        info!("Removed document from index: {}", path.display());
                    }
                }
            }
        }

        Ok(())
    }

    async fn index_document(&self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let document = self.processor.process_document(path, &content)?;
        
        // Check if document needs reprocessing
        if !self.vectordb.needs_reprocessing(&document.doc_id, &document.rev_id).await? {
            debug!("Document {} is up to date, skipping", path.display());
            return Ok(());
        }
        
        // Generate embeddings for all chunks
        let embeddings = self.generate_embeddings(&document).await?;
        
        // Store in vector database
        self.vectordb.upsert_document(&document, &embeddings).await?;

        Ok(())
    }

    async fn remove_document(&self, path: &Path) -> Result<()> {
        let doc_id = self.processor.generate_document_id(path);
        self.vectordb.delete_document(&doc_id).await?;
        Ok(())
    }

    async fn generate_embeddings(&self, document: &Document) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        for chunk in &document.chunks {
            let embedding = self.generate_text_embedding(&chunk.content).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let batch_response = self.embedder.generate_embeddings(&[text.to_string()]).await?;
        
        if batch_response.embeddings.is_empty() {
            anyhow::bail!("No embeddings returned for text");
        }
        
        Ok(batch_response.embeddings[0].embedding.clone())
    }

    pub async fn search_documents(
        &self,
        query: &str,
        limit: Option<usize>,
        filters: Option<HashMap<String, Value>>,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for the query
        let query_embedding = self.generate_text_embedding(query).await?;
        
        // Search in vector database
        let results = self
            .vectordb
            .search(&query_embedding, limit.unwrap_or(10), filters)
            .await?;

        Ok(results)
    }

    pub async fn get_collection_stats(&self) -> Result<String> {
        let info = self.vectordb.get_collection_info().await?;
        Ok(format!(
            "Collection '{}': {} active documents, {} tombstoned",
            self.config.collection_name, info.active_documents, info.tombstoned_documents
        ))
    }
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let docs_directory = temp_dir.path().to_path_buf();
        
        let config = Config {
            docs_directory,
            qdrant_url: "http://localhost:6333".to_string(),
            collection_name: "test_docs".to_string(),
            openai_api_key: None,
        };
        
        (config, temp_dir)
    }

    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file(Path::new("test.md")));
        assert!(is_markdown_file(Path::new("test.MD")));
        assert!(is_markdown_file(Path::new("/path/to/file.md")));
        assert!(!is_markdown_file(Path::new("test.txt")));
        assert!(!is_markdown_file(Path::new("test")));
        assert!(!is_markdown_file(Path::new("test.mdx")));
    }

    #[tokio::test]
    async fn test_document_indexer_creation() {
        use crate::embedding_provider::{EmbeddingConfig, MockEmbedder};
        
        let (config, _temp_dir) = create_test_config().await;
        let embedder = Box::new(MockEmbedder::new(EmbeddingConfig::default()));
        
        // This test will fail if Qdrant is not running, which is expected in CI
        // In a real environment, you'd use a test container or mock
        match DocumentIndexer::new(config, embedder).await {
            Ok(_) => {
                // If Qdrant is available, great!
            }
            Err(e) => {
                // Expected if Qdrant is not running
                assert!(e.to_string().contains("Failed to initialize vector database"));
            }
        }
    }
}
