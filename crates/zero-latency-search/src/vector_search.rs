use async_trait::async_trait;
use zero_latency_core::Result;
use crate::{models::*, traits::*};
use std::sync::Arc;
use zero_latency_vector::VectorRepository;

/// Vector search step that queries the vector database
pub struct VectorSearchStep {
    vector_repo: Arc<dyn VectorRepository>,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl VectorSearchStep {
    pub fn new(
        vector_repo: Arc<dyn VectorRepository>,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        Self {
            vector_repo,
            embedding_service,
        }
    }
}

#[async_trait]
impl SearchStep for VectorSearchStep {
    fn name(&self) -> &str {
        "vector_search"
    }

    async fn execute(&self, context: &mut SearchContext) -> Result<()> {
        // Use enhanced query if available, otherwise fall back to original query
        let query_text = if let Some(ref enhanced) = context.enhanced_query {
            println!("🔍 VectorSearchStep: Using enhanced query: '{}'", enhanced.enhanced);
            &enhanced.enhanced
        } else {
            println!("🔍 VectorSearchStep: Using original query: '{}'", context.request.query.raw);
            &context.request.query.raw
        };
        
        println!("🔍 VectorSearchStep: Generating embedding for query: '{}'", query_text);
        let query_embedding = self.embedding_service.generate_embedding(query_text).await?;
        println!("✅ VectorSearchStep: Generated embedding with {} dimensions", query_embedding.len());
        
        // Search the vector database
        println!("🔍 VectorSearchStep: Searching vector database with limit {}", context.request.limit);
        let vector_results = self.vector_repo.search(query_embedding, context.request.limit).await?;
        println!("📊 VectorSearchStep: Found {} vector results", vector_results.len());
        
        if vector_results.is_empty() {
            println!("⚠️  VectorSearchStep: No results found from vector database!");
        } else {
            for (i, result) in vector_results.iter().take(3).enumerate() {
                println!("📄 VectorSearchStep: Result {}: similarity={:.4}, doc_id={}", i, result.similarity, result.document_id);
            }
        }
        
        // Convert vector results to search results
        let search_results: Vec<SearchResult> = vector_results
            .into_iter()
            .map(|result| SearchResult {
                chunk_id: uuid::Uuid::new_v4(), // Generate a chunk ID
                document_id: result.document_id,
                document_title: result.metadata.title.clone(),
                document_path: result.metadata.custom.get("path").cloned().unwrap_or_default(),
                content: result.metadata.content.clone(),
                snippet: Some({
                    // Create a snippet (first 200 characters)
                    let content = &result.metadata.content;
                    if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.clone()
                    }
                }),
                heading_path: result.metadata.heading_path.clone(),
                final_score: result.similarity,
                ranking_signals: None,
                url: result.metadata.url.clone(),
            })
            .collect();
        
        println!("🎯 VectorSearchStep: Converted to {} search results", search_results.len());
        
        // Set the results in context
        context.set_raw_results(search_results);
        context.metadata.result_sources.push("vector_database".to_string());
        
        Ok(())
    }
}

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
}
