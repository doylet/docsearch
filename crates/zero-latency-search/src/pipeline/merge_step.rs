use crate::models::SearchResult;
use crate::fusion::result_merger::{ResultMerger, MergerConfig, QueryVariantResults, MergeMetrics};
use zero_latency_core::error::ZeroLatencyError;

/// Configuration for the merge step
#[derive(Debug, Clone)]
pub struct MergeStepConfig {
    /// Configuration for result merging
    pub merger_config: MergerConfig,
    /// Whether to enable merge step processing
    pub enabled: bool,
    /// Maximum number of variants to process
    pub max_variants: usize,
}

impl Default for MergeStepConfig {
    fn default() -> Self {
        Self {
            merger_config: MergerConfig::default(),
            enabled: true,
            max_variants: 10,
        }
    }
}

/// A search step that merges results from multiple query variants
/// This step is typically used after query expansion to deduplicate and merge
/// results from various query reformulations.
#[derive(Debug)]
pub struct MergeStep {
    config: MergeStepConfig,
    merger: ResultMerger,
    step_name: String,
}

impl MergeStep {
    /// Create a new merge step with specified configuration
    pub fn new(config: MergeStepConfig) -> Self {
        let merger = ResultMerger::new(config.merger_config.clone());
        
        Self {
            config,
            merger,
            step_name: "merge_step".to_string(),
        }
    }

    /// Create merge step with default configuration
    pub fn with_default() -> Self {
        Self::new(MergeStepConfig::default())
    }

    /// Create merge step with custom name
    pub fn with_name(config: MergeStepConfig, name: String) -> Self {
        let merger = ResultMerger::new(config.merger_config.clone());
        
        Self {
            config,
            merger,
            step_name: name,
        }
    }

    /// Get step name
    pub fn name(&self) -> &str {
        &self.step_name
    }

    /// Process multiple search results from different query variants
    pub fn merge_variant_results(
        &self,
        variant_results: Vec<QueryVariantResults>,
    ) -> Result<(Vec<SearchResult>, MergeMetrics), ZeroLatencyError> {
        if !self.config.enabled {
            // If merge step is disabled, just combine all results
            let mut all_results = Vec::new();
            let mut total_before = 0;
            
            for variant in &variant_results {
                total_before += variant.results.len();
                all_results.extend(variant.results.clone());
            }
            
            // Sort by score and return
            all_results.sort_by(|a, b| b.scores.fused.partial_cmp(&a.scores.fused)
                .unwrap_or(std::cmp::Ordering::Equal));
            
            let metrics = MergeMetrics {
                query_variants_processed: variant_results.len(),
                total_results_before_merge: total_before,
                total_results_after_merge: all_results.len(),
                deduplication_metrics: crate::fusion::deduplication::DeduplicationMetrics {
                    total_input_results: total_before,
                    duplicates_found: 0,
                    duplicates_removed: 0,
                    duplicates_merged: 0,
                    final_result_count: all_results.len(),
                    similarity_comparisons: 0,
                    processing_time_ms: 0,
                },
                processing_time_ms: 0,
                variant_contributions: std::collections::HashMap::new(),
            };
            
            return Ok((all_results, metrics));
        }

        // Limit number of variants if configured
        let limited_variants = if variant_results.len() > self.config.max_variants {
            variant_results.into_iter().take(self.config.max_variants).collect()
        } else {
            variant_results
        };

        self.merger.merge_variant_results(limited_variants)
    }

    /// Get configuration
    pub fn get_config(&self) -> &MergeStepConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MergeStepConfig) {
        self.merger.update_config(config.merger_config.clone());
        self.config = config;
    }
}

/// Builder for creating merge steps with fluent configuration
#[derive(Debug, Default)]
pub struct MergeStepBuilder {
    config: MergeStepConfig,
    name: Option<String>,
}

impl MergeStepBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether merge step is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set maximum number of variants to process
    pub fn max_variants(mut self, max_variants: usize) -> Self {
        self.config.max_variants = max_variants;
        self
    }

    /// Set merger configuration
    pub fn merger_config(mut self, merger_config: MergerConfig) -> Self {
        self.config.merger_config = merger_config;
        self
    }

    /// Set step name
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Build the merge step
    pub fn build(self) -> MergeStep {
        match self.name {
            Some(name) => MergeStep::with_name(self.config, name),
            None => MergeStep::new(self.config),
        }
    }
}

/// Utility for creating common merge step configurations
pub struct MergeStepPresets;

impl MergeStepPresets {
    /// Create a merge step optimized for query expansion scenarios
    pub fn for_query_expansion() -> MergeStep {
        let config = MergeStepConfig {
            merger_config: MergerConfig {
                duplication_strategy: crate::fusion::deduplication::DuplicationStrategy::MergeWithProvenance,
                max_results: 50,
                preserve_variant_provenance: true,
                ..Default::default()
            },
            enabled: true,
            max_variants: 5,
        };
        
        MergeStep::with_name(config, "query_expansion_merge".to_string())
    }

    /// Create a merge step optimized for multi-collection search
    pub fn for_multi_collection() -> MergeStep {
        let config = MergeStepConfig {
            merger_config: MergerConfig {
                duplication_strategy: crate::fusion::deduplication::DuplicationStrategy::RemoveKeepBest,
                max_results: 100,
                preserve_variant_provenance: false,
                ..Default::default()
            },
            enabled: true,
            max_variants: 10,
        };
        
        MergeStep::with_name(config, "multi_collection_merge".to_string())
    }

    /// Create a simple merge step that just deduplicates without complex merging
    pub fn simple_deduplication() -> MergeStep {
        let config = MergeStepConfig {
            merger_config: MergerConfig {
                duplication_strategy: crate::fusion::deduplication::DuplicationStrategy::RemoveKeepFirst,
                max_results: 200,
                preserve_variant_provenance: false,
                ..Default::default()
            },
            enabled: true,
            max_variants: 20,
        };
        
        MergeStep::with_name(config, "simple_deduplication".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SearchResult;
    use zero_latency_core::DocId;
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
    fn test_merge_step_basic_functionality() {
        let merge_step = MergeStep::with_default();
        
        let variant_results = vec![
            QueryVariantResults {
                variant_id: "variant1".to_string(),
                original_query: "test query".to_string(),
                results: vec![
                    create_test_result("doc1", 0.9),
                    create_test_result("doc2", 0.7),
                ],
                variant_metadata: None,
            },
            QueryVariantResults {
                variant_id: "variant2".to_string(),
                original_query: "test query expanded".to_string(),
                results: vec![
                    create_test_result("doc1", 0.8), // duplicate
                    create_test_result("doc3", 0.6),
                ],
                variant_metadata: None,
            },
        ];

        let (merged_results, metrics) = merge_step.merge_variant_results(variant_results).unwrap();
        
        assert_eq!(metrics.query_variants_processed, 2);
        assert!(merged_results.len() <= 3);
    }

    #[test]
    fn test_merge_step_disabled() {
        let config = MergeStepConfig {
            enabled: false,
            ..Default::default()
        };
        let merge_step = MergeStep::new(config);
        
        let variant_results = vec![
            QueryVariantResults {
                variant_id: "variant1".to_string(),
                original_query: "test".to_string(),
                results: vec![create_test_result("doc1", 0.9)],
                variant_metadata: None,
            },
        ];

        let (results, metrics) = merge_step.merge_variant_results(variant_results).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(metrics.deduplication_metrics.duplicates_found, 0);
    }

    #[test]
    fn test_merge_step_builder() {
        let merge_step = MergeStepBuilder::new()
            .enabled(true)
            .max_variants(5)
            .name("custom_merge".to_string())
            .build();
        
        assert_eq!(merge_step.name(), "custom_merge");
        assert!(merge_step.config.enabled);
        assert_eq!(merge_step.config.max_variants, 5);
    }

    #[test]
    fn test_preset_configurations() {
        let query_expansion_merge = MergeStepPresets::for_query_expansion();
        assert_eq!(query_expansion_merge.name(), "query_expansion_merge");
        
        let multi_collection_merge = MergeStepPresets::for_multi_collection();
        assert_eq!(multi_collection_merge.name(), "multi_collection_merge");
        
        let simple_dedup = MergeStepPresets::simple_deduplication();
        assert_eq!(simple_dedup.name(), "simple_deduplication");
    }

    #[test]
    fn test_max_variants_limiting() {
        let config = MergeStepConfig {
            max_variants: 2,
            ..Default::default()
        };
        let merge_step = MergeStep::new(config);
        
        let variant_results = vec![
            QueryVariantResults {
                variant_id: "v1".to_string(),
                original_query: "q1".to_string(),
                results: vec![create_test_result("doc1", 0.9)],
                variant_metadata: None,
            },
            QueryVariantResults {
                variant_id: "v2".to_string(),
                original_query: "q2".to_string(),
                results: vec![create_test_result("doc2", 0.8)],
                variant_metadata: None,
            },
            QueryVariantResults {
                variant_id: "v3".to_string(),
                original_query: "q3".to_string(),
                results: vec![create_test_result("doc3", 0.7)],
                variant_metadata: None,
            },
        ];

        let (_, metrics) = merge_step.merge_variant_results(variant_results).unwrap();
        
        // Should only process first 2 variants due to max_variants limit
        assert_eq!(metrics.query_variants_processed, 2);
    }
}
