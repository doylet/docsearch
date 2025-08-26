/// Load Testing Scenarios
/// 
/// Defines realistic production workload scenarios for comprehensive
/// performance testing and memory optimization validation.

use std::collections::HashMap;
use std::time::Duration;
use zero_latency_search::{SearchRequest, SearchFilters, SearchOptions};
use zero_latency_core::values::SearchQuery;
use serde_json::Value;

/// Embedding input for load testing scenarios
#[derive(Debug, Clone)]
pub struct EmbeddingInput {
    pub text: String,
    pub metadata: Option<Value>,
}

/// Configuration for a load testing scenario
#[derive(Debug, Clone)]
pub struct ScenarioConfig {
    pub name: String,
    pub description: String,
    pub weight: f32,  // Relative frequency in mixed workloads
    pub timeout: Duration,
}

/// Individual load testing scenario
pub trait LoadTestScenario: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn config(&self) -> &ScenarioConfig;
    
    /// Generate a request for this scenario
    fn generate_request(&self) -> ScenarioRequest;
    
    /// Validate the response for correctness
    fn validate_response(&self, response: &ScenarioResponse) -> Result<(), String>;
    
    /// Get expected memory usage pattern
    fn expected_memory_pattern(&self) -> MemoryPattern;
}

/// Request types for load testing scenarios
#[derive(Debug, Clone)]
pub enum ScenarioRequest {
    Embedding(EmbeddingInput),
    Search(SearchRequest),
    Batch(Vec<ScenarioRequest>),
    Mixed(HashMap<String, Value>),
}

/// Response types from load testing scenarios
#[derive(Debug, Clone)]
pub enum ScenarioResponse {
    Embedding(Vec<f32>),
    Search(Value),
    Batch(Vec<ScenarioResponse>),
    Mixed(HashMap<String, Value>),
    Error(String),
}

/// Expected memory usage pattern for validation
#[derive(Debug, Clone)]
pub struct MemoryPattern {
    pub baseline_mb: f64,
    pub peak_mb: f64,
    pub growth_rate: f64,  // MB per operation
    pub should_stabilize: bool,
}

/// High-frequency embedding generation scenario
pub struct EmbeddingIntensiveScenario {
    config: ScenarioConfig,
    document_pool: Vec<String>,
}

impl EmbeddingIntensiveScenario {
    pub fn new() -> Self {
        Self {
            config: ScenarioConfig {
                name: "embedding_intensive".to_string(),
                description: "High-frequency embedding generation with document corpus".to_string(),
                weight: 0.4,
                timeout: Duration::from_secs(5),
            },
            document_pool: vec![
                "Machine learning algorithms enable computers to learn patterns from data without explicit programming.".to_string(),
                "Rust's ownership system prevents memory leaks and data races at compile time.".to_string(),
                "Vector databases optimize storage and retrieval of high-dimensional embeddings.".to_string(),
                "Semantic search uses meaning rather than keywords to find relevant documents.".to_string(),
                "Zero-latency systems require careful optimization of memory allocation and processing pipelines.".to_string(),
                "Production deployments must handle concurrent requests while maintaining response quality.".to_string(),
                "Load testing validates system performance under realistic usage scenarios.".to_string(),
                "Performance monitoring enables early detection of optimization regressions.".to_string(),
            ],
        }
    }
}

impl LoadTestScenario for EmbeddingIntensiveScenario {
    fn name(&self) -> &str { &self.config.name }
    fn description(&self) -> &str { &self.config.description }
    fn config(&self) -> &ScenarioConfig { &self.config }
    
    fn generate_request(&self) -> ScenarioRequest {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let doc = &self.document_pool[rng.gen_range(0..self.document_pool.len())];
        
        ScenarioRequest::Embedding(EmbeddingInput {
            text: doc.clone(),
            metadata: Some(serde_json::json!({
                "scenario": "embedding_intensive",
                "timestamp": chrono::Utc::now().timestamp()
            })),
        })
    }
    
    fn validate_response(&self, response: &ScenarioResponse) -> Result<(), String> {
        match response {
            ScenarioResponse::Embedding(embedding) => {
                if embedding.is_empty() {
                    Err("Empty embedding vector".to_string())
                } else if embedding.len() != 384 {  // Expected embedding dimension
                    Err(format!("Unexpected embedding dimension: {}", embedding.len()))
                } else {
                    Ok(())
                }
            },
            ScenarioResponse::Error(err) => Err(format!("Embedding error: {}", err)),
            _ => Err("Unexpected response type for embedding scenario".to_string()),
        }
    }
    
    fn expected_memory_pattern(&self) -> MemoryPattern {
        MemoryPattern {
            baseline_mb: 50.0,
            peak_mb: 200.0,
            growth_rate: 0.1,  // Should be minimal with vector pooling
            should_stabilize: true,
        }
    }
}

/// Search-heavy scenario with varied query patterns
pub struct SearchIntensiveScenario {
    config: ScenarioConfig,
    query_pool: Vec<String>,
}

impl SearchIntensiveScenario {
    pub fn new() -> Self {
        Self {
            config: ScenarioConfig {
                name: "search_intensive".to_string(),
                description: "High-frequency search requests with diverse query patterns".to_string(),
                weight: 0.3,
                timeout: Duration::from_secs(3),
            },
            query_pool: vec![
                "machine learning algorithms".to_string(),
                "rust memory safety".to_string(),
                "vector database optimization".to_string(),
                "semantic search implementation".to_string(),
                "production performance monitoring".to_string(),
            ],
        }
    }
}

impl LoadTestScenario for SearchIntensiveScenario {
    fn name(&self) -> &str { &self.config.name }
    fn description(&self) -> &str { &self.config.description }
    fn config(&self) -> &ScenarioConfig { &self.config }
    
    fn generate_request(&self) -> ScenarioRequest {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let query = &self.query_pool[rng.gen_range(0..self.query_pool.len())];
        
        ScenarioRequest::Search(SearchRequest {
            query: SearchQuery::new(query.clone()),
            limit: rng.gen_range(1..=10),
            offset: 0,
            filters: SearchFilters::default(),
            options: SearchOptions::default(),
        })
    }
    
    fn validate_response(&self, response: &ScenarioResponse) -> Result<(), String> {
        match response {
            ScenarioResponse::Search(results) => {
                if results.is_null() {
                    Err("Null search results".to_string())
                } else {
                    Ok(())
                }
            },
            ScenarioResponse::Error(err) => Err(format!("Search error: {}", err)),
            _ => Err("Unexpected response type for search scenario".to_string()),
        }
    }
    
    fn expected_memory_pattern(&self) -> MemoryPattern {
        MemoryPattern {
            baseline_mb: 30.0,
            peak_mb: 150.0,
            growth_rate: 0.05,  // Should be minimal with smart caching
            should_stabilize: true,
        }
    }
}

/// Mixed workload scenario simulating realistic production usage
pub struct MixedWorkloadScenario {
    config: ScenarioConfig,
    embedding_scenario: EmbeddingIntensiveScenario,
    search_scenario: SearchIntensiveScenario,
}

impl MixedWorkloadScenario {
    pub fn new() -> Self {
        Self {
            config: ScenarioConfig {
                name: "mixed_workload".to_string(),
                description: "Realistic production workload with mixed embedding and search operations".to_string(),
                weight: 0.3,
                timeout: Duration::from_secs(10),
            },
            embedding_scenario: EmbeddingIntensiveScenario::new(),
            search_scenario: SearchIntensiveScenario::new(),
        }
    }
}

impl LoadTestScenario for MixedWorkloadScenario {
    fn name(&self) -> &str { &self.config.name }
    fn description(&self) -> &str { &self.config.description }
    fn config(&self) -> &ScenarioConfig { &self.config }
    
    fn generate_request(&self) -> ScenarioRequest {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // 70% embedding, 30% search for realistic workload
        if rng.gen::<f32>() < 0.7 {
            self.embedding_scenario.generate_request()
        } else {
            self.search_scenario.generate_request()
        }
    }
    
    fn validate_response(&self, response: &ScenarioResponse) -> Result<(), String> {
        match response {
            ScenarioResponse::Embedding(_) => self.embedding_scenario.validate_response(response),
            ScenarioResponse::Search(_) => self.search_scenario.validate_response(response),
            ScenarioResponse::Error(err) => Err(format!("Mixed workload error: {}", err)),
            _ => Err("Unexpected response type for mixed workload scenario".to_string()),
        }
    }
    
    fn expected_memory_pattern(&self) -> MemoryPattern {
        MemoryPattern {
            baseline_mb: 60.0,
            peak_mb: 250.0,
            growth_rate: 0.08,  // Combination of both scenarios
            should_stabilize: true,
        }
    }
}

/// Factory for creating scenario instances
pub fn create_scenarios() -> Vec<Box<dyn LoadTestScenario>> {
    vec![
        Box::new(EmbeddingIntensiveScenario::new()),
        Box::new(SearchIntensiveScenario::new()),
        Box::new(MixedWorkloadScenario::new()),
    ]
}
