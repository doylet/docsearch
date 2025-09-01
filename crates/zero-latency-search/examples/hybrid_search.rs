//! Hybrid Search Usage Example
//!
//! This example demonstrates how to use the hybrid search capabilities
//! combining vector and BM25 search with result fusion.

use zero_latency_search::{
    models::{SearchRequest, SearchFilters, SearchOptions},
    fusion::FusionConfig,
};
use zero_latency_core::{Result, values::SearchQuery};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Hybrid Search Usage Example");
    println!("==============================");

    // Example search request using the actual API
    let search_request = SearchRequest {
        query: SearchQuery::new("machine learning algorithms"),
        limit: 5,
        offset: 0,
        filters: SearchFilters::default(),
        options: SearchOptions::default(),
    };

    println!("\n🔍 Example search configuration:");
    println!("Query: '{}'", search_request.query.raw);
    println!("Limit: {}", search_request.limit);
    println!("Include snippets: {}", search_request.options.include_snippets);

    // Demonstrate fusion configuration
    let fusion_config = FusionConfig::default();
    println!("\n⚖️ Score fusion configuration:");
    println!("BM25 weight: {:.1}", fusion_config.bm25_weight);
    println!("Vector weight: {:.1}", fusion_config.vector_weight);

    // Note: This is a usage example - actual search execution would require
    // a properly configured search service instance
    println!("\n📝 Usage Notes:");
    println!("• Use SearchRequest::new() for simple queries");
    println!("• Configure SearchFilters for document type filtering");
    println!("• Enable query enhancement in SearchOptions");
    println!("• Score fusion combines vector and BM25 results");

    println!("\n🎉 Hybrid search API example complete!");
    Ok(())
}
