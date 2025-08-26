use crate::application::interfaces::SearchService;
use crate::application::interfaces::VectorStorage;
use crate::infrastructure::memory::MemoryEfficientCache;
use serde::{Deserialize, Serialize};
/// Enhanced Search Service
///
/// Advanced search capabilities including confidence scoring, multi-modal filtering,
/// and batch operations for improved performance and user experience.
use std::collections::HashMap;
use std::sync::Arc;
use zero_latency_core::values::SearchQuery;
use zero_latency_search::models::{SearchFilters, SearchOptions, SearchRequest, SearchResult};

/// Enhanced search request with advanced filtering and scoring options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchRequest {
    /// Primary search query
    pub query: String,

    /// Maximum number of results to return
    pub limit: Option<usize>,

    /// Minimum confidence score threshold (0.0 - 1.0)
    pub min_confidence: Option<f32>,

    /// Target collection(s) to search
    pub collections: Option<Vec<String>>,

    /// Metadata filters for multi-modal search
    pub metadata_filters: Option<HashMap<String, MetadataFilter>>,

    /// Result ranking preferences
    pub ranking: Option<RankingConfig>,

    /// Enable detailed scoring information
    pub include_scores: bool,

    /// Enable result explanations
    pub include_explanations: bool,
}

/// Metadata filtering options for multi-modal search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataFilter {
    Equals(String),
    Contains(String),
    Range {
        min: Option<f64>,
        max: Option<f64>,
    },
    In(Vec<String>),
    DateRange {
        start: Option<String>,
        end: Option<String>,
    },
}

/// Result ranking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    /// Boost factor for semantic similarity (default: 1.0)
    pub semantic_weight: f32,

    /// Boost factor for metadata relevance (default: 0.5)
    pub metadata_weight: f32,

    /// Boost factor for recency (default: 0.3)
    pub recency_weight: f32,

    /// Custom scoring function
    pub custom_scoring: Option<String>,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 1.0,
            metadata_weight: 0.5,
            recency_weight: 0.3,
            custom_scoring: None,
        }
    }
}

/// Enhanced search result with confidence scoring and explanations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchResult {
    /// Document ID
    pub id: String,

    /// Document content or summary
    pub content: String,

    /// Metadata associated with the document
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Overall confidence score (0.0 - 1.0)
    pub confidence_score: f32,

    /// Detailed scoring breakdown
    pub scores: Option<DetailedScores>,

    /// Result explanation
    pub explanation: Option<String>,

    /// Collection the result came from
    pub collection: Option<String>,
}

/// Detailed scoring breakdown for result analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedScores {
    /// Semantic similarity score
    pub semantic_score: f32,

    /// Metadata relevance score  
    pub metadata_score: f32,

    /// Recency score
    pub recency_score: f32,

    /// Custom scoring component
    pub custom_score: Option<f32>,

    /// Final weighted score
    pub final_score: f32,
}

/// Batch search request for multiple queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchRequest {
    /// Multiple search requests to process
    pub requests: Vec<EnhancedSearchRequest>,

    /// Maximum total processing time (seconds)
    pub timeout: Option<u64>,

    /// Whether to fail fast on individual request errors
    pub fail_fast: bool,
}

/// Batch search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchResponse {
    /// Results for each search request
    pub results: Vec<BatchSearchResult>,

    /// Total processing time
    pub processing_time_ms: u64,

    /// Overall success rate
    pub success_rate: f32,
}

/// Individual result in a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchResult {
    /// Index of the original request
    pub request_index: usize,

    /// Search results if successful
    pub results: Option<Vec<EnhancedSearchResult>>,

    /// Error information if failed
    pub error: Option<String>,

    /// Processing time for this specific request
    pub processing_time_ms: u64,
}

/// Enhanced search service with advanced capabilities
pub struct EnhancedSearchService {
    /// Core search service
    search_service: Arc<dyn SearchService>,

    /// Vector stores by collection
    vector_stores: HashMap<String, Arc<dyn VectorStorage>>,

    /// Query result cache
    result_cache: MemoryEfficientCache<String, Vec<EnhancedSearchResult>>,

    /// Ranking engine
    ranking_engine: RankingEngine,

    /// Configuration
    config: EnhancedSearchConfig,
}

/// Configuration for enhanced search service
#[derive(Debug, Clone)]
pub struct EnhancedSearchConfig {
    /// Default confidence threshold
    pub default_min_confidence: f32,

    /// Maximum results per query
    pub max_results_per_query: usize,

    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,

    /// Enable query result caching
    pub cache_enabled: bool,

    /// Default ranking configuration
    pub default_ranking: RankingConfig,

    /// Batch processing timeout in seconds
    pub batch_timeout_seconds: u64,

    /// Maximum batch size
    pub max_batch_size: usize,
}

impl Default for EnhancedSearchConfig {
    fn default() -> Self {
        Self {
            default_min_confidence: 0.1,
            max_results_per_query: 100,
            cache_ttl_seconds: 300, // 5 minutes
            cache_enabled: std::env::var("ENHANCED_SEARCH_CACHE_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            default_ranking: RankingConfig::default(),
            batch_timeout_seconds: std::env::var("ENHANCED_SEARCH_BATCH_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            max_batch_size: std::env::var("ENHANCED_SEARCH_MAX_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
        }
    }
}

/// Ranking engine for result scoring and ordering
pub struct RankingEngine {
    config: RankingConfig,
}

impl RankingEngine {
    pub fn new(config: RankingConfig) -> Self {
        Self { config }
    }

    /// Calculate confidence score based on multiple factors
    pub fn calculate_confidence(
        &self,
        semantic_similarity: f32,
        metadata_relevance: f32,
        recency_score: f32,
        custom_score: Option<f32>,
    ) -> (f32, DetailedScores) {
        let semantic_component = semantic_similarity * self.config.semantic_weight;
        let metadata_component = metadata_relevance * self.config.metadata_weight;
        let recency_component = recency_score * self.config.recency_weight;
        let custom_component = custom_score.unwrap_or(0.0);

        let total_weight =
            self.config.semantic_weight + self.config.metadata_weight + self.config.recency_weight;

        let final_score =
            (semantic_component + metadata_component + recency_component + custom_component)
                / total_weight;

        let detailed_scores = DetailedScores {
            semantic_score: semantic_similarity,
            metadata_score: metadata_relevance,
            recency_score,
            custom_score,
            final_score,
        };

        (final_score.clamp(0.0, 1.0), detailed_scores)
    }

    /// Generate explanation for the scoring
    pub fn generate_explanation(&self, scores: &DetailedScores, query: &str) -> String {
        format!(
            "Result scored {:.3} for query '{}'. Breakdown: semantic similarity {:.3}, metadata relevance {:.3}, recency {:.3}",
            scores.final_score,
            query,
            scores.semantic_score,
            scores.metadata_score,
            scores.recency_score
        )
    }
}

impl EnhancedSearchService {
    pub fn new(
        search_service: Arc<dyn SearchService>,
        vector_stores: HashMap<String, Arc<dyn VectorStorage>>,
        config: EnhancedSearchConfig,
    ) -> Self {
        use crate::infrastructure::memory::cache::CacheConfig;
        let cache_config = CacheConfig {
            max_entries: 1000,
            ..Default::default()
        };
        let result_cache = MemoryEfficientCache::new(cache_config);

        let ranking_engine = RankingEngine::new(config.default_ranking.clone());

        Self {
            search_service,
            vector_stores,
            result_cache,
            ranking_engine,
            config,
        }
    }

    /// Enhanced search with confidence scoring and advanced filtering
    pub async fn enhanced_search(
        &self,
        request: EnhancedSearchRequest,
    ) -> Result<Vec<EnhancedSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = self.generate_cache_key(&request);

        // Check cache first if enabled
        if self.config.cache_enabled {
            if let Some(cached_results) = self.result_cache.get(&cache_key) {
                return Ok(cached_results.clone());
            }
        }

        // Convert to core search request
        let core_request = self.convert_to_core_request(&request)?;

        // Perform core search
        let core_results = self.search_service.search(core_request).await?;

        // Enhance results with scoring and filtering
        let mut enhanced_results = Vec::new();

        for core_result in core_results.results.iter() {
            if let Some(enhanced_result) = self.enhance_result(core_result, &request).await? {
                // Apply confidence threshold
                if let Some(min_confidence) = request.min_confidence {
                    if enhanced_result.confidence_score < min_confidence {
                        continue;
                    }
                }
                enhanced_results.push(enhanced_result);
            }
        }

        // Apply custom ranking if specified
        if let Some(ranking_config) = &request.ranking {
            self.apply_custom_ranking(&mut enhanced_results, ranking_config);
        }

        // Apply limit
        let limit = request.limit.unwrap_or(self.config.max_results_per_query);
        enhanced_results.truncate(limit);

        // Cache results if enabled
        if self.config.cache_enabled {
            self.result_cache
                .insert(cache_key, enhanced_results.clone());
        }

        Ok(enhanced_results)
    }

    /// Batch search for multiple queries
    pub async fn batch_search(
        &self,
        request: BatchSearchRequest,
    ) -> Result<BatchSearchResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        // Validate batch size
        if request.requests.len() > self.config.max_batch_size {
            return Err(format!(
                "Batch size {} exceeds maximum {}",
                request.requests.len(),
                self.config.max_batch_size
            )
            .into());
        }

        let mut batch_results = Vec::new();
        let mut successful_requests = 0;

        // Process requests concurrently
        let tasks: Vec<_> = request
            .requests
            .into_iter()
            .enumerate()
            .map(|(index, req)| {
                let service = self;
                async move {
                    let request_start = std::time::Instant::now();

                    match service.enhanced_search(req).await {
                        Ok(results) => BatchSearchResult {
                            request_index: index,
                            results: Some(results),
                            error: None,
                            processing_time_ms: request_start.elapsed().as_millis() as u64,
                        },
                        Err(error) => BatchSearchResult {
                            request_index: index,
                            results: None,
                            error: Some(error.to_string()),
                            processing_time_ms: request_start.elapsed().as_millis() as u64,
                        },
                    }
                }
            })
            .collect();

        // Execute with timeout
        let timeout = std::time::Duration::from_secs(
            request.timeout.unwrap_or(self.config.batch_timeout_seconds),
        );

        let results = tokio::time::timeout(timeout, futures::future::join_all(tasks)).await?;

        for result in results {
            if result.error.is_none() {
                successful_requests += 1;
            }
            batch_results.push(result);
        }

        let total_requests = batch_results.len();
        let success_rate = if total_requests > 0 {
            successful_requests as f32 / total_requests as f32
        } else {
            0.0
        };

        Ok(BatchSearchResponse {
            results: batch_results,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            success_rate,
        })
    }

    /// Cross-collection search
    pub async fn cross_collection_search(
        &self,
        request: EnhancedSearchRequest,
    ) -> Result<Vec<EnhancedSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let collections = if let Some(collections) = &request.collections {
            collections.clone()
        } else {
            self.vector_stores.keys().cloned().collect()
        };

        let mut all_results = Vec::new();

        // Search each collection
        for collection in collections {
            let mut collection_request = request.clone();
            collection_request.collections = Some(vec![collection.clone()]);

            match self.enhanced_search(collection_request).await {
                Ok(mut results) => {
                    // Tag results with collection
                    for result in &mut results {
                        result.collection = Some(collection.clone());
                    }
                    all_results.extend(results);
                }
                Err(error) => {
                    eprintln!("Error searching collection {}: {}", collection, error);
                }
            }
        }

        // Re-rank combined results
        if let Some(ranking_config) = &request.ranking {
            self.apply_custom_ranking(&mut all_results, ranking_config);
        } else {
            // Sort by confidence score
            all_results
                .sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        }

        // Apply limit
        let limit = request.limit.unwrap_or(self.config.max_results_per_query);
        all_results.truncate(limit);

        Ok(all_results)
    }

    // Helper methods

    fn generate_cache_key(&self, request: &EnhancedSearchRequest) -> String {
        // Generate a cache key based on request parameters
        format!(
            "{}:{}:{:?}:{:?}",
            request.query,
            request.limit.unwrap_or(10),
            request.min_confidence,
            request.collections
        )
    }

    fn convert_to_core_request(
        &self,
        request: &EnhancedSearchRequest,
    ) -> Result<SearchRequest, Box<dyn std::error::Error + Send + Sync>> {
        // Convert enhanced request to core search request
        Ok(SearchRequest {
            query: SearchQuery::new(request.query.clone()),
            limit: request.limit.unwrap_or(20),
            offset: 0,
            filters: SearchFilters::default(),
            options: SearchOptions::default(),
        })
    }

    async fn enhance_result(
        &self,
        core_result: &SearchResult,
        request: &EnhancedSearchRequest,
    ) -> Result<Option<EnhancedSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        // Apply metadata filtering if specified
        if let Some(metadata_filters) = &request.metadata_filters {
            if !self.apply_metadata_filters(core_result, metadata_filters) {
                return Ok(None);
            }
        }
        // Calculate confidence score
        let semantic_similarity = core_result.final_score.value();
        let metadata_relevance = self.calculate_metadata_relevance(core_result, &request.query);
        let recency_score = self.calculate_recency_score(core_result);
        let (confidence_score, detailed_scores) = self.ranking_engine.calculate_confidence(
            semantic_similarity,
            metadata_relevance,
            recency_score,
            None,
        );
        let explanation = if request.include_explanations {
            Some(
                self.ranking_engine
                    .generate_explanation(&detailed_scores, &request.query),
            )
        } else {
            None
        };
        let scores = if request.include_scores {
            Some(detailed_scores)
        } else {
            None
        };
        Ok(Some(EnhancedSearchResult {
            id: core_result.chunk_id.to_string(),
            content: core_result.content.clone(),
            metadata: None, // TODO: Map fields as needed
            confidence_score,
            scores,
            explanation,
            collection: None, // Will be set by calling function if needed
        }))
    }

    fn apply_metadata_filters(
        &self,
        _result: &SearchResult,
        _filters: &HashMap<String, MetadataFilter>,
    ) -> bool {
        // No metadata available in SearchResult; always return true or implement custom logic if needed
        true
    }

    fn apply_single_metadata_filter(
        &self,
        metadata: &HashMap<String, serde_json::Value>,
        key: &str,
        filter: &MetadataFilter,
    ) -> bool {
        let value = match metadata.get(key) {
            Some(value) => value,
            None => return false,
        };

        match filter {
            MetadataFilter::Equals(expected) => value.as_str().map_or(false, |v| v == expected),
            MetadataFilter::Contains(substring) => {
                value.as_str().map_or(false, |v| v.contains(substring))
            }
            MetadataFilter::Range { min, max } => {
                let num_value = value.as_f64().unwrap_or(0.0);
                let min_ok = min.map_or(true, |min| num_value >= min);
                let max_ok = max.map_or(true, |max| num_value <= max);
                min_ok && max_ok
            }
            MetadataFilter::In(options) => value
                .as_str()
                .map_or(false, |v| options.contains(&v.to_string())),
            MetadataFilter::DateRange { start: _, end: _ } => {
                // TODO: Implement date range filtering
                true
            }
        }
    }

    fn calculate_metadata_relevance(&self, result: &SearchResult, query: &str) -> f32 {
        // Use document_title and content for a simple relevance heuristic
        let query_lower = query.to_lowercase();
        if result.document_title.to_lowercase().contains(&query_lower)
            || result.content.to_lowercase().contains(&query_lower)
        {
            0.8
        } else {
            0.2
        }
    }

    fn calculate_recency_score(&self, _result: &SearchResult) -> f32 {
        // No recency info in SearchResult; return default
        0.5
    }

    fn apply_custom_ranking(
        &self,
        results: &mut Vec<EnhancedSearchResult>,
        ranking_config: &RankingConfig,
    ) {
        // Apply custom ranking logic
        let ranking_engine = RankingEngine::new(ranking_config.clone());

        for result in results.iter_mut() {
            if let Some(scores) = &result.scores {
                let (new_confidence, new_scores) = ranking_engine.calculate_confidence(
                    scores.semantic_score,
                    scores.metadata_score,
                    scores.recency_score,
                    scores.custom_score,
                );
                result.confidence_score = new_confidence;
                result.scores = Some(new_scores);
            }
        }

        // Sort by updated confidence scores
        results.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranking_engine_calculation() {
        let config = RankingConfig::default();
        let engine = RankingEngine::new(config);

        let (confidence, scores) = engine.calculate_confidence(0.8, 0.6, 0.4, None);

        assert!(confidence > 0.0 && confidence <= 1.0);
        assert_eq!(scores.semantic_score, 0.8);
        assert_eq!(scores.metadata_score, 0.6);
        assert_eq!(scores.recency_score, 0.4);
    }

    #[test]
    fn test_enhanced_search_config_defaults() {
        let config = EnhancedSearchConfig::default();

        assert_eq!(config.default_min_confidence, 0.1);
        assert_eq!(config.max_results_per_query, 100);
        assert!(config.cache_enabled); // Default should be true
    }

    // #[test]
    // fn test_metadata_filter_equals() {
    //     let service = EnhancedSearchService::new(
    //         Arc::new(crate::test_utils::MockSearchService::new()),
    //         HashMap::new(),
    //         EnhancedSearchConfig::default(),
    //     );
    //     let mut metadata = HashMap::new();
    //     metadata.insert("type".to_string(), serde_json::Value::String("document".to_string()));
    //     let filter = MetadataFilter::Equals("document".to_string());
    //     assert!(service.apply_single_metadata_filter(&metadata, "type", &filter));
    //     let filter = MetadataFilter::Equals("image".to_string());
    //     assert!(!service.apply_single_metadata_filter(&metadata, "type", &filter));
    // }
    //
    // #[test]
    // fn test_metadata_filter_range() {
    //     let service = EnhancedSearchService::new(
    //         Arc::new(crate::test_utils::MockSearchService::new()),
    //         HashMap::new(),
    //         EnhancedSearchConfig::default(),
    //     );
    //     let mut metadata = HashMap::new();
    //     metadata.insert("size".to_string(), serde_json::Value::Number(serde_json::Number::from(50)));
    //     let filter = MetadataFilter::Range { min: Some(10.0), max: Some(100.0) };
    //     assert!(service.apply_single_metadata_filter(&metadata, "size", &filter));
    //     let filter = MetadataFilter::Range { min: Some(60.0), max: Some(100.0) };
    //     assert!(!service.apply_single_metadata_filter(&metadata, "size", &filter));
    // }
}
