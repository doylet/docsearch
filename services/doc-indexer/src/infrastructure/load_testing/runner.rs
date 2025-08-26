/// Load Test Runner
/// 
/// Orchestrates load testing scenarios with comprehensive metrics collection,
/// memory optimization validation, and production deployment readiness assessment.

use crate::infrastructure::load_testing::{
    LoadTestConfig, scenario::{LoadTestScenario, ScenarioRequest, ScenarioResponse},
    metrics::{LoadTestMetrics, MetricsCollector}
};
use crate::application::interfaces::{EmbeddingService, SearchService};
use super::scenario::EmbeddingInput;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::pin::Pin;
use std::future::Future;
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// Results from a complete load test execution
#[derive(Debug)]
pub struct LoadTestResult {
    pub metrics: LoadTestMetrics,
    pub success: bool,
    pub summary: String,
    pub recommendations: Vec<String>,
}

/// Main load test orchestrator
pub struct LoadTestRunner {
    config: LoadTestConfig,
    scenarios: Vec<Box<dyn LoadTestScenario>>,
    embedding_service: Arc<dyn EmbeddingService>,
    search_service: Arc<dyn SearchService>,
}

impl LoadTestRunner {
    pub fn new(
        config: LoadTestConfig,
        scenarios: Vec<Box<dyn LoadTestScenario>>,
        embedding_service: Arc<dyn EmbeddingService>,
        search_service: Arc<dyn SearchService>,
    ) -> Self {
        Self {
            config,
            scenarios,
            embedding_service,
            search_service,
        }
    }
    
    /// Execute the complete load test suite
    pub async fn run(&self, test_name: String) -> Result<LoadTestResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting load test: {}", test_name);
        println!("Configuration:");
        println!("  Concurrency: {}", self.config.concurrency);
        println!("  Duration: {:?}", self.config.duration);
        println!("  Scenarios: {}", self.scenarios.len());
        
        let mut metrics_collector = MetricsCollector::new();
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));
        let start_time = Instant::now();
        
        // Start memory monitoring task
        let memory_monitor = self.start_memory_monitoring(metrics_collector.clone()).await;
        
        // Run load test scenarios
        let scenario_results = self.run_scenarios(
            &mut metrics_collector,
            semaphore.clone(),
            start_time,
        ).await?;
        
        // Stop memory monitoring
        memory_monitor.abort();
        
        // Finalize metrics
        let metrics = metrics_collector.finalize(test_name.clone());
        
        // Analyze results
        let analysis = self.analyze_results(&metrics);
        
        Ok(LoadTestResult {
            success: analysis.success,
            summary: analysis.summary,
            recommendations: analysis.recommendations,
            metrics,
        })
    }
    
    async fn start_memory_monitoring(
        &self,
        mut metrics_collector: MetricsCollector,
    ) -> tokio::task::JoinHandle<()> {
        let sample_interval = Duration::from_millis(500);
        
        tokio::spawn(async move {
            loop {
                metrics_collector.sample_memory();
                sleep(sample_interval).await;
            }
        })
    }
    
    async fn run_scenarios(
        &self,
        metrics_collector: &mut MetricsCollector,
        semaphore: Arc<Semaphore>,
        start_time: Instant,
    ) -> Result<Vec<ScenarioExecutionResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut handles = Vec::new();
        let mut request_count = 0u64;
        
        // Generate requests according to scenario weights
        let total_weight: f32 = self.scenarios.iter().map(|s| s.config().weight).sum();
        
        while start_time.elapsed() < self.config.duration {
            for scenario in &self.scenarios {
                let weight_ratio = scenario.config().weight / total_weight;
                let scenario_requests = (self.config.concurrency as f32 * weight_ratio) as usize;
                
                for _ in 0..scenario_requests {
                    if start_time.elapsed() >= self.config.duration {
                        break;
                    }
                    
                    let permit = semaphore.clone().acquire_owned().await?;
                    let request = scenario.generate_request();
                    let scenario_name = scenario.name().to_string();
                    let embedding_service = self.embedding_service.clone();
                    let search_service = self.search_service.clone();
                    let scenario_timeout = scenario.config().timeout;
                    
                    let handle = tokio::spawn(async move {
                        let _permit = permit; // Keep permit until task completes
                        
                        let execution_start = Instant::now();
                        let result = Self::execute_request(
                            request,
                            embedding_service,
                            search_service,
                            scenario_timeout,
                        ).await;
                        let execution_time = execution_start.elapsed();
                        
                        ScenarioExecutionResult {
                            scenario_name,
                            execution_time,
                            result,
                        }
                    });
                    
                    handles.push(handle);
                    request_count += 1;
                    
                    // Apply rate limiting if configured
                    if let Some(rate_limit) = self.config.rate_limit {
                        let delay = Duration::from_secs_f64(1.0 / rate_limit as f64);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        println!("⏳ Waiting for {} requests to complete...", handles.len());
        
        // Collect all results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    // Update metrics collector
                    match &result.result {
                        Ok(_) => metrics_collector.record_success(&result.scenario_name, result.execution_time),
                        Err(error) => metrics_collector.record_error(&result.scenario_name, &error.to_string()),
                    }
                    results.push(result);
                },
                Err(join_error) => {
                    eprintln!("Task join error: {}", join_error);
                    metrics_collector.record_error("unknown", "task_join_error");
                }
            }
        }
        
        Ok(results)
    }
    
    fn execute_request(
        request: ScenarioRequest,
        embedding_service: Arc<dyn EmbeddingService>,
        search_service: Arc<dyn SearchService>,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<ScenarioResponse, Box<dyn std::error::Error + Send + Sync>>> + Send>> {
        Box::pin(async move {
            match request {
                ScenarioRequest::Embedding(input) => {
                    let embedding = embedding_service.generate_embeddings(&input.text).await?;
                    Ok(ScenarioResponse::Embedding(embedding))
                },
                ScenarioRequest::Search(search_req) => {
                        let results = search_service.search(search_req).await?;
                    Ok(ScenarioResponse::Search(serde_json::to_value(results)?))
                },
                ScenarioRequest::Batch(requests) => {
                    let mut responses = Vec::new();
                    for req in requests {
                        let response = Self::execute_request(
                            req,
                            embedding_service.clone(),
                            search_service.clone(),
                            timeout,
                        ).await?;
                        responses.push(response);
                    }
                    Ok(ScenarioResponse::Batch(responses))
                },
                ScenarioRequest::Mixed(_) => {
                    // For now, treat mixed as embedding request
                    let input = EmbeddingInput {
                        text: "mixed workload test".to_string(),
                        metadata: None,
                    };
                    let embedding = embedding_service.generate_embeddings(&input.text).await?;
                    Ok(ScenarioResponse::Embedding(embedding))
                },
            }
    })
    }
    
    fn analyze_results(&self, metrics: &LoadTestMetrics) -> LoadTestAnalysis {
        let mut success = true;
        let mut recommendations = Vec::new();
        
        // Performance analysis
        if metrics.performance.response_times.mean > Duration::from_millis(1000) {
            success = false;
            recommendations.push("Response times exceed 1s threshold - consider performance optimization".to_string());
        }
        
        if metrics.performance.throughput.requests_per_second < 10.0 {
            success = false;
            recommendations.push("Throughput below 10 req/s - investigate bottlenecks".to_string());
        }
        
        // Memory analysis
        if metrics.memory.leak_detection.suspected_leak {
            success = false;
            recommendations.push("Memory leak detected - investigate memory management".to_string());
        }
        
        if metrics.memory.process_memory.peak_mb > 1000.0 {
            recommendations.push("High memory usage detected - consider memory optimization".to_string());
        }
        
        // Error analysis
        if metrics.errors.error_rate > 0.01 { // More than 1% error rate
            success = false;
            recommendations.push(format!(
                "Error rate {:.2}% exceeds 1% threshold - investigate error causes",
                metrics.errors.error_rate * 100.0
            ));
        }
        
        // Memory optimization validation
        if self.config.validate_optimizations {
            recommendations.extend(self.validate_memory_optimizations(metrics));
        }
        
        // Success recommendations
        if success {
            recommendations.push("✅ Load test passed all criteria".to_string());
            recommendations.push("✅ System ready for production deployment".to_string());
        }
        
        let summary = crate::infrastructure::load_testing::metrics::generate_summary_report(metrics);
        
        LoadTestAnalysis {
            success,
            summary,
            recommendations,
        }
    }
    
    fn validate_memory_optimizations(&self, metrics: &LoadTestMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Validate memory stability
        if metrics.memory.leak_detection.stability_score > 0.8 {
            recommendations.push("✅ Memory usage stable - optimizations effective".to_string());
        } else {
            recommendations.push("⚠️ Memory usage unstable - review optimization settings".to_string());
        }
        
        // Check memory growth rate
        let growth_rate = metrics.memory.leak_detection.growth_rate_mb_per_sec;
        if growth_rate < 0.1 {
            recommendations.push("✅ Memory growth rate acceptable".to_string());
        } else {
            recommendations.push(format!(
                "⚠️ Memory growth rate {:.2} MB/s may indicate optimization issues",
                growth_rate
            ));
        }
        
        // Validate memory efficiency for scenarios
        for (scenario_name, scenario_metrics) in &metrics.scenarios {
            if scenario_metrics.memory_efficiency > 0.8 {
                recommendations.push(format!("✅ {} scenario memory efficient", scenario_name));
            } else {
                recommendations.push(format!(
                    "⚠️ {} scenario memory efficiency {:.2} - review optimization",
                    scenario_name, scenario_metrics.memory_efficiency
                ));
            }
        }
        
        recommendations
    }
}

/// Internal result structure for scenario execution
#[derive(Debug)]
struct ScenarioExecutionResult {
    scenario_name: String,
    execution_time: Duration,
    result: Result<ScenarioResponse, Box<dyn std::error::Error + Send + Sync>>,
}

/// Analysis results from load test execution
struct LoadTestAnalysis {
    success: bool,
    summary: String,
    recommendations: Vec<String>,
}

/// Create a default load test runner with standard scenarios
pub async fn create_default_runner(
    embedding_service: Arc<dyn EmbeddingService>,
    search_service: Arc<dyn SearchService>,
) -> LoadTestRunner {
    let config = LoadTestConfig::default();
    let scenarios = crate::infrastructure::load_testing::scenario::create_scenarios();
    
    LoadTestRunner::new(config, scenarios, embedding_service, search_service)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed test_utils mocks: MockEmbeddingService, MockSearchService
    use std::sync::Arc;
    

}
