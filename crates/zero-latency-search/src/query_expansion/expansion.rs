use crate::models::*;
use crate::traits::SearchStep;
use crate::query_expansion::strategies::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use zero_latency_core::{error::ZeroLatencyError, values::Score, Result};

/// Configuration for query expansion strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExpansionConfig {
    /// Maximum number of expanded queries to generate
    pub max_expansions: usize,
    /// Weight applied to original query vs expanded queries
    pub original_query_weight: f32,
    /// Weight applied to expanded query results
    pub expansion_weight: f32,
    /// Enable synonym expansion
    pub enable_synonyms: bool,
    /// Enable morphological variants (stemming)
    pub enable_morphological: bool,
    /// Enable contextual expansions
    pub enable_contextual: bool,
    /// Maximum terms to add per expansion type
    pub max_terms_per_expansion: usize,
    /// Minimum similarity threshold for expansion terms
    pub similarity_threshold: f32,
}

impl Default for QueryExpansionConfig {
    fn default() -> Self {
        Self {
            max_expansions: 5,
            original_query_weight: 1.0,
            expansion_weight: 0.7,
            enable_synonyms: true,
            enable_morphological: true,
            enable_contextual: false, // Requires more sophisticated NLP
            max_terms_per_expansion: 3,
            similarity_threshold: 0.6,
        }
    }
}

/// Represents an expanded query variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedQuery {
    /// The expanded query text
    pub query: String,
    /// Type of expansion used
    pub expansion_type: ExpansionType,
    /// Weight to apply to results from this query
    pub weight: f32,
    /// Source terms that led to this expansion
    pub source_terms: Vec<String>,
    /// Added terms in this expansion
    pub added_terms: Vec<String>,
}

/// Types of query expansion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExpansionType {
    /// Original query (no expansion)
    Original,
    /// Synonym-based expansion
    Synonym,
    /// Morphological variants (stemming, plurals, etc.)
    Morphological,
    /// Contextual expansion based on document corpus
    Contextual,
    /// Combination of multiple expansion types
    Combined,
}

/// Query expansion step that generates multiple query variants
pub struct QueryExpansionStep<T: SearchStep> {
    /// The underlying search step to execute for each expanded query
    inner_search: T,
    /// Configuration for expansion behavior
    config: QueryExpansionConfig,
    /// Strategy implementations for different expansion types
    strategies: ExpansionStrategies,
}

/// Collection of expansion strategy implementations
pub struct ExpansionStrategies {
    /// Synonym expansion strategy
    pub synonym: Box<dyn SynonymExpansion + Send + Sync>,
    /// Morphological expansion strategy  
    pub morphological: Box<dyn MorphologicalExpansion + Send + Sync>,
    /// Contextual expansion strategy
    pub contextual: Option<Box<dyn ContextualExpansion + Send + Sync>>,
}

impl<T: SearchStep> QueryExpansionStep<T> {
    /// Create a new query expansion step
    pub fn new(
        inner_search: T,
        config: QueryExpansionConfig,
        strategies: ExpansionStrategies,
    ) -> Self {
        Self {
            inner_search,
            config,
            strategies,
        }
    }

    /// Generate expanded queries from the original query
    pub async fn generate_expansions(
        &self,
        original_query: &str,
    ) -> Result<Vec<ExpandedQuery>> {
        let mut expansions = Vec::new();
        
        // Always include the original query with full weight
        expansions.push(ExpandedQuery {
            query: original_query.to_string(),
            expansion_type: ExpansionType::Original,
            weight: self.config.original_query_weight,
            source_terms: original_query.split_whitespace().map(|s| s.to_string()).collect(),
            added_terms: vec![],
        });

        // Generate synonym expansions
        if self.config.enable_synonyms {
            match self.strategies.synonym.expand(original_query, &self.config).await {
                Ok(mut synonym_expansions) => {
                    expansions.append(&mut synonym_expansions);
                }
                Err(e) => {
                    warn!("Synonym expansion failed: {}", e);
                }
            }
        }

        // Generate morphological expansions
        if self.config.enable_morphological {
            match self.strategies.morphological.expand(original_query, &self.config).await {
                Ok(mut morphological_expansions) => {
                    expansions.append(&mut morphological_expansions);
                }
                Err(e) => {
                    warn!("Morphological expansion failed: {}", e);
                }
            }
        }

        // Generate contextual expansions (if available)
        if self.config.enable_contextual {
            if let Some(ref contextual) = self.strategies.contextual {
                match contextual.expand(original_query, &self.config).await {
                    Ok(mut contextual_expansions) => {
                        expansions.append(&mut contextual_expansions);
                    }
                    Err(e) => {
                        warn!("Contextual expansion failed: {}", e);
                    }
                }
            }
        }

        // Limit total expansions
        expansions.truncate(self.config.max_expansions + 1); // +1 for original

        info!(
            "Generated {} query expansions for query: {}",
            expansions.len() - 1, // -1 for original
            original_query
        );

        Ok(expansions)
    }

    /// Combine and deduplicate results from multiple expanded queries
    fn combine_expansion_results(
        &self,
        results_by_expansion: Vec<(ExpandedQuery, Vec<SearchResult>)>,
    ) -> Vec<SearchResult> {
        let mut combined_results: HashMap<String, SearchResult> = HashMap::new();
        let mut doc_scores: HashMap<String, f32> = HashMap::new();

        for (expansion, results) in results_by_expansion {
            debug!(
                "Processing {} results from {} expansion: {}",
                results.len(),
                expansion.expansion_type,
                expansion.query
            );

            for mut result in results {
                let doc_key = result.doc_id.to_string();
                
                // Apply expansion weight to the score
                let weighted_score = result.scores.fused * expansion.weight;
                
                match combined_results.get_mut(&doc_key) {
                    Some(existing_result) => {
                        // Document already exists, combine scores
                        let current_score = doc_scores.get(&doc_key).unwrap_or(&0.0);
                        let new_score = current_score + weighted_score;
                        doc_scores.insert(doc_key.clone(), new_score);
                        existing_result.scores.fused = new_score;
                        existing_result.final_score = Score::new(new_score).unwrap_or_else(|_| Score::zero());

                        // Update from_signals for query expansion
                        existing_result.from_signals.query_expansion = true;
                    }
                    None => {
                        // New document
                        result.scores.fused = weighted_score;
                        result.final_score = Score::new(weighted_score).unwrap_or_else(|_| Score::zero());
                        doc_scores.insert(doc_key.clone(), weighted_score);

                        // Add expansion information to from_signals
                        result.from_signals.query_expansion = true;

                        combined_results.insert(doc_key, result);
                    }
                }
            }
        }

        // Convert back to vector and sort by combined score
        let mut final_results: Vec<SearchResult> = combined_results.into_values().collect();
        final_results.sort_by(|a, b| b.scores.fused.partial_cmp(&a.scores.fused).unwrap_or(std::cmp::Ordering::Equal));

        info!("Combined expansion results: {} unique documents", final_results.len());
        final_results
    }
}

#[async_trait]
impl<T: SearchStep> SearchStep for QueryExpansionStep<T> {
    fn name(&self) -> &str {
        "query_expansion"
    }

    async fn execute(&self, context: &mut SearchContext) -> Result<()> {
        let original_query = &context.request.query.raw;
        
        // Generate expanded queries
        let expansions = self.generate_expansions(original_query).await
            .map_err(|e| ZeroLatencyError::internal(e.to_string()))?;
        
        // Execute search for each expanded query
        let mut results_by_expansion = Vec::new();
        
        for expansion in expansions {
            // Create modified context for this expansion
            let mut expanded_context = context.clone();
            expanded_context.request.query.raw = expansion.query.clone();
            expanded_context.raw_results.clear(); // Clear previous results
            
            // Execute search with the inner search step
            match self.inner_search.execute(&mut expanded_context).await {
                Ok(_) => {
                    debug!(
                        "Expansion '{}' returned {} results",
                        expansion.query,
                        expanded_context.raw_results.len()
                    );
                    results_by_expansion.push((expansion, expanded_context.raw_results));
                }
                Err(e) => {
                    warn!(
                        "Search failed for expansion '{}': {}",
                        expansion.query, e
                    );
                    // Continue with other expansions
                }
            }
        }
        
        // Combine and deduplicate results
        let combined_results = self.combine_expansion_results(results_by_expansion);
        
        // Apply original limit and update context
        context.raw_results = combined_results
            .into_iter()
            .take(context.request.limit)
            .collect();
            
        info!(
            "Query expansion completed: {} final results for query '{}'",
            context.raw_results.len(),
            original_query
        );
        
        Ok(())
    }
}

impl std::fmt::Display for ExpansionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpansionType::Original => write!(f, "original"),
            ExpansionType::Synonym => write!(f, "synonym"),
            ExpansionType::Morphological => write!(f, "morphological"),
            ExpansionType::Contextual => write!(f, "contextual"),
            ExpansionType::Combined => write!(f, "combined"),
        }
    }
}

/// Error type for query expansion operations
#[derive(Debug, thiserror::Error)]
pub enum ExpansionError {
    #[error("Synonym lookup failed: {0}")]
    SynonymLookup(String),
    #[error("Morphological analysis failed: {0}")]
    MorphologicalAnalysis(String),
    #[error("Contextual expansion failed: {0}")]
    ContextualExpansion(String),
    #[error("Too many expansion terms: {0}")]
    TooManyTerms(usize),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl From<ExpansionError> for ZeroLatencyError {
    fn from(e: ExpansionError) -> Self {
        ZeroLatencyError::internal(e.to_string())
    }
}
