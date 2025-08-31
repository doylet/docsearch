use std::collections::HashMap;
use crate::models::SearchResult;
use crate::fusion::deduplication::{ResultDeduplicator, DeduplicationConfig, DuplicationStrategy, DeduplicationMetrics};
use zero_latency_core::error::ZeroLatencyError;

/// Configuration for result merging behavior
#[derive(Debug, Clone)]
pub struct MergerConfig {
    /// Deduplication configuration
    pub deduplication: DeduplicationConfig,
    /// Strategy for handling duplicates
    pub duplication_strategy: DuplicationStrategy,
    /// Maximum number of results to return after merging
    pub max_results: usize,
    /// Whether to preserve query variant provenance
    pub preserve_variant_provenance: bool,
}

impl Default for MergerConfig {
    fn default() -> Self {
        Self {
            deduplication: DeduplicationConfig::default(),
            duplication_strategy: DuplicationStrategy::MergeWithProvenance,
            max_results: 100,
            preserve_variant_provenance: true,
        }
    }
}

/// Metadata about the merging process
#[derive(Debug, Clone)]
pub struct MergeMetrics {
    pub query_variants_processed: usize,
    pub total_results_before_merge: usize,
    pub total_results_after_merge: usize,
    pub deduplication_metrics: DeduplicationMetrics,
    pub processing_time_ms: u64,
    pub variant_contributions: HashMap<String, usize>,
}

/// Represents results from a single query variant
#[derive(Debug, Clone)]
pub struct QueryVariantResults {
    pub variant_id: String,
    pub original_query: String,
    pub results: Vec<SearchResult>,
    pub variant_metadata: Option<serde_json::Value>,
}

/// Core result merger for handling multiple query variants
#[derive(Debug)]
pub struct ResultMerger {
    config: MergerConfig,
    deduplicator: ResultDeduplicator,
}

impl ResultMerger {
    /// Create a new result merger with specified configuration
    pub fn new(config: MergerConfig) -> Self {
        let deduplicator = ResultDeduplicator::new(
            config.deduplication.clone(),
            config.duplication_strategy.clone(),
        );
        
        Self {
            config,
            deduplicator,
        }
    }

    /// Create merger with default configuration
    pub fn with_default() -> Self {
        Self::new(MergerConfig::default())
    }

    /// Merge results from multiple query variants
    pub fn merge_variant_results(
        &self,
        variant_results: Vec<QueryVariantResults>,
    ) -> Result<(Vec<SearchResult>, MergeMetrics), ZeroLatencyError> {
        let start_time = std::time::Instant::now();
        
        if variant_results.is_empty() {
            return Ok((Vec::new(), MergeMetrics {
                query_variants_processed: 0,
                total_results_before_merge: 0,
                total_results_after_merge: 0,
                deduplication_metrics: DeduplicationMetrics {
                    total_input_results: 0,
                    duplicates_found: 0,
                    duplicates_removed: 0,
                    duplicates_merged: 0,
                    final_result_count: 0,
                    similarity_comparisons: 0,
                    processing_time_ms: 0,
                },
                processing_time_ms: 0,
                variant_contributions: HashMap::new(),
            }));
        }

        // Step 1: Collect and count all results
        let (all_results, variant_contributions, total_before) = self.collect_all_results(&variant_results)?;

        // Step 2: Deduplicate and merge results
        let (deduplicated_results, dedup_metrics) = self.deduplicator.deduplicate(all_results)?;

        // Step 3: Apply final ranking and limit
        let final_results = self.apply_final_ranking_and_limit(deduplicated_results)?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        let metrics = MergeMetrics {
            query_variants_processed: variant_results.len(),
            total_results_before_merge: total_before,
            total_results_after_merge: final_results.len(),
            deduplication_metrics: dedup_metrics,
            processing_time_ms: processing_time,
            variant_contributions,
        };

        Ok((final_results, metrics))
    }

    /// Collect all results from variants and track contributions
    fn collect_all_results(
        &self,
        variant_results: &[QueryVariantResults],
    ) -> Result<(Vec<SearchResult>, HashMap<String, usize>, usize), ZeroLatencyError> {
        let mut all_results = Vec::new();
        let mut variant_contributions = HashMap::new();
        let mut total_count = 0;

        for variant in variant_results {
            let variant_count = variant.results.len();
            total_count += variant_count;
            variant_contributions.insert(variant.variant_id.clone(), variant_count);

            for result in variant.results.clone() {
                all_results.push(result);
            }
        }

        Ok((all_results, variant_contributions, total_count))
    }

    /// Apply final ranking and limit results
    fn apply_final_ranking_and_limit(&self, mut results: Vec<SearchResult>) -> Result<Vec<SearchResult>, ZeroLatencyError> {
        // Sort by fused score (descending) with stable tie-breaking
        results.sort_by(|a, b| {
            match b.scores.fused.partial_cmp(&a.scores.fused) {
                Some(std::cmp::Ordering::Equal) | None => {
                    // Tie-break by document ID for determinism
                    a.doc_id.cmp(&b.doc_id)
                }
                Some(other) => other,
            }
        });

        // Limit to max results
        if results.len() > self.config.max_results {
            results.truncate(self.config.max_results);
        }

        Ok(results)
    }

    /// Get current merger configuration
    pub fn get_config(&self) -> &MergerConfig {
        &self.config
    }

    /// Update merger configuration
    pub fn update_config(&mut self, config: MergerConfig) {
        // Update deduplicator configuration
        self.deduplicator.update_config(config.deduplication.clone());
        self.deduplicator.update_strategy(config.duplication_strategy.clone());
        
        self.config = config;
    }

    /// Merge two sets of search results directly
    pub fn merge_two_result_sets(
        &self,
        results1: Vec<SearchResult>,
        results2: Vec<SearchResult>,
    ) -> Result<(Vec<SearchResult>, MergeMetrics), ZeroLatencyError> {
        let variant_results = vec![
            QueryVariantResults {
                variant_id: "variant_1".to_string(),
                original_query: "query_1".to_string(),
                results: results1,
                variant_metadata: None,
            },
            QueryVariantResults {
                variant_id: "variant_2".to_string(),
                original_query: "query_2".to_string(),
                results: results2,
                variant_metadata: None,
            },
        ];

        self.merge_variant_results(variant_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SearchResult;
    use zero_latency_core::{DocId, Uuid};
    use crate::fusion::{FromSignals, ScoreBreakdown};

    fn create_test_result(doc_id: &str, score: f32) -> SearchResult {
        let doc_id = DocId::new("test_collection", doc_id, 1);
        let scores = ScoreBreakdown {
            bm25_raw: Some(score * 0.4),
            vector_raw: Some(score * 0.6),
            bm25_normalized: Some(score * 0.4),
            vector_normalized: Some(score * 0.6),
            fused: score,
            normalization_method: crate::fusion::score_fusion::NormalizationMethod::MinMax,
        };
        
        SearchResult::new(
            doc_id.clone(),
            format!("uri/{}", doc_id),
            format!("Title for {}", doc_id),
            format!("Content for {}", doc_id),
            scores,
            FromSignals::hybrid(),
        )
    }

    #[test]
    fn test_merge_variant_results() {
        let merger = ResultMerger::with_default();
        
        let variant_results = vec![
            QueryVariantResults {
                variant_id: "original".to_string(),
                original_query: "rust programming".to_string(),
                results: vec![
                    create_test_result("doc1", 0.9),
                    create_test_result("doc2", 0.8),
                ],
                variant_metadata: None,
            },
            QueryVariantResults {
                variant_id: "expanded".to_string(),
                original_query: "rust development".to_string(),
                results: vec![
                    create_test_result("doc1", 0.85), // duplicate
                    create_test_result("doc3", 0.7),
                ],
                variant_metadata: None,
            },
        ];

        let (merged_results, metrics) = merger.merge_variant_results(variant_results).unwrap();
        
        assert_eq!(metrics.query_variants_processed, 2);
        assert_eq!(metrics.total_results_before_merge, 4);
        assert!(merged_results.len() <= 3); // After deduplication
    }

    #[test]
    fn test_merge_two_result_sets() {
        let merger = ResultMerger::with_default();
        
        let results1 = vec![
            create_test_result("doc1", 0.9),
            create_test_result("doc2", 0.7),
        ];
        
        let results2 = vec![
            create_test_result("doc2", 0.8), // duplicate with different score
            create_test_result("doc3", 0.6),
        ];

        let (merged, metrics) = merger.merge_two_result_sets(results1, results2).unwrap();
        
        assert_eq!(metrics.query_variants_processed, 2);
        assert!(merged.len() <= 3); // After deduplication
        
        // Check that results are properly ranked
        for i in 1..merged.len() {
            assert!(merged[i-1].scores.fused >= merged[i].scores.fused);
        }
    }
}
