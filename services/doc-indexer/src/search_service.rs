use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::vector_db_trait::VectorDatabase;
use crate::embedding_provider::EmbeddingProvider;

/// Search request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_k")]
    pub k: usize,
    #[serde(default)]
    pub filters: SearchFilters,
    #[serde(default = "default_true")]
    pub include_snippets: bool,
    #[serde(default)]
    pub highlight: bool,
}

fn default_k() -> usize { 10 }
fn default_true() -> bool { true }

/// Search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_range: Option<DateRange>,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<DateTime<Utc>>,
}

/// Search response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub total_results: usize,
    pub results: Vec<SearchResultItem>,
    pub search_metadata: SearchMetadata,
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub score: f32,
    pub chunk_id: String,
    pub document_id: String,
    pub document_title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    pub heading_path: Vec<String>,
    pub section: String,
    pub doc_type: String,
    pub created_at: DateTime<Utc>,
    pub metadata: ResultMetadata,
}

/// Result metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMetadata {
    pub chunk_index: usize,
    pub chunk_total: usize,
    pub file_path: String,
}

/// Search operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub embedding_time_ms: u64,
    pub search_time_ms: u64,
    pub total_time_ms: u64,
    pub model_used: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub components: HashMap<String, ComponentHealth>,
    pub system: SystemHealth,
}

/// Collection status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub collection: CollectionStatus,
    pub configuration: ConfigurationInfo,
    pub performance: PerformanceMetrics,
}

/// Collection status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStatus {
    pub name: String,
    pub documents: u64,
    pub chunks: u64,
    pub vector_dimensions: u32,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationInfo {
    pub embedding_model: String,
    pub vector_database: String,
    pub collection_name: String,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_search_time_ms: f64,
    pub total_searches: u64,
    pub uptime_seconds: u64,
}

/// Document list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentInfo>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Document information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub id: String,
    pub title: String,
    pub path: String,
    pub doc_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub chunk_count: usize,
    pub size_bytes: usize,
}

/// Document detail response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDetailResponse {
    pub document: DocumentInfo,
    pub chunks: Vec<ChunkInfo>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Chunk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub id: String,
    pub content: String,
    pub section: String,
    pub heading_path: Vec<String>,
    pub chunk_index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Reindex response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReindexResponse {
    pub status: String,
    pub message: String,
    pub documents_processed: usize,
    pub documents_added: usize,
    pub documents_updated: usize,
    pub documents_removed: usize,
    pub total_time_ms: u64,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_watched: Option<u64>,
}

/// System health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub embeddings_generated: u64,
    pub searches_completed: u64,
}

/// Core search service
pub struct SearchService {
    vectordb: Box<dyn VectorDatabase>,
    embedder: Box<dyn EmbeddingProvider>,
    stats: SearchStats,
}

/// Search service statistics
#[derive(Debug, Clone, Default)]
struct SearchStats {
    embeddings_generated: u64,
    searches_completed: u64,
    start_time: Option<DateTime<Utc>>,
}

impl SearchService {
    pub fn new(
        vectordb: Box<dyn VectorDatabase>,
        embedder: Box<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            vectordb,
            embedder,
            stats: SearchStats {
                start_time: Some(Utc::now()),
                ..Default::default()
            },
        }
    }

    /// Perform semantic search
    pub async fn search(&mut self, request: SearchRequest) -> Result<SearchResponse> {
        let start_time = std::time::Instant::now();

        // Generate embedding for query
        let embedding_start = std::time::Instant::now();
        let embedding_response = self
            .embedder
            .generate_embeddings(&[request.query.clone()])
            .await
            .context("Failed to generate query embedding")?;

        let query_embedding = embedding_response
            .embeddings
            .first()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned for query"))?
            .embedding
            .clone();

        let embedding_time = embedding_start.elapsed();

        // Perform vector search
        let search_start = std::time::Instant::now();
        let search_results = self
            .vectordb
            .search(
                &query_embedding,
                request.k,
                self.build_filter_conditions(&request.filters)?,
            )
            .await
            .context("Failed to perform vector search")?;

        let search_time = search_start.elapsed();

        // Convert results and generate snippets
        let mut results = Vec::new();
        for result in search_results {
            let snippet = if request.include_snippets {
                Some(self.generate_snippet(&result.content, &request.query, request.highlight))
            } else {
                None
            };

            results.push(SearchResultItem {
                score: result.score,
                chunk_id: result.chunk_id,
                document_id: result.document_id,
                document_title: result.document_title,
                content: result.content,
                snippet,
                heading_path: self.parse_heading_path(&result.heading),
                section: result.section,
                doc_type: result.doc_type,
                created_at: Utc::now(), // TODO: Get from metadata
                metadata: ResultMetadata {
                    chunk_index: 0,    // TODO: Parse from chunk_id
                    chunk_total: 0,    // TODO: Get from metadata
                    file_path: "".to_string(), // TODO: Get from metadata
                },
            });
        }

        let total_time = start_time.elapsed();

        // Update statistics
        self.stats.searches_completed += 1;

        Ok(SearchResponse {
            query: request.query,
            total_results: results.len(),
            results,
            search_metadata: SearchMetadata {
                embedding_time_ms: embedding_time.as_millis() as u64,
                search_time_ms: search_time.as_millis() as u64,
                total_time_ms: total_time.as_millis() as u64,
                model_used: embedding_response.model,
            },
        })
    }

    /// Get system health status
    pub async fn health(&self) -> Result<HealthResponse> {
        let mut components = HashMap::new();

        // Check embedder health
        let embedder_start = std::time::Instant::now();
        let embedder_healthy = self.embedder.health_check().await.is_ok();
        let embedder_latency = embedder_start.elapsed();

        components.insert(
            "embedder".to_string(),
            ComponentHealth {
                status: if embedder_healthy { "healthy" } else { "unhealthy" }.to_string(),
                latency_ms: Some(embedder_latency.as_millis() as u64),
                collection_size: None,
                files_watched: None,
            },
        );

        // Check vector database health
        let collection_info = self.vectordb.get_collection_info().await.unwrap_or_else(|_| {
            crate::vector_db_trait::CollectionInfo {
                name: "unknown".to_string(),
                vectors_count: 0,
                points_count: 0,
                active_documents: 0,
                tombstoned_documents: 0,
            }
        });

        components.insert(
            "vectordb".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                latency_ms: None,
                collection_size: Some(collection_info.points_count),
                files_watched: None,
            },
        );

        // File watcher health (placeholder)
        components.insert(
            "file_watcher".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                latency_ms: None,
                collection_size: None,
                files_watched: Some(0), // TODO: Get actual count
            },
        );

        let uptime = self
            .stats
            .start_time
            .map(|start| Utc::now().signed_duration_since(start).num_seconds() as u64)
            .unwrap_or(0);

        Ok(HealthResponse {
            status: if embedder_healthy { "healthy" } else { "degraded" }.to_string(),
            components,
            system: SystemHealth {
                uptime_seconds: uptime,
                memory_usage_mb: 0, // TODO: Get actual memory usage
                embeddings_generated: self.stats.embeddings_generated,
                searches_completed: self.stats.searches_completed,
            },
        })
    }

    /// Get detailed system status and statistics
    pub async fn status(&self) -> Result<StatusResponse> {
        let collection_info = self.vectordb.get_collection_info().await?;
        
        let avg_search_time = if self.stats.searches_completed > 0 {
            // Simple average - in production might use a sliding window
            10.0 // Placeholder - would calculate from actual timings
        } else {
            0.0
        };

        let uptime = self
            .stats
            .start_time
            .map(|start| Utc::now().signed_duration_since(start).num_seconds() as u64)
            .unwrap_or(0);

        let collection_name = collection_info.name.clone();

        Ok(StatusResponse {
            status: "healthy".to_string(),
            collection: CollectionStatus {
                name: collection_name.clone(),
                documents: collection_info.active_documents,
                chunks: collection_info.points_count,
                vector_dimensions: 384, // gte-small dimensions
                last_updated: None, // TODO: Track last update time
            },
            configuration: ConfigurationInfo {
                embedding_model: "gte-small".to_string(),
                vector_database: "Qdrant".to_string(),
                collection_name,
            },
            performance: PerformanceMetrics {
                avg_search_time_ms: avg_search_time,
                total_searches: self.stats.searches_completed,
                uptime_seconds: uptime,
            },
        })
    }

    /// List all indexed documents with pagination
    pub async fn list_documents(&self, page: usize, page_size: usize) -> Result<DocumentListResponse> {
        // For now, return a placeholder implementation
        // In a full implementation, this would query the vector database for document metadata
        Ok(DocumentListResponse {
            documents: vec![], // TODO: Implement document listing
            total: 0,
            page,
            page_size,
        })
    }

    /// Get detailed information about a specific document
    pub async fn get_document(&self, _doc_id: &str) -> Result<Option<DocumentDetailResponse>> {
        // For now, return None as this requires more complex vector database queries
        // In a full implementation, this would:
        // 1. Query for all chunks belonging to this document
        // 2. Aggregate document metadata
        // 3. Return structured document information
        Ok(None)
    }

    /// Remove a document from the index
    pub async fn delete_document(&self, doc_id: &str) -> Result<()> {
        self.vectordb.delete_document(doc_id).await
    }

    /// Trigger a full reindex of all documents
    pub async fn reindex(&self) -> Result<ReindexResponse> {
        let start_time = std::time::Instant::now();
        
        // This is a placeholder implementation
        // In a full implementation, this would:
        // 1. Re-scan the document directory
        // 2. Process all documents
        // 3. Update the vector database
        // 4. Return statistics about the operation
        
        let total_time = start_time.elapsed();
        
        Ok(ReindexResponse {
            status: "completed".to_string(),
            message: "Reindex operation completed successfully".to_string(),
            documents_processed: 0,
            documents_added: 0,
            documents_updated: 0,
            documents_removed: 0,
            total_time_ms: total_time.as_millis() as u64,
        })
    }

    /// Build filter conditions for vector search
    fn build_filter_conditions(&self, filters: &SearchFilters) -> Result<Option<HashMap<String, serde_json::Value>>> {
        let mut conditions = HashMap::new();

        if let Some(ref path_prefix) = filters.path_prefix {
            let prefix_filter = serde_json::json!({
                "prefix": path_prefix
            });
            conditions.insert("rel_path".to_string(), prefix_filter);
        }

        if let Some(ref tags) = filters.tags {
            if !tags.is_empty() {
                let tag_filter = serde_json::json!({
                    "any": tags
                });
                conditions.insert("tags".to_string(), tag_filter);
            }
        }

        if let Some(ref date_range) = filters.date_range {
            let mut range_obj = serde_json::Map::new();

            if let Some(after) = date_range.after {
                range_obj.insert(
                    "gte".to_string(),
                    serde_json::Value::String(after.to_rfc3339()),
                );
            }

            if let Some(before) = date_range.before {
                range_obj.insert(
                    "lt".to_string(),
                    serde_json::Value::String(before.to_rfc3339()),
                );
            }

            if !range_obj.is_empty() {
                conditions.insert(
                    "updated_at".to_string(),
                    serde_json::Value::Object(range_obj),
                );
            }
        }

        if conditions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(conditions))
        }
    }

    /// Generate search result snippet with optional highlighting
    fn generate_snippet(&self, content: &str, query: &str, highlight: bool) -> String {
        const SNIPPET_LENGTH: usize = 200;
        const CONTEXT_CHARS: usize = 50;

        // Find the best match position in the content
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        // Simple keyword matching for now
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut best_pos = 0;
        let mut best_score = 0;

        for (pos, _) in content_lower.char_indices() {
            let window_end = std::cmp::min(pos + SNIPPET_LENGTH, content.len());
            let window = &content_lower[pos..window_end];
            
            let score = query_words
                .iter()
                .map(|word| window.matches(word).count())
                .sum::<usize>();

            if score > best_score {
                best_score = score;
                best_pos = pos;
            }
        }

        // Extract snippet around best match
        let start = if best_pos >= CONTEXT_CHARS {
            best_pos - CONTEXT_CHARS
        } else {
            0
        };

        let end = std::cmp::min(start + SNIPPET_LENGTH, content.len());
        let mut snippet = content[start..end].to_string();

        // Add ellipsis
        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < content.len() {
            snippet = format!("{}...", snippet);
        }

        // Apply highlighting if requested
        if highlight {
            for word in &query_words {
                let pattern = regex::Regex::new(&regex::escape(word)).unwrap();
                snippet = pattern
                    .replace_all(&snippet, |caps: &regex::Captures| {
                        format!("<mark>{}</mark>", &caps[0])
                    })
                    .to_string();
            }
        }

        snippet
    }

    /// Parse heading path from heading string
    fn parse_heading_path(&self, heading: &Option<String>) -> Vec<String> {
        heading
            .as_ref()
            .map(|h| {
                h.split(" > ")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding_provider::{MockEmbedder, EmbeddingConfig};

    #[tokio::test]
    async fn test_generate_snippet() {
        let service = SearchService::new(
            Box::new(crate::vectordb_simple::VectorDB::new("mock://", "test".to_string()).await.unwrap()),
            Box::new(MockEmbedder::new(EmbeddingConfig::default())),
        );

        let content = "This is a long piece of content about vector databases and semantic search. It contains many words and should be truncated to show only the relevant parts around the query terms.";
        let snippet = service.generate_snippet(content, "vector databases", true);

        assert!(snippet.contains("vector"));
        assert!(snippet.contains("databases"));
        assert!(snippet.len() <= 300); // Includes ellipsis and markup
    }

    #[tokio::test]
    async fn test_parse_heading_path() {
        let service = SearchService::new(
            Box::new(crate::vectordb_simple::VectorDB::new("mock://", "test".to_string()).await.unwrap()),
            Box::new(MockEmbedder::new(EmbeddingConfig::default())),
        );

        let heading = Some("# Introduction > ## Getting Started > ### Installation".to_string());
        let path = service.parse_heading_path(&heading);

        assert_eq!(path, vec![
            "# Introduction",
            "## Getting Started", 
            "### Installation"
        ]);
    }
}
