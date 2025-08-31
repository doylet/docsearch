use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zero_latency_core::{DocId, Result};

use crate::evaluation::dataset::{EvaluationDataset, RelevanceRating};
use crate::models::{SearchRequest, SearchResponse, SearchResult};

/// Individual search quality metrics for a single query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    /// Query text
    pub query: String,
    /// Number of relevant documents in ground truth
    pub total_relevant: usize,
    /// Number of results returned
    pub total_returned: usize,
    /// Normalized Discounted Cumulative Gain at K
    pub ndcg_at_k: HashMap<usize, f64>,
    /// Hit rate at K (did we find any relevant docs in top K?)
    pub hit_at_k: HashMap<usize, f64>,
    /// Precision at K
    pub precision_at_k: HashMap<usize, f64>,
    /// Recall at K  
    pub recall_at_k: HashMap<usize, f64>,
    /// Mean Reciprocal Rank
    pub mrr: f64,
    /// Average Precision
    pub average_precision: f64,
}

/// Aggregated metrics across all queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Number of queries evaluated
    pub num_queries: usize,
    /// Mean NDCG@K across all queries
    pub mean_ndcg_at_k: HashMap<usize, f64>,
    /// Mean Hit@K across all queries
    pub mean_hit_at_k: HashMap<usize, f64>,
    /// Mean Precision@K across all queries
    pub mean_precision_at_k: HashMap<usize, f64>,
    /// Mean Recall@K across all queries
    pub mean_recall_at_k: HashMap<usize, f64>,
    /// Mean Reciprocal Rank across all queries
    pub mean_mrr: f64,
    /// Mean Average Precision across all queries
    pub mean_average_precision: f64,
}

/// Complete evaluation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    /// Dataset information
    pub dataset_name: String,
    /// Evaluation timestamp
    pub timestamp: String,
    /// Model/system being evaluated
    pub system_name: String,
    /// Per-query metrics
    pub query_metrics: Vec<QueryMetrics>,
    /// Aggregated metrics
    pub aggregated: AggregatedMetrics,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Metrics calculation engine
pub struct MetricsCalculator {
    /// K values to calculate metrics for
    k_values: Vec<usize>,
}

impl Default for MetricsCalculator {
    fn default() -> Self {
        Self {
            k_values: vec![1, 3, 5, 10, 20],
        }
    }
}

impl MetricsCalculator {
    pub fn new(k_values: Vec<usize>) -> Self {
        Self { k_values }
    }
    
    /// Calculate NDCG@K for a single query
    pub fn calculate_ndcg_at_k(
        &self,
        results: &[SearchResult],
        ground_truth: &[RelevanceRating],
        k: usize,
    ) -> f64 {
        if results.is_empty() || ground_truth.is_empty() {
            return 0.0;
        }
        
        let k = k.min(results.len()).min(ground_truth.len());
        
        // Calculate DCG@K
        let mut dcg = 0.0;
        for i in 0..k {
            let relevance = f64::from(ground_truth[i]);
            if i == 0 {
                dcg += relevance;
            } else {
                dcg += relevance / (i as f64 + 1.0).log2();
            }
        }
        
        // Calculate IDCG@K (perfect ranking)
        let mut sorted_relevance: Vec<f64> = ground_truth.iter().map(|&r| f64::from(r)).collect();
        sorted_relevance.sort_by(|a, b| b.partial_cmp(a).unwrap());
        
        let mut idcg = 0.0;
        for i in 0..k.min(sorted_relevance.len()) {
            let relevance = sorted_relevance[i];
            if i == 0 {
                idcg += relevance;
            } else {
                idcg += relevance / (i as f64 + 1.0).log2();
            }
        }
        
        if idcg == 0.0 {
            0.0
        } else {
            dcg / idcg
        }
    }
    
    /// Calculate Hit@K - did we find any relevant document in top K?
    pub fn calculate_hit_at_k(
        &self,
        ground_truth: &[RelevanceRating],
        k: usize,
    ) -> f64 {
        if ground_truth.is_empty() {
            return 0.0;
        }
        
        let k = k.min(ground_truth.len());
        
        for i in 0..k {
            if ground_truth[i] != RelevanceRating::NotRelevant {
                return 1.0;
            }
        }
        0.0
    }
    
    /// Calculate Precision@K
    pub fn calculate_precision_at_k(
        &self,
        ground_truth: &[RelevanceRating],
        k: usize,
    ) -> f64 {
        if ground_truth.is_empty() {
            return 0.0;
        }
        
        let k = k.min(ground_truth.len());
        let relevant_count = ground_truth[0..k]
            .iter()
            .filter(|&&rating| rating != RelevanceRating::NotRelevant)
            .count();
        
        relevant_count as f64 / k as f64
    }
    
    /// Calculate Recall@K
    pub fn calculate_recall_at_k(
        &self,
        ground_truth: &[RelevanceRating],
        total_relevant: usize,
        k: usize,
    ) -> f64 {
        if total_relevant == 0 {
            return 0.0;
        }
        
        let k = k.min(ground_truth.len());
        let relevant_found = ground_truth[0..k]
            .iter()
            .filter(|&&rating| rating != RelevanceRating::NotRelevant)
            .count();
        
        relevant_found as f64 / total_relevant as f64
    }
    
    /// Calculate Mean Reciprocal Rank
    pub fn calculate_mrr(&self, ground_truth: &[RelevanceRating]) -> f64 {
        for (i, &rating) in ground_truth.iter().enumerate() {
            if rating != RelevanceRating::NotRelevant {
                return 1.0 / (i as f64 + 1.0);
            }
        }
        0.0
    }
    
    /// Calculate Average Precision
    pub fn calculate_average_precision(&self, ground_truth: &[RelevanceRating]) -> f64 {
        let total_relevant = ground_truth
            .iter()
            .filter(|&&rating| rating != RelevanceRating::NotRelevant)
            .count();
        
        if total_relevant == 0 {
            return 0.0;
        }
        
        let mut ap = 0.0;
        let mut relevant_found = 0;
        
        for (i, &rating) in ground_truth.iter().enumerate() {
            if rating != RelevanceRating::NotRelevant {
                relevant_found += 1;
                let precision_at_i = relevant_found as f64 / (i as f64 + 1.0);
                ap += precision_at_i;
            }
        }
        
        ap / total_relevant as f64
    }
    
    /// Calculate metrics for a single query
    pub fn calculate_query_metrics(
        &self,
        query: &str,
        results: &[SearchResult],
        dataset: &EvaluationDataset,
    ) -> Result<QueryMetrics> {
        let ground_truth_examples = dataset.get_examples_for_query(query);
        
        // Build ground truth mapping: doc_id -> relevance
        let mut ground_truth_map: HashMap<DocId, RelevanceRating> = HashMap::new();
        for example in &ground_truth_examples {
            ground_truth_map.insert(example.doc_id.clone(), example.relevance);
        }
        
        // Create relevance vector in result order
        let ground_truth: Vec<RelevanceRating> = results
            .iter()
            .map(|result| {
                ground_truth_map
                    .get(&result.doc_id)
                    .copied()
                    .unwrap_or(RelevanceRating::NotRelevant)
            })
            .collect();
        
        let total_relevant = ground_truth_examples.len();
        
        // Calculate all metrics
        let mut ndcg_at_k = HashMap::new();
        let mut hit_at_k = HashMap::new();
        let mut precision_at_k = HashMap::new();
        let mut recall_at_k = HashMap::new();
        
        for &k in &self.k_values {
            ndcg_at_k.insert(k, self.calculate_ndcg_at_k(results, &ground_truth, k));
            hit_at_k.insert(k, self.calculate_hit_at_k(&ground_truth, k));
            precision_at_k.insert(k, self.calculate_precision_at_k(&ground_truth, k));
            recall_at_k.insert(k, self.calculate_recall_at_k(&ground_truth, total_relevant, k));
        }
        
        let mrr = self.calculate_mrr(&ground_truth);
        let average_precision = self.calculate_average_precision(&ground_truth);
        
        Ok(QueryMetrics {
            query: query.to_string(),
            total_relevant,
            total_returned: results.len(),
            ndcg_at_k,
            hit_at_k,
            precision_at_k,
            recall_at_k,
            mrr,
            average_precision,
        })
    }
    
    /// Aggregate metrics across multiple queries
    pub fn aggregate_metrics(&self, query_metrics: &[QueryMetrics]) -> AggregatedMetrics {
        if query_metrics.is_empty() {
            return AggregatedMetrics {
                num_queries: 0,
                mean_ndcg_at_k: HashMap::new(),
                mean_hit_at_k: HashMap::new(),
                mean_precision_at_k: HashMap::new(),
                mean_recall_at_k: HashMap::new(),
                mean_mrr: 0.0,
                mean_average_precision: 0.0,
            };
        }
        
        let num_queries = query_metrics.len();
        let mut mean_ndcg_at_k = HashMap::new();
        let mut mean_hit_at_k = HashMap::new();
        let mut mean_precision_at_k = HashMap::new();
        let mut mean_recall_at_k = HashMap::new();
        
        // Calculate means for each K value
        for &k in &self.k_values {
            let ndcg_sum: f64 = query_metrics
                .iter()
                .map(|m| m.ndcg_at_k.get(&k).copied().unwrap_or(0.0))
                .sum();
            mean_ndcg_at_k.insert(k, ndcg_sum / num_queries as f64);
            
            let hit_sum: f64 = query_metrics
                .iter()
                .map(|m| m.hit_at_k.get(&k).copied().unwrap_or(0.0))
                .sum();
            mean_hit_at_k.insert(k, hit_sum / num_queries as f64);
            
            let precision_sum: f64 = query_metrics
                .iter()
                .map(|m| m.precision_at_k.get(&k).copied().unwrap_or(0.0))
                .sum();
            mean_precision_at_k.insert(k, precision_sum / num_queries as f64);
            
            let recall_sum: f64 = query_metrics
                .iter()
                .map(|m| m.recall_at_k.get(&k).copied().unwrap_or(0.0))
                .sum();
            mean_recall_at_k.insert(k, recall_sum / num_queries as f64);
        }
        
        let mean_mrr: f64 = query_metrics.iter().map(|m| m.mrr).sum::<f64>() / num_queries as f64;
        let mean_average_precision: f64 = query_metrics
            .iter()
            .map(|m| m.average_precision)
            .sum::<f64>()
            / num_queries as f64;
        
        AggregatedMetrics {
            num_queries,
            mean_ndcg_at_k,
            mean_hit_at_k,
            mean_precision_at_k,
            mean_recall_at_k,
            mean_mrr,
            mean_average_precision,
        }
    }
}

/// CI-friendly regression checker
pub struct RegressionChecker {
    /// Threshold for NDCG@10 regression (e.g., 0.03 = 3%)
    pub ndcg_10_threshold: f64,
}

impl Default for RegressionChecker {
    fn default() -> Self {
        Self {
            ndcg_10_threshold: 0.03, // 3% regression threshold
        }
    }
}

impl RegressionChecker {
    pub fn new(ndcg_10_threshold: f64) -> Self {
        Self { ndcg_10_threshold }
    }
    
    /// Check if current metrics represent a regression from baseline
    pub fn check_regression(
        &self,
        current: &AggregatedMetrics,
        baseline: &AggregatedMetrics,
    ) -> RegressionCheckResult {
        let current_ndcg_10 = current.mean_ndcg_at_k.get(&10).copied().unwrap_or(0.0);
        let baseline_ndcg_10 = baseline.mean_ndcg_at_k.get(&10).copied().unwrap_or(0.0);
        
        let delta = current_ndcg_10 - baseline_ndcg_10;
        let regression_detected = delta < -self.ndcg_10_threshold;
        
        RegressionCheckResult {
            regression_detected,
            current_ndcg_10,
            baseline_ndcg_10,
            delta,
            threshold: self.ndcg_10_threshold,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionCheckResult {
    pub regression_detected: bool,
    pub current_ndcg_10: f64,
    pub baseline_ndcg_10: f64,
    pub delta: f64,
    pub threshold: f64,
}

impl RegressionCheckResult {
    pub fn format_message(&self) -> String {
        if self.regression_detected {
            format!(
                "❌ REGRESSION DETECTED: NDCG@10 dropped by {:.1}% ({:.4} -> {:.4}), exceeds threshold of {:.1}%",
                self.delta.abs() * 100.0,
                self.baseline_ndcg_10,
                self.current_ndcg_10,
                self.threshold * 100.0
            )
        } else if self.delta > 0.0 {
            format!(
                "✅ IMPROVEMENT: NDCG@10 improved by {:.1}% ({:.4} -> {:.4})",
                self.delta * 100.0,
                self.baseline_ndcg_10,
                self.current_ndcg_10
            )
        } else {
            format!(
                "✅ NO REGRESSION: NDCG@10 within acceptable range ({:.4} -> {:.4}, delta: {:.1}%)",
                self.baseline_ndcg_10,
                self.current_ndcg_10,
                self.delta * 100.0
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::dataset::{EvaluationDataset, LabeledExample};
    use zero_latency_core::DocId;

    #[test]
    fn test_ndcg_calculation() {
        let calculator = MetricsCalculator::default();
        
        // Mock results (not used in this specific test, but needed for function signature)
        let results = vec![];
        
        // Perfect ranking: [2, 1, 0, 0]
        let perfect_relevance = vec![
            RelevanceRating::HighlyRelevant,
            RelevanceRating::SomewhatRelevant,
            RelevanceRating::NotRelevant,
            RelevanceRating::NotRelevant,
        ];
        
        let ndcg = calculator.calculate_ndcg_at_k(&results, &perfect_relevance, 4);
        assert!((ndcg - 1.0).abs() < 1e-6); // Should be perfect score
        
        // Reverse ranking: [0, 0, 1, 2]
        let reverse_relevance = vec![
            RelevanceRating::NotRelevant,
            RelevanceRating::NotRelevant,
            RelevanceRating::SomewhatRelevant,
            RelevanceRating::HighlyRelevant,
        ];
        
        let ndcg_reverse = calculator.calculate_ndcg_at_k(&results, &reverse_relevance, 4);
        assert!(ndcg_reverse < ndcg); // Should be worse than perfect
    }

    #[test]
    fn test_precision_recall() {
        let calculator = MetricsCalculator::default();
        
        let relevance = vec![
            RelevanceRating::HighlyRelevant,
            RelevanceRating::NotRelevant,
            RelevanceRating::SomewhatRelevant,
        ];
        
        let precision_at_3 = calculator.calculate_precision_at_k(&relevance, 3);
        assert!((precision_at_3 - 2.0/3.0).abs() < 1e-6); // 2 relevant out of 3
        
        let recall_at_3 = calculator.calculate_recall_at_k(&relevance, 2, 3);
        assert!((recall_at_3 - 1.0).abs() < 1e-6); // Found all 2 relevant docs
    }

    #[test]
    fn test_mrr() {
        let calculator = MetricsCalculator::default();
        
        let relevance = vec![
            RelevanceRating::NotRelevant,
            RelevanceRating::HighlyRelevant, // First relevant at position 2
            RelevanceRating::SomewhatRelevant,
        ];
        
        let mrr = calculator.calculate_mrr(&relevance);
        assert!((mrr - 0.5).abs() < 1e-6); // 1/2 = 0.5
    }

    #[test]
    fn test_regression_checker() {
        let checker = RegressionChecker::new(0.05); // 5% threshold
        
        let mut baseline = AggregatedMetrics {
            num_queries: 10,
            mean_ndcg_at_k: HashMap::new(),
            mean_hit_at_k: HashMap::new(),
            mean_precision_at_k: HashMap::new(),
            mean_recall_at_k: HashMap::new(),
            mean_mrr: 0.5,
            mean_average_precision: 0.6,
        };
        baseline.mean_ndcg_at_k.insert(10, 0.8);
        
        let mut current = baseline.clone();
        current.mean_ndcg_at_k.insert(10, 0.72); // 8% drop
        
        let result = checker.check_regression(&current, &baseline);
        assert!(result.regression_detected); // Should detect regression
        
        // Test no regression
        current.mean_ndcg_at_k.insert(10, 0.79); // 1% drop
        let result = checker.check_regression(&current, &baseline);
        assert!(!result.regression_detected); // Should not detect regression
    }
}
