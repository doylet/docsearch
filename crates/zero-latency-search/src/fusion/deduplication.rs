use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use crate::models::SearchResult;
use zero_latency_core::error::ZeroLatencyError;

/// Configuration for result deduplication behavior
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    /// Similarity threshold for considering results duplicates (0.0-1.0)
    pub similarity_threshold: f64,
    /// Whether to preserve ranking stability across document reorderings
    pub stable_ranking: bool,
    /// Maximum number of results to consider for deduplication
    pub max_results: usize,
    /// Whether to enable content-based similarity detection
    pub content_similarity_enabled: bool,
    /// Minimum content length to apply similarity detection
    pub min_content_length: usize,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            stable_ranking: true,
            max_results: 1000,
            content_similarity_enabled: true,
            min_content_length: 50,
        }
    }
}

/// Strategy for handling duplicate results
#[derive(Debug, Clone)]
pub enum DuplicationStrategy {
    /// Remove duplicates, keeping the highest scoring instance
    RemoveKeepBest,
    /// Remove duplicates, keeping the first encountered instance
    RemoveKeepFirst,
    /// Merge duplicate results, combining scores
    MergeScores,
    /// Merge duplicates, preserving all provenance information
    MergeWithProvenance,
}

/// Metadata about deduplication process
#[derive(Debug, Clone)]
pub struct DeduplicationMetrics {
    pub total_input_results: usize,
    pub duplicates_found: usize,
    pub duplicates_removed: usize,
    pub duplicates_merged: usize,
    pub final_result_count: usize,
    pub similarity_comparisons: usize,
    pub processing_time_ms: u64,
}

/// Core deduplication engine
#[derive(Debug)]
pub struct ResultDeduplicator {
    config: DeduplicationConfig,
    strategy: DuplicationStrategy,
}

impl ResultDeduplicator {
    /// Create a new deduplicator with specified configuration
    pub fn new(config: DeduplicationConfig, strategy: DuplicationStrategy) -> Self {
        Self { config, strategy }
    }

    /// Create deduplicator with default configuration
    pub fn with_default() -> Self {
        Self::new(DeduplicationConfig::default(), DuplicationStrategy::MergeWithProvenance)
    }

    /// Deduplicate a list of search results
    pub fn deduplicate(
        &self,
        results: Vec<SearchResult>,
    ) -> Result<(Vec<SearchResult>, DeduplicationMetrics), ZeroLatencyError> {
        let start_time = std::time::Instant::now();
        let total_input = results.len();
        
        if results.is_empty() {
            return Ok((results, DeduplicationMetrics {
                total_input_results: 0,
                duplicates_found: 0,
                duplicates_removed: 0,
                duplicates_merged: 0,
                final_result_count: 0,
                similarity_comparisons: 0,
                processing_time_ms: 0,
            }));
        }

        // Simple deduplication by document ID
        let mut seen_ids = HashSet::new();
        let mut deduplicated = Vec::new();
        let mut similarity_comparisons = 0;

        for result in results {
            similarity_comparisons += 1;
            
            // Simple ID-based deduplication
            let key = self.get_deduplication_key(&result);
            
            if !seen_ids.contains(&key) {
                seen_ids.insert(key);
                deduplicated.push(result);
            }
        }

        // Apply stable ranking if configured
        if self.config.stable_ranking {
            self.apply_stable_ranking(&mut deduplicated)?;
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let final_count = deduplicated.len();
        
        let metrics = DeduplicationMetrics {
            total_input_results: total_input,
            duplicates_found: total_input - final_count,
            duplicates_removed: total_input - final_count,
            duplicates_merged: 0,
            final_result_count: final_count,
            similarity_comparisons,
            processing_time_ms: processing_time,
        };

        Ok((deduplicated, metrics))
    }

    /// Get deduplication key for a result
    fn get_deduplication_key(&self, result: &SearchResult) -> String {
        // Primary deduplication by document ID
        format!("{}", result.doc_id)
    }

    /// Apply stable ranking with deterministic tie-breaking
    fn apply_stable_ranking(&self, results: &mut [SearchResult]) -> Result<(), ZeroLatencyError> {
        results.sort_by(|a, b| {
            // Primary sort: fused score (descending)
            match b.scores.fused.partial_cmp(&a.scores.fused) {
                Some(Ordering::Equal) | None => {
                    // Secondary sort: document ID for determinism
                    a.doc_id.cmp(&b.doc_id)
                }
                Some(other) => other,
            }
        });

        Ok(())
    }

    /// Get deduplication configuration
    pub fn get_config(&self) -> &DeduplicationConfig {
        &self.config
    }

    /// Update deduplication configuration
    pub fn update_config(&mut self, config: DeduplicationConfig) {
        self.config = config;
    }

    /// Update deduplication strategy
    pub fn update_strategy(&mut self, strategy: DuplicationStrategy) {
        self.strategy = strategy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SearchResult;
    use zero_latency_core::{DocId, values::Score, Uuid};
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
    fn test_deduplication_exact_matches() {
        let deduplicator = ResultDeduplicator::with_default();
        
        let results = vec![
            create_test_result("doc1", 0.9),
            create_test_result("doc1", 0.8), // duplicate
            create_test_result("doc2", 0.7),
        ];

        let (deduplicated, metrics) = deduplicator.deduplicate(results).unwrap();
        
        assert_eq!(deduplicated.len(), 2);
        assert_eq!(metrics.duplicates_found, 1);
    }

    #[test]
    fn test_stable_ranking() {
        let deduplicator = ResultDeduplicator::with_default();
        
        let results = vec![
            create_test_result("doc3", 0.7),
            create_test_result("doc1", 0.9),
            create_test_result("doc2", 0.8),
        ];

        let (deduplicated, _) = deduplicator.deduplicate(results).unwrap();
        
        // Should be sorted by score (descending)
        assert!(deduplicated[0].scores.fused >= deduplicated[1].scores.fused);
        assert!(deduplicated[1].scores.fused >= deduplicated[2].scores.fused);
    }
}
