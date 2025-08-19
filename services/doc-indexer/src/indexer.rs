use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path};
use std::time::Duration;
use tokio::fs;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use crate::config::Config;
use crate::document::{Document, DocumentProcessor};
use crate::vectordb_simple::{SearchResult, VectorDB};
use crate::watcher::{DocumentWatcher, FileEvent};

pub struct DocumentIndexer {
    config: Config,
    vector_db: VectorDB,
    document_processor: DocumentProcessor,
}

impl DocumentIndexer {
    pub async fn new(config: Config) -> Result<Self> {
        let vector_db = VectorDB::new(&config.qdrant_url, config.collection_name.clone())
            .await
            .context("Failed to initialize vector database")?;

        // Ensure collection exists
        vector_db
            .ensure_collection_exists()
            .await
            .context("Failed to ensure collection exists")?;

        let document_processor = DocumentProcessor::new();

        Ok(Self {
            config,
            vector_db,
            document_processor,
        })
    }

    pub async fn index_all_documents(&self) -> Result<()> {
        info!("Starting initial indexing of all documents in: {}", self.config.docs_path.display());

        let mut indexed_count = 0;
        let mut error_count = 0;

        for entry in WalkDir::new(&self.config.docs_path)
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
        match self.vector_db.get_collection_info().await {
            Ok(info) => {
                info!(
                    "Collection '{}' now contains {} points ({} vectors)",
                    info.name, info.points_count, info.vectors_count
                );
            }
            Err(e) => warn!("Failed to get collection info: {}", e),
        }

        Ok(())
    }

    pub async fn start_watching(&self) -> Result<()> {
        info!("Starting file watcher for: {}", self.config.docs_path.display());

        let (watcher, mut rx) = DocumentWatcher::new(self.config.docs_path.clone())
            .context("Failed to create document watcher")?;

        // Process file events
        while let Some(event) = rx.recv().await {
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

        watcher.stop().await;
        Ok(())
    }

    async fn index_document(&self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let document = self.document_processor.process_document(path, &content)?;
        
        // Generate embeddings for all chunks
        let embeddings = self.generate_embeddings(&document).await?;
        
        // Store in vector database
        self.vector_db.upsert_document(&document, &embeddings).await?;

        Ok(())
    }

    async fn remove_document(&self, path: &Path) -> Result<()> {
        let document_id = self.document_processor.generate_document_id(path);
        self.vector_db.delete_document(&document_id).await?;
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

    async fn generate_text_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        // For now, return a mock embedding vector
        // In production, this would call OpenAI's API
        Ok(vec![0.1f32; 1536])
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
            .vector_db
            .search(&query_embedding, limit.unwrap_or(10), filters)
            .await?;

        Ok(results)
    }

    pub async fn get_collection_stats(&self) -> Result<String> {
        let info = self.vector_db.get_collection_info().await?;
        Ok(format!(
            "Collection '{}': {} documents, {} vectors",
            info.name, info.points_count, info.vectors_count
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
    use std::fs;

    async fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let docs_path = temp_dir.path().to_path_buf();
        
        let config = Config {
            docs_path,
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
        let (config, _temp_dir) = create_test_config().await;
        
        // This test will fail if Qdrant is not running, which is expected in CI
        // In a real environment, you'd use a test container or mock
        match DocumentIndexer::new(config).await {
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
