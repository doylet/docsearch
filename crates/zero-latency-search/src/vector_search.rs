use crate::{models::*, traits::*};
use async_trait::async_trait;
use std::sync::Arc;
use zero_latency_core::{Result, DocId};
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
            println!(
                "🔍 VectorSearchStep: Using enhanced query: '{}'",
                enhanced.enhanced
            );
            &enhanced.enhanced
        } else {
            println!(
                "🔍 VectorSearchStep: Using original query: '{}'",
                context.request.query.raw
            );
            &context.request.query.raw
        };

        println!(
            "🔍 VectorSearchStep: Generating embedding for query: '{}'",
            query_text
        );
        let query_embedding = self
            .embedding_service
            .generate_embedding(query_text)
            .await?;
        println!(
            "✅ VectorSearchStep: Generated embedding with {} dimensions",
            query_embedding.len()
        );

        // Search the vector database
        println!(
            "🔍 VectorSearchStep: Searching vector database with limit {}",
            context.request.limit
        );

        // Check if collection filter is specified
        let vector_results =
            if let Some(collection_name) = context.request.filters.custom.get("collection") {
                println!(
                    "🔍 VectorSearchStep: Searching in collection: '{}'",
                    collection_name
                );
                self.vector_repo
                    .search_in_collection(collection_name, query_embedding, context.request.limit)
                    .await?
            } else {
                println!("🔍 VectorSearchStep: Searching across all collections");
                self.vector_repo
                    .search(query_embedding, context.request.limit)
                    .await?
            };
        println!(
            "📊 VectorSearchStep: Found {} vector results",
            vector_results.len()
        );

        if vector_results.is_empty() {
            println!("⚠️  VectorSearchStep: No results found from vector database!");
        } else {
            for (i, result) in vector_results.iter().take(3).enumerate() {
                println!(
                    "📄 VectorSearchStep: Result {}: similarity={:.4}, doc_id={}",
                    i, result.similarity, result.document_id
                );
            }
        }

        // Convert vector results to search results
        let search_results: Vec<SearchResult> = vector_results
            .into_iter()
            .map(|result| {
                use crate::fusion::{ScoreBreakdown, FromSignals, NormalizationMethod};
                
                // Create a DocId from the result
                let doc_id = DocId::new(
                    result.metadata.collection.as_deref().unwrap_or("default"),
                    &result.document_id.to_string(),
                    1,
                );
                
                // Create score breakdown for vector-only search
                let similarity_f32 = result.similarity.value();
                let scores = ScoreBreakdown {
                    bm25_raw: None,
                    vector_raw: Some(similarity_f32),
                    bm25_normalized: None,
                    vector_normalized: Some(similarity_f32), // Already normalized in most vector DBs
                    fused: similarity_f32,
                    normalization_method: NormalizationMethod::MinMax,
                };
                
                // Create from_signals tracking
                let from_signals = FromSignals::vector_only();
                
                let uri = result
                    .metadata
                    .custom
                    .get("path")
                    .cloned()
                    .unwrap_or_else(|| format!("doc:{}", result.document_id));
                
                let title = result.metadata.title.clone();
                let content = result.metadata.content.clone();
                
                SearchResult {
                    doc_id,
                    chunk_id: uuid::Uuid::new_v4(),
                    document_id: result.document_id, // Legacy compatibility
                    uri: uri.clone(),
                    title: title.clone(),
                    document_path: uri, // Legacy compatibility
                    content: content.clone(),
                    snippet: Some({
                        // Create a snippet (first 200 characters)
                        if content.len() > 200 {
                            format!("{}...", &content[..200])
                        } else {
                            content.clone()
                        }
                    }),
                    section_path: Vec::new(),
                    heading_path: result.metadata.heading_path.clone(), // Legacy compatibility
                    scores,
                    final_score: result.similarity, // Legacy compatibility
                    from_signals,
                    ranking_signals: None,
                    url: result.metadata.url.clone(),
                    collection: result.metadata.collection.clone(),
                    custom_metadata: result.metadata.custom.clone(),
                }
            })
            .collect();

        println!(
            "🎯 VectorSearchStep: Converted to {} search results",
            search_results.len()
        );

        // Set the results in context
        context.set_raw_results(search_results);
        context
            .metadata
            .result_sources
            .push("vector_database".to_string());

        Ok(())
    }
}

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
}
