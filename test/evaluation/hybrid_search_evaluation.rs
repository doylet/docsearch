//! Comprehensive evaluation framework for hybrid search quality assessment
//!
//! This module implements A/B testing and statistical significance validation
//! for hybrid BM25 + vector search vs baseline vector-only search.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use zero_latency_core::{Result, ZeroLatencyError};
use zero_latency_search::evaluation::{
    dataset::EvaluationDataset,
    metrics::{EvaluationReport, MetricsCalculator, QueryMetrics},
};

/// Statistical significance test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificanceTest {
    /// Test method used (e.g., "randomization_test", "mann_whitney_u")
    pub method: String,
    /// P-value from the statistical test
    pub p_value: f64,
    /// Is the difference statistically significant? (p < 0.05)
    pub is_significant: bool,
    /// Confidence level used (default 0.95 for 95% confidence)
    pub confidence_level: f64,
    /// Effect size (Cohen's d or similar)
    pub effect_size: f64,
    /// Confidence interval for the metric difference
    pub confidence_interval: (f64, f64),
}

/// Per-query comparison between two search systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryComparison {
    /// Query text
    pub query: String,
    /// Query category for grouping
    pub category: Option<String>,
    /// Baseline system metrics
    pub baseline_metrics: QueryMetrics,
    /// Test system metrics  
    pub test_metrics: QueryMetrics,
    /// Per-metric deltas (test - baseline)
    pub metric_deltas: HashMap<String, f64>,
    /// Performance comparison
    pub performance_comparison: PerformanceComparison,
}

/// Performance metrics comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    /// Search latency in milliseconds
    pub baseline_latency_ms: f64,
    pub test_latency_ms: f64,
    /// Token usage (if applicable)
    pub baseline_tokens: Option<u32>,
    pub test_tokens: Option<u32>,
    /// Memory usage (if measurable)
    pub baseline_memory_mb: Option<f64>,
    pub test_memory_mb: Option<f64>,
}

/// A/B test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    /// Dataset path
    pub dataset_path: String,
    /// Baseline system name
    pub baseline_name: String,
    /// Test system name
    pub test_name: String,
    /// Number of bootstrap samples for significance testing
    pub bootstrap_samples: usize,
    /// Confidence level for statistical tests
    pub confidence_level: f64,
    /// Timeout for individual search requests (ms)
    pub search_timeout_ms: u64,
    /// Enable multi-query expansion testing
    pub enable_multi_query_expansion: bool,
    /// Random seed for reproducibility
    pub random_seed: Option<u64>,
    /// K values for metrics calculation
    pub k_values: Vec<usize>,
    /// BM25 search parameters
    pub bm25_k: usize,
    /// Vector search parameters
    pub vector_k: usize,
    pub similarity_threshold: f64,
    /// Score fusion weights
    pub fusion_weights: FusionWeights,
}

/// Score fusion weights for hybrid search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionWeights {
    pub bm25_weight: f64,
    pub vector_weight: f64,
}

impl Default for ABTestConfig {
    fn default() -> Self {
        Self {
            dataset_path: "test/evaluation/labeled_dataset.json".to_string(),
            baseline_name: "vector_only".to_string(),
            test_name: "hybrid_bm25_vector".to_string(),
            bootstrap_samples: 10000,
            confidence_level: 0.95,
            search_timeout_ms: 1000,
            enable_multi_query_expansion: true,
            random_seed: Some(42),
            k_values: vec![1, 3, 5, 10, 20],
            bm25_k: 20,
            vector_k: 20,
            similarity_threshold: 0.7,
            fusion_weights: FusionWeights {
                bm25_weight: 0.3,
                vector_weight: 0.7,
            },
        }
    }
}

/// Complete A/B test report with statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestReport {
    /// Test configuration used
    pub config: ABTestConfig,
    /// Test execution timestamp
    pub timestamp: String,
    /// Baseline system evaluation report
    pub baseline_report: EvaluationReport,
    /// Test system evaluation report
    pub test_report: EvaluationReport,
    /// Per-query comparisons
    pub query_comparisons: Vec<QueryComparison>,
    /// Statistical significance tests for key metrics
    pub significance_tests: HashMap<String, SignificanceTest>,
    /// Overall performance summary
    pub performance_summary: PerformanceSummary,
    /// Recommendation and conclusion
    pub recommendation: TestRecommendation,
}

/// Overall performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total queries evaluated
    pub total_queries: usize,
    /// Queries with improved NDCG@10
    pub queries_improved_ndcg: usize,
    /// Queries with degraded NDCG@10
    pub queries_degraded_ndcg: usize,
    /// Mean NDCG@10 improvement
    pub mean_ndcg10_improvement: f64,
    /// Mean recall improvement
    pub mean_recall_improvement: f64,
    /// Mean latency change (ms)
    pub mean_latency_change_ms: f64,
    /// P95 latency for test system
    pub p95_latency_ms: f64,
    /// Success rate (queries without errors)
    pub success_rate: f64,
}

/// Test recommendation based on results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRecommendation {
    /// Should we deploy the test system?
    pub deploy_recommendation: DeployRecommendation,
    /// Key findings summary
    pub key_findings: Vec<String>,
    /// Risks and concerns
    pub risks: Vec<String>,
    /// Next steps and follow-up actions
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeployRecommendation {
    StronglyRecommend,
    Recommend,
    Conditional(String), // Condition for deployment
    NotRecommend(String), // Reason for not recommending
    StronglyNotRecommend(String), // Strong reason against
}

/// Hybrid search evaluation runner
pub struct HybridSearchEvaluator {
    config: ABTestConfig,
    metrics_calculator: MetricsCalculator,
}

impl HybridSearchEvaluator {
    pub fn new(config: ABTestConfig) -> Self {
        Self {
            config,
            metrics_calculator: MetricsCalculator::new(vec![1, 3, 5, 10, 20]),
        }
    }
    
    /// Run comprehensive A/B test evaluation
    pub async fn run_ab_test(&self) -> Result<ABTestReport> {
        println!("🚀 Starting comprehensive hybrid search A/B test evaluation...");
        
        // Load evaluation dataset
        let dataset = EvaluationDataset::from_json_file(&self.config.dataset_path)?;
        println!("📊 Loaded {} labeled examples for evaluation", dataset.examples.len());
        
        // Run baseline evaluation (vector-only)
        println!("🔍 Evaluating baseline system: {}", self.config.baseline_name);
        let baseline_report = self.evaluate_vector_only_system(&dataset).await?;
        
        // Run test evaluation (hybrid)
        println!("🔍 Evaluating test system: {}", self.config.test_name);
        let test_report = self.evaluate_hybrid_system(&dataset).await?;
        
        // Calculate per-query comparisons
        println!("📈 Calculating per-query comparisons...");
        let query_comparisons = self.calculate_query_comparisons(&baseline_report, &test_report)?;
        
        // Perform statistical significance tests
        println!("📊 Running statistical significance tests...");
        let significance_tests = self.calculate_significance_tests(&query_comparisons)?;
        
        // Generate performance summary
        let performance_summary = self.generate_performance_summary(&query_comparisons);
        
        // Generate recommendation
        let recommendation = self.generate_recommendation(&significance_tests, &performance_summary);
        
        let report = ABTestReport {
            config: self.config.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            baseline_report,
            test_report,
            query_comparisons,
            significance_tests,
            performance_summary,
            recommendation,
        };
        
        println!("✅ A/B test evaluation completed!");
        Ok(report)
    }
    
    /// Evaluate vector-only search system
    async fn evaluate_vector_only_system(&self, dataset: &EvaluationDataset) -> Result<EvaluationReport> {
        println!("🔍 Evaluating vector-only search system...");
        
        // For now, return a mock evaluation report with placeholder metrics
        // This TODO implements the basic structure for vector-only evaluation
        let mut query_metrics = Vec::new();
        let start_time = std::time::Instant::now();
        
        // Group examples by query to create query-level metrics
        let queries_by_text = self.group_examples_by_query(&dataset.examples);
        
        for (query_text, examples) in queries_by_text {
            let start_query_time = std::time::Instant::now();
            
            // TODO: Implement actual vector search execution
            // For now, simulate search with placeholder results
            let simulated_results = self.simulate_vector_search(&query_text, &examples);
            let query_latency_ms = start_query_time.elapsed().as_millis() as f64;
            
            // Calculate metrics for this query
            let metrics = self.calculate_query_metrics_from_examples(
                &query_text,
                &simulated_results,
                &examples,
            );
            
            query_metrics.push(metrics);
        }
        
        let total_latency_ms = start_time.elapsed().as_millis() as f64;
        
        // Calculate aggregated metrics
        let aggregated = self.calculate_aggregated_metrics(&query_metrics);
        
        Ok(EvaluationReport {
            dataset_name: dataset.name.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            system_name: "vector_only".to_string(),
            query_metrics,
            aggregated,
            metadata: HashMap::from([
                ("total_queries".to_string(), queries_by_text.len().to_string()),
                ("total_examples".to_string(), dataset.examples.len().to_string()),
                ("total_latency_ms".to_string(), total_latency_ms.to_string()),
                ("avg_latency_ms".to_string(), (total_latency_ms / queries_by_text.len() as f64).to_string()),
                ("evaluation_type".to_string(), "vector_only_baseline".to_string()),
            ]),
        })
    }
    
    /// Evaluate hybrid BM25 + vector search system
    async fn evaluate_hybrid_system(&self, dataset: &EvaluationDataset) -> Result<EvaluationReport> {
        use zero_latency_search::{
            hybrid::HybridSearchStep,
            vector_search::VectorSearchStep,
            bm25::BM25SearchStep,
            fusion::ScoreFusion,
            models::{SearchContext, SearchRequest},
            traits::SearchStep,
        };
        
        println!("🚀 Evaluating hybrid BM25 + vector search system...");
        
        // Create BM25 and vector search components
        let bm25_step = BM25SearchStep::new(self.config.bm25_k);
        let vector_step = VectorSearchStep::new(
            self.config.vector_k,
            self.config.similarity_threshold,
        );
        
        // Create score fusion component
        let score_fusion = ScoreFusion::new(
            self.config.fusion_weights.bm25_weight,
            self.config.fusion_weights.vector_weight,
        );
        
        // Create hybrid search step
        let hybrid_step = HybridSearchStep::new(
            bm25_step,
            vector_step,
            score_fusion,
        );
        
        let mut query_metrics = Vec::new();
        let start_time = std::time::Instant::now();
        
        for query_info in &dataset.queries {
            let start_query_time = std::time::Instant::now();
            
            // Create search request
            let request = SearchRequest {
                query: query_info.query.clone(),
                limit: self.config.k_values.iter().max().copied().unwrap_or(20),
                collections: vec![], // Evaluate on all collections
                filters: HashMap::new(),
            };
            
            // Create search context
            let mut context = SearchContext::new(request);
            
            // Execute hybrid search
            if let Err(e) = hybrid_step.execute(&mut context).await {
                tracing::warn!("Hybrid search failed for query '{}': {}", query_info.query, e);
                continue;
            }
            
            let query_latency_ms = start_query_time.elapsed().as_millis() as f64;
            
            // Calculate metrics for this query
            let metrics = self.calculate_query_metrics(
                &query_info.query,
                &context.search_results,
                &query_info.relevant_docs,
                query_latency_ms,
            )?;
            
            query_metrics.push(metrics);
        }
        
        let total_latency_ms = start_time.elapsed().as_millis() as f64;
        
        // Calculate aggregated metrics
        let aggregated = self.calculate_aggregated_metrics(&query_metrics);
        
        Ok(EvaluationReport {
            dataset_name: dataset.name.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            system_name: "hybrid_bm25_vector".to_string(),
            query_metrics,
            aggregated,
            metadata: HashMap::from([
                ("total_queries".to_string(), dataset.queries.len().to_string()),
                ("total_latency_ms".to_string(), total_latency_ms.to_string()),
                ("avg_latency_ms".to_string(), (total_latency_ms / dataset.queries.len() as f64).to_string()),
                ("bm25_k".to_string(), self.config.bm25_k.to_string()),
                ("vector_k".to_string(), self.config.vector_k.to_string()),
                ("similarity_threshold".to_string(), self.config.similarity_threshold.to_string()),
                ("bm25_weight".to_string(), self.config.fusion_weights.bm25_weight.to_string()),
                ("vector_weight".to_string(), self.config.fusion_weights.vector_weight.to_string()),
            ]),
        })
    }
    
    /// Calculate per-query performance comparisons
    fn calculate_query_comparisons(
        &self,
        baseline: &EvaluationReport,
        test: &EvaluationReport,
    ) -> Result<Vec<QueryComparison>> {
        let mut comparisons = Vec::new();
        
        for (baseline_query, test_query) in baseline.query_metrics.iter().zip(test.query_metrics.iter()) {
            if baseline_query.query != test_query.query {
                return Err(ZeroLatencyError::validation(
                    "query_mismatch", 
                    "Query mismatch between baseline and test results"
                ));
            }
            
            let mut metric_deltas = HashMap::new();
            
            // Calculate NDCG@10 delta
            let baseline_ndcg10 = baseline_query.ndcg_at_k.get(&10).unwrap_or(&0.0);
            let test_ndcg10 = test_query.ndcg_at_k.get(&10).unwrap_or(&0.0);
            metric_deltas.insert("ndcg_at_10".to_string(), test_ndcg10 - baseline_ndcg10);
            
            // Calculate other metric deltas
            metric_deltas.insert("mrr".to_string(), test_query.mrr - baseline_query.mrr);
            metric_deltas.insert("average_precision".to_string(), 
                test_query.average_precision - baseline_query.average_precision);
            
            let comparison = QueryComparison {
                query: baseline_query.query.clone(),
                category: None, // Categories can be extracted from dataset if available
                baseline_metrics: baseline_query.clone(),
                test_metrics: test_query.clone(),
                metric_deltas,
                performance_comparison: PerformanceComparison {
                    // TODO: Extract actual latency from evaluation reports metadata
                    baseline_latency_ms: baseline.metadata.get("avg_latency_ms")
                        .and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    test_latency_ms: test.metadata.get("avg_latency_ms")
                        .and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    baseline_tokens: None, // Token counting not implemented yet
                    test_tokens: None,
                    baseline_memory_mb: None, // Memory profiling not implemented yet
                    test_memory_mb: None,
                },
            };
            
            comparisons.push(comparison);
        }
        
        Ok(comparisons)
    }
    
    /// Calculate statistical significance tests for key metrics
    fn calculate_significance_tests(
        &self,
        comparisons: &[QueryComparison],
    ) -> Result<HashMap<String, SignificanceTest>> {
        let mut tests = HashMap::new();
        
        // Extract NDCG@10 deltas for significance testing
        let ndcg10_deltas: Vec<f64> = comparisons
            .iter()
            .filter_map(|comp| comp.metric_deltas.get("ndcg_at_10"))
            .copied()
            .collect();
        
        if !ndcg10_deltas.is_empty() {
            let ndcg10_test = self.randomization_test(&ndcg10_deltas)?;
            tests.insert("ndcg_at_10".to_string(), ndcg10_test);
        }
        
        // Test other metrics
        let mrr_deltas: Vec<f64> = comparisons
            .iter()
            .filter_map(|comp| comp.metric_deltas.get("mrr"))
            .copied()
            .collect();
            
        if !mrr_deltas.is_empty() {
            let mrr_test = self.randomization_test(&mrr_deltas)?;
            tests.insert("mrr".to_string(), mrr_test);
        }
        
        Ok(tests)
    }
    
    /// Perform randomization test for statistical significance
    fn randomization_test(&self, deltas: &[f64]) -> Result<SignificanceTest> {
        if deltas.is_empty() {
            return Err(ZeroLatencyError::validation(
                "deltas", 
                "Empty deltas for significance test"
            ));
        }
        
        let observed_mean = deltas.iter().sum::<f64>() / deltas.len() as f64;
        let mut rng = fastrand::Rng::with_seed(self.config.random_seed.unwrap_or(42));
        
        let mut permutation_means = Vec::with_capacity(self.config.bootstrap_samples);
        
        // Generate bootstrap samples
        for _ in 0..self.config.bootstrap_samples {
            let permuted_deltas: Vec<f64> = deltas.iter().map(|&d| {
                if rng.bool() { d } else { -d }
            }).collect();
            
            let permutation_mean = permuted_deltas.iter().sum::<f64>() / permuted_deltas.len() as f64;
            permutation_means.push(permutation_mean);
        }
        
        // Calculate p-value
        let extreme_count = permutation_means
            .iter()
            .filter(|&&mean| mean.abs() >= observed_mean.abs())
            .count();
        
        let p_value = extreme_count as f64 / self.config.bootstrap_samples as f64;
        let is_significant = p_value < (1.0 - self.config.confidence_level);
        
        // Calculate effect size (Cohen's d)
        let std_dev = {
            let mean = deltas.iter().sum::<f64>() / deltas.len() as f64;
            let variance = deltas.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / deltas.len() as f64;
            variance.sqrt()
        };
        
        let effect_size = if std_dev > 0.0 { observed_mean / std_dev } else { 0.0 };
        
        // Calculate confidence interval using bootstrap
        permutation_means.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let alpha = 1.0 - self.config.confidence_level;
        let lower_idx = ((alpha / 2.0) * permutation_means.len() as f64) as usize;
        let upper_idx = ((1.0 - alpha / 2.0) * permutation_means.len() as f64) as usize;
        
        let confidence_interval = (
            permutation_means.get(lower_idx).copied().unwrap_or(0.0),
            permutation_means.get(upper_idx.min(permutation_means.len() - 1)).copied().unwrap_or(0.0),
        );
        
        Ok(SignificanceTest {
            method: "randomization_test".to_string(),
            p_value,
            is_significant,
            confidence_level: self.config.confidence_level,
            effect_size,
            confidence_interval,
        })
    }
    
    /// Generate performance summary from query comparisons
    fn generate_performance_summary(&self, comparisons: &[QueryComparison]) -> PerformanceSummary {
        let total_queries = comparisons.len();
        
        let queries_improved_ndcg = comparisons
            .iter()
            .filter(|comp| comp.metric_deltas.get("ndcg_at_10").unwrap_or(&0.0) > &0.0)
            .count();
            
        let queries_degraded_ndcg = comparisons
            .iter()
            .filter(|comp| comp.metric_deltas.get("ndcg_at_10").unwrap_or(&0.0) < &0.0)
            .count();
        
        let mean_ndcg10_improvement = comparisons
            .iter()
            .filter_map(|comp| comp.metric_deltas.get("ndcg_at_10"))
            .sum::<f64>() / total_queries as f64;
        
        let mean_recall_improvement = comparisons
            .iter()
            .filter_map(|comp| comp.metric_deltas.get("recall_at_10"))
            .sum::<f64>() / total_queries as f64;
            
        let mean_latency_change_ms = comparisons
            .iter()
            .map(|comp| comp.performance_comparison.test_latency_ms - comp.performance_comparison.baseline_latency_ms)
            .sum::<f64>() / total_queries as f64;
        
        // Calculate P95 latency
        let mut latencies: Vec<f64> = comparisons
            .iter()
            .map(|comp| comp.performance_comparison.test_latency_ms)
            .collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_idx = ((latencies.len() as f64) * 0.95) as usize;
        let p95_latency_ms = latencies.get(p95_idx).copied().unwrap_or(0.0);
        
        PerformanceSummary {
            total_queries,
            queries_improved_ndcg,
            queries_degraded_ndcg,
            mean_ndcg10_improvement,
            mean_recall_improvement,
            mean_latency_change_ms,
            p95_latency_ms,
            success_rate: 1.0, // TODO: Calculate actual success rate
        }
    }
    
    /// Generate deployment recommendation based on test results
    fn generate_recommendation(
        &self,
        significance_tests: &HashMap<String, SignificanceTest>,
        performance_summary: &PerformanceSummary,
    ) -> TestRecommendation {
        let mut key_findings = Vec::new();
        let mut risks = Vec::new();
        let mut next_steps = Vec::new();
        
        // Analyze NDCG@10 results
        if let Some(ndcg_test) = significance_tests.get("ndcg_at_10") {
            let improvement_pct = performance_summary.mean_ndcg10_improvement * 100.0;
            
            if ndcg_test.is_significant && improvement_pct >= 15.0 {
                key_findings.push(format!(
                    "✅ NDCG@10 improved by {:.1}% with statistical significance (p={:.4})",
                    improvement_pct, ndcg_test.p_value
                ));
            } else if improvement_pct >= 15.0 {
                key_findings.push(format!(
                    "⚠️  NDCG@10 improved by {:.1}% but lacks statistical significance (p={:.4})",
                    improvement_pct, ndcg_test.p_value
                ));
                risks.push("Improvement may not be reliable due to lack of statistical significance".to_string());
            } else {
                key_findings.push(format!(
                    "❌ NDCG@10 improvement ({:.1}%) below target threshold (15%)",
                    improvement_pct
                ));
            }
        }
        
        // Analyze latency performance
        if performance_summary.p95_latency_ms > 350.0 {
            risks.push(format!(
                "P95 latency ({:.1}ms) exceeds target threshold (350ms)",
                performance_summary.p95_latency_ms
            ));
        } else {
            key_findings.push(format!(
                "✅ P95 latency ({:.1}ms) meets performance target",
                performance_summary.p95_latency_ms
            ));
        }
        
        // Generate deployment recommendation
        let deploy_recommendation = if performance_summary.mean_ndcg10_improvement >= 0.15
            && performance_summary.p95_latency_ms <= 350.0
            && significance_tests.get("ndcg_at_10").map_or(false, |t| t.is_significant)
        {
            next_steps.push("Proceed with gradual rollout to production".to_string());
            next_steps.push("Monitor search quality metrics closely".to_string());
            DeployRecommendation::StronglyRecommend
        } else if performance_summary.mean_ndcg10_improvement >= 0.10 {
            next_steps.push("Consider extended evaluation period".to_string());
            next_steps.push("Optimize performance before full deployment".to_string());
            DeployRecommendation::Conditional("Pending performance optimization".to_string())
        } else {
            next_steps.push("Re-evaluate hybrid search configuration".to_string());
            next_steps.push("Consider alternative fusion strategies".to_string());
            DeployRecommendation::NotRecommend("Insufficient quality improvement".to_string())
        };
        
        TestRecommendation {
            deploy_recommendation,
            key_findings,
            risks,
            next_steps,
        }
    }
    
    /// Calculate query metrics for a single query
    fn calculate_query_metrics(
        &self,
        query: &str,
        search_results: &[zero_latency_search::models::SearchResult],
        relevant_docs: &[zero_latency_search::evaluation::dataset::RelevanceRating],
        latency_ms: f64,
    ) -> Result<QueryMetrics> {
        use zero_latency_search::evaluation::metrics::MetricsCalculator;
        use zero_latency_search::evaluation::dataset::RelevanceRating;
        
        let calculator = MetricsCalculator::new(self.config.k_values.clone());
        
        // Create relevance judgment array by mapping search results to ground truth
        // This requires matching search results to ground truth documents
        let mut ground_truth_ranked = Vec::new();
        for result in search_results {
            // Find matching relevance rating for this document
            let rating = relevant_docs.iter()
                .find(|r| r.doc_id == result.doc_id)
                .map(|r| r.relevance)
                .unwrap_or(RelevanceRating::NotRelevant);
            ground_truth_ranked.push(rating);
        }
        
        // Calculate total relevant documents in the dataset
        let total_relevant = relevant_docs.iter()
            .filter(|r| r.relevance != RelevanceRating::NotRelevant)
            .count();
        
        // Calculate metrics using the existing metrics calculator
        let ndcg_at_k = self.config.k_values.iter()
            .map(|&k| (k, calculator.calculate_ndcg_at_k(search_results, relevant_docs, k)))
            .collect();
            
        let hit_at_k = self.config.k_values.iter()
            .map(|&k| (k, calculator.calculate_hit_at_k(&ground_truth_ranked, k)))
            .collect();
            
        let precision_at_k = self.config.k_values.iter()
            .map(|&k| (k, calculator.calculate_precision_at_k(&ground_truth_ranked, k)))
            .collect();
            
        let recall_at_k = self.config.k_values.iter()
            .map(|&k| (k, calculator.calculate_recall_at_k(&ground_truth_ranked, total_relevant, k)))
            .collect();
        
        let mrr = calculator.calculate_mrr(&ground_truth_ranked);
        let average_precision = calculator.calculate_average_precision(&ground_truth_ranked);
        
        Ok(QueryMetrics {
            query: query.to_string(),
            total_relevant,
            total_returned: search_results.len(),
            ndcg_at_k,
            hit_at_k,
            precision_at_k,
            recall_at_k,
            mrr,
            average_precision,
        })
    }
    
    /// Calculate aggregated metrics across all queries
    fn calculate_aggregated_metrics(&self, query_metrics: &[QueryMetrics]) -> zero_latency_search::evaluation::metrics::AggregatedMetrics {
        use zero_latency_search::evaluation::metrics::AggregatedMetrics;
        
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
        
        // Calculate mean NDCG@K for each K value
        let mean_ndcg_at_k = self.config.k_values.iter()
            .map(|&k| {
                let sum: f64 = query_metrics.iter()
                    .filter_map(|qm| qm.ndcg_at_k.get(&k))
                    .sum();
                (k, sum / num_queries as f64)
            })
            .collect();
            
        let mean_hit_at_k = self.config.k_values.iter()
            .map(|&k| {
                let sum: f64 = query_metrics.iter()
                    .filter_map(|qm| qm.hit_at_k.get(&k))
                    .sum();
                (k, sum / num_queries as f64)
            })
            .collect();
            
        let mean_precision_at_k = self.config.k_values.iter()
            .map(|&k| {
                let sum: f64 = query_metrics.iter()
                    .filter_map(|qm| qm.precision_at_k.get(&k))
                    .sum();
                (k, sum / num_queries as f64)
            })
            .collect();
            
        let mean_recall_at_k = self.config.k_values.iter()
            .map(|&k| {
                let sum: f64 = query_metrics.iter()
                    .filter_map(|qm| qm.recall_at_k.get(&k))
                    .sum();
                (k, sum / num_queries as f64)
            })
            .collect();
        
        let mean_mrr = query_metrics.iter().map(|qm| qm.mrr).sum::<f64>() / num_queries as f64;
        let mean_average_precision = query_metrics.iter().map(|qm| qm.average_precision).sum::<f64>() / num_queries as f64;
        
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

/// Convenience function to run hybrid search evaluation
pub async fn run_hybrid_search_evaluation(config: Option<ABTestConfig>) -> Result<ABTestReport> {
    let config = config.unwrap_or_default();
    let evaluator = HybridSearchEvaluator::new(config);
    evaluator.run_ab_test().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_evaluation_framework_initialization() {
        let config = ABTestConfig::default();
        let evaluator = HybridSearchEvaluator::new(config);
        
        // Test framework can be initialized
        assert_eq!(evaluator.config.bootstrap_samples, 10000);
        assert_eq!(evaluator.config.confidence_level, 0.95);
    }
    
    #[test]
    fn test_significance_test_calculation() {
        let config = ABTestConfig::default();
        let evaluator = HybridSearchEvaluator::new(config);
        
        // Test with synthetic data showing improvement
        let deltas = vec![0.1, 0.15, 0.05, 0.2, 0.12, 0.08, 0.18, 0.03, 0.14, 0.09];
        let result = evaluator.randomization_test(&deltas).unwrap();
        
        assert_eq!(result.method, "randomization_test");
        assert!(result.effect_size > 0.0);
        assert!(result.confidence_interval.0 <= result.confidence_interval.1);
    }
    
    #[test]
    fn test_performance_summary_generation() {
        let config = ABTestConfig::default();
        let evaluator = HybridSearchEvaluator::new(config);
        
        // Create mock query comparisons
        let comparisons = vec![
            QueryComparison {
                query: "test query 1".to_string(),
                category: None,
                baseline_metrics: QueryMetrics {
                    query: "test query 1".to_string(),
                    total_relevant: 5,
                    total_returned: 10,
                    ndcg_at_k: [(10, 0.7)].into_iter().collect(),
                    hit_at_k: HashMap::new(),
                    precision_at_k: HashMap::new(),
                    recall_at_k: HashMap::new(),
                    mrr: 0.8,
                    average_precision: 0.75,
                },
                test_metrics: QueryMetrics {
                    query: "test query 1".to_string(),
                    total_relevant: 5,
                    total_returned: 10,
                    ndcg_at_k: [(10, 0.85)].into_iter().collect(),
                    hit_at_k: HashMap::new(),
                    precision_at_k: HashMap::new(),
                    recall_at_k: HashMap::new(),
                    mrr: 0.9,
                    average_precision: 0.85,
                },
                metric_deltas: [("ndcg_at_10".to_string(), 0.15)].into_iter().collect(),
                performance_comparison: PerformanceComparison {
                    baseline_latency_ms: 200.0,
                    test_latency_ms: 250.0,
                    baseline_tokens: None,
                    test_tokens: None,
                    baseline_memory_mb: None,
                    test_memory_mb: None,
                },
            }
        ];
        
        let summary = evaluator.generate_performance_summary(&comparisons);
        
        assert_eq!(summary.total_queries, 1);
        assert_eq!(summary.queries_improved_ndcg, 1);
        assert_eq!(summary.queries_degraded_ndcg, 0);
        assert_eq!(summary.mean_ndcg10_improvement, 0.15);
    }
}
