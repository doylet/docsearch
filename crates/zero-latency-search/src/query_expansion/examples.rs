#![cfg(feature = "examples")]

/// Example demonstrating hybrid search with query expansion
/// 
/// This example shows how to combine:
/// 1. Query expansion for enhanced recall
/// 2. Hybrid search (BM25 + Vector) for comprehensive results
/// 3. Score fusion for optimal ranking
use crate::bm25::BM25SearchStep;
use crate::hybrid::HybridSearchStep;
use crate::query_expansion::{QueryExpansionStep, QueryExpansionConfig, ExpansionStrategyFactory};
use crate::vector_search::VectorSearchStep;
use crate::fusion::ScoreFusion;
use crate::models::*;
use crate::traits::SearchStep;

/// Create a complete search pipeline with expansion and hybrid search
pub fn create_enhanced_search_pipeline() -> QueryExpansionStep<HybridSearchStep> {
    // Configure query expansion
    let expansion_config = QueryExpansionConfig {
        max_expansions: 3,
        original_query_weight: 1.0,
        expansion_weight: 0.8,
        enable_synonyms: true,
        enable_morphological: true,
        enable_contextual: false,
        max_terms_per_expansion: 2,
        similarity_threshold: 0.7,
    };
    
    // Create expansion strategies
    let strategies = ExpansionStrategyFactory::create_default();
    
    // Create the underlying hybrid search pipeline
    let bm25_step = BM25SearchStep::new(
        // BM25 adapter would be configured here
        todo!("BM25 adapter configuration")
    );
    
    let vector_step = VectorSearchStep::new(
        // Vector storage would be configured here
        todo!("Vector storage configuration")
    );
    
    let score_fusion = ScoreFusion::default();
    let hybrid_step = HybridSearchStep::new(bm25_step, vector_step, score_fusion);
    
    // Wrap hybrid search with query expansion
    QueryExpansionStep::new(hybrid_step, expansion_config, strategies)
}

/// Example search execution
pub async fn example_search() -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let search_pipeline = create_enhanced_search_pipeline();
    
    let request = SearchRequest::new("algorithm optimization")
        .with_limit(10);
    
    let mut context = SearchContext::new(request);
    
    // Execute the enhanced search pipeline
    search_pipeline.execute(&mut context).await?;
    
    // The context.raw_results now contains:
    // 1. Results from original query "algorithm optimization"
    // 2. Results from synonym expansions like "method improvement", "technique enhancement"
    // 3. Results from morphological variants like "algorithms optimizations", "algorithmic optimize"
    // 4. All combined and deduplicated with weighted scoring
    
    Ok(context.raw_results)
}

/// Example query expansion behavior
pub fn example_expansions() {
    // Query: "algorithm optimization"
    // 
    // Synonym expansions might generate:
    // - "method optimization"
    // - "technique optimization" 
    // - "algorithm improvement"
    //
    // Morphological expansions might generate:
    // - "algorithms optimization"
    // - "algorithm optimizations"
    // - "algorithmic optimization"
    //
    // Each expansion is executed against both BM25 and vector search,
    // results are combined with weighted scoring, and duplicates are merged.
    //
    // Final results include provenance tracking showing which
    // engines and expansions contributed to each result.
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expansion_config() {
        let config = QueryExpansionConfig::default();
        assert_eq!(config.max_expansions, 5);
        assert_eq!(config.original_query_weight, 1.0);
        assert_eq!(config.expansion_weight, 0.7);
        assert!(config.enable_synonyms);
        assert!(config.enable_morphological);
        assert!(!config.enable_contextual);
    }
    
    #[test]
    fn test_expansion_strategies() {
        let strategies = ExpansionStrategyFactory::create_default();
        
        // Verify all strategies are created
        assert!(strategies.contextual.is_some());
    }
}
