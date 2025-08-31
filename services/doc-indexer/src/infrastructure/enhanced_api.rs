use crate::infrastructure::{
    BatchOperationRequest, BatchOperationResult, BatchOperationType, BatchProcessor,
    CollectionConfig, CollectionManager, CreateCollectionRequest, CrossCollectionSearchRequest,
    EnhancedSearchRequest, EnhancedSearchResult, EnhancedSearchService,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Enhanced API Integration
///
/// High-level API integration for Phase 4D enhanced features including
/// collection management, batch operations, and advanced search capabilities.
use std::sync::Arc;

/// Enhanced API service that provides high-level access to Phase 4D features
pub struct EnhancedApiService {
    /// Enhanced search service
    search_service: Arc<EnhancedSearchService>,

    /// Collection manager
    collection_manager: Arc<CollectionManager>,

    /// Batch processor
    batch_processor: Arc<BatchProcessor>,

    /// Service configuration
    config: ApiServiceConfig,
}

/// Configuration for the enhanced API service
#[derive(Debug, Clone)]
pub struct ApiServiceConfig {
    /// Enable enhanced search features
    pub enable_enhanced_search: bool,

    /// Enable collection management
    pub enable_collection_management: bool,

    /// Enable batch operations
    pub enable_batch_operations: bool,

    /// Maximum search results per request
    pub max_search_results: usize,

    /// Maximum collections per user/session
    pub max_collections_per_user: usize,

    /// Maximum concurrent batch operations
    pub max_concurrent_batch_ops: usize,

    /// Default collection for operations without explicit collection
    pub default_collection: Option<String>,

    /// Enable API rate limiting
    pub enable_rate_limiting: bool,

    /// Rate limit (requests per minute)
    pub rate_limit_per_minute: u32,
}

impl Default for ApiServiceConfig {
    fn default() -> Self {
        Self {
            enable_enhanced_search: std::env::var("API_ENABLE_ENHANCED_SEARCH")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            enable_collection_management: std::env::var("API_ENABLE_COLLECTION_MANAGEMENT")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            enable_batch_operations: std::env::var("API_ENABLE_BATCH_OPERATIONS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            max_search_results: std::env::var("API_MAX_SEARCH_RESULTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            max_collections_per_user: std::env::var("API_MAX_COLLECTIONS_PER_USER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            max_concurrent_batch_ops: std::env::var("API_MAX_CONCURRENT_BATCH_OPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            default_collection: std::env::var("API_DEFAULT_COLLECTION").ok(),
            enable_rate_limiting: std::env::var("API_ENABLE_RATE_LIMITING")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            rate_limit_per_minute: std::env::var("API_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        }
    }
}

/// Enhanced search API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnhancedSearchRequest {
    /// Search query
    pub query: String,

    /// Collection to search (optional, uses default if not specified)
    pub collection: Option<String>,

    /// Maximum number of results
    pub limit: Option<usize>,

    /// Minimum confidence threshold
    pub min_confidence: Option<f32>,

    /// Enable semantic search
    pub semantic_search: Option<bool>,

    /// Metadata filters
    pub metadata_filters: Option<HashMap<String, serde_json::Value>>,

    /// Enable search refinement suggestions
    pub enable_refinements: Option<bool>,

    /// Enable result grouping
    pub enable_grouping: Option<bool>,

    /// Custom ranking weights
    pub ranking_weights: Option<HashMap<String, f32>>,
}

/// Enhanced search API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnhancedSearchResponse {
    /// Search results
    pub results: Vec<EnhancedSearchResult>,

    /// Total number of results found
    pub total_results: usize,

    /// Search metadata
    pub metadata: SearchResponseMetadata,

    /// Search refinement suggestions (if enabled)
    pub refinements: Option<Vec<String>>,

    /// Result groupings (if enabled)
    pub groupings: Option<HashMap<String, Vec<String>>>,
}

/// Metadata included in search responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponseMetadata {
    /// Collection searched
    pub collection: String,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Whether semantic search was used
    pub semantic_search_used: bool,

    /// Search confidence statistics
    pub confidence_stats: ConfidenceStats,

    /// Cache hit information
    pub cache_hit: bool,
}

/// Confidence score statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceStats {
    /// Average confidence across all results
    pub average: f32,

    /// Minimum confidence score
    pub minimum: f32,

    /// Maximum confidence score
    pub maximum: f32,

    /// Standard deviation of confidence scores
    pub std_deviation: f32,
}

/// Collection management API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCreateCollectionRequest {
    /// Collection name
    pub name: String,

    /// Collection description
    pub description: Option<String>,

    /// Vector dimension
    pub vector_dimension: Option<usize>,

    /// Distance metric (cosine, euclidean, etc.)
    pub distance_metric: Option<String>,

    /// Initial metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Enable semantic search for this collection
    pub enable_semantic_search: Option<bool>,

    /// Default search limit for this collection
    pub default_search_limit: Option<usize>,
}

/// Collection information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCollectionInfo {
    /// Collection configuration
    pub config: CollectionConfig,

    /// Collection statistics
    pub stats: Option<CollectionStats>,

    /// Collection health status
    pub health: CollectionHealthStatus,
}

/// Simple collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Number of documents
    pub document_count: u64,

    /// Total size in bytes
    pub total_size_bytes: u64,

    /// Average document size
    pub avg_document_size: f64,

    /// Last updated timestamp
    pub last_updated: u64,
}

/// Collection health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionHealthStatus {
    /// Collection is healthy and operational
    Healthy,

    /// Collection has minor issues but is operational
    Warning { message: String },

    /// Collection has errors and may not be fully operational
    Error { message: String },

    /// Collection is not accessible
    Unavailable,
}

/// Cross-collection search API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCrossCollectionSearchRequest {
    /// Base search request
    pub search_request: ApiEnhancedSearchRequest,

    /// Collections to search (if None, search all active collections)
    pub target_collections: Option<Vec<String>>,

    /// Whether to merge results or return per-collection
    pub merge_results: Option<bool>,

    /// Maximum results per collection
    pub max_results_per_collection: Option<usize>,
}

/// Cross-collection search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCrossCollectionSearchResponse {
    /// Merged results (if merge_results = true)
    pub merged_results: Option<Vec<EnhancedSearchResult>>,

    /// Results by collection (if merge_results = false)
    pub collection_results: Option<HashMap<String, Vec<EnhancedSearchResult>>>,

    /// Search statistics
    pub search_stats: CrossCollectionStats,
}

/// Cross-collection search statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCollectionStats {
    /// Collections searched
    pub collections_searched: Vec<String>,

    /// Total results found
    pub total_results: usize,

    /// Results per collection
    pub results_per_collection: HashMap<String, usize>,

    /// Total processing time
    pub total_processing_time_ms: u64,
}

/// Batch operation API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBatchOperationRequest {
    /// Operation type and parameters
    pub operation: BatchOperationType,

    /// Processing priority (1-10, higher = more priority)
    pub priority: Option<u8>,

    /// Callback URL for completion notification
    pub callback_url: Option<String>,

    /// Maximum processing time (seconds)
    pub timeout_seconds: Option<u64>,

    /// Whether to continue processing on individual errors
    pub continue_on_error: Option<bool>,

    /// Custom operation metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Batch operation status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBatchOperationResponse {
    /// Operation ID
    pub operation_id: String,

    /// Detailed operation result
    pub result: BatchOperationResult,

    /// Quick status summary
    pub summary: BatchOperationSummary,
}

/// Quick summary of batch operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationSummary {
    /// Current status
    pub status: String,

    /// Completion percentage
    pub completion_percentage: f64,

    /// Items processed so far
    pub processed_items: usize,

    /// Total items to process
    pub total_items: usize,

    /// Estimated remaining time (seconds)
    pub estimated_remaining_seconds: Option<u64>,

    /// Whether the operation has any errors
    pub has_errors: bool,

    /// Number of errors encountered
    pub error_count: usize,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Additional error details
    pub details: Option<HashMap<String, serde_json::Value>>,

    /// Timestamp when error occurred
    pub timestamp: u64,

    /// Request ID for tracking
    pub request_id: Option<String>,
}

impl EnhancedApiService {
    pub fn new(
        search_service: Arc<EnhancedSearchService>,
        collection_manager: Arc<CollectionManager>,
        batch_processor: Arc<BatchProcessor>,
        config: ApiServiceConfig,
    ) -> Self {
        Self {
            search_service,
            collection_manager,
            batch_processor,
            config,
        }
    }

    /// Perform enhanced search with additional API-level features
    pub async fn enhanced_search(
        &self,
        request: ApiEnhancedSearchRequest,
    ) -> Result<ApiEnhancedSearchResponse, ApiErrorResponse> {
        if !self.config.enable_enhanced_search {
            return Err(ApiErrorResponse {
                code: "FEATURE_DISABLED".to_string(),
                message: "Enhanced search is not enabled".to_string(),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            });
        }

        // Validate request limits
        if let Some(limit) = request.limit {
            if limit > self.config.max_search_results {
                return Err(ApiErrorResponse {
                    code: "LIMIT_EXCEEDED".to_string(),
                    message: format!(
                        "Search limit {} exceeds maximum {}",
                        limit, self.config.max_search_results
                    ),
                    details: None,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    request_id: None,
                });
            }
        }

        let start_time = std::time::Instant::now();

        // Convert API request to internal request
        let internal_request = EnhancedSearchRequest {
            query: request.query.clone(),
            limit: request.limit,
            min_confidence: request.min_confidence,
            collections: request.collection.as_ref().map(|c| vec![c.clone()]).or(self
                .config
                .default_collection
                .as_ref()
                .map(|c| vec![c.clone()])),
            metadata_filters: None, // PENDING: Metadata filter conversion from JSON requires enhanced query parser
            ranking: None,          // or map from request if available
            include_scores: true,   // or map from request if available
            include_explanations: false, // or map from request if available
        };

        // Perform the search
        let results = match self.search_service.enhanced_search(internal_request).await {
            Ok(results) => results,
            Err(error) => {
                return Err(ApiErrorResponse {
                    code: "SEARCH_ERROR".to_string(),
                    message: format!("Search failed: {}", error),
                    details: None,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    request_id: None,
                });
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Calculate confidence statistics
        let confidence_stats = if !results.is_empty() {
            let scores: Vec<f32> = results.iter().map(|r| r.confidence_score).collect();
            let sum: f32 = scores.iter().sum();
            let avg = sum / scores.len() as f32;
            let min = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

            let variance: f32 = scores
                .iter()
                .map(|&score| (score - avg).powi(2))
                .sum::<f32>()
                / scores.len() as f32;
            let std_dev = variance.sqrt();

            ConfidenceStats {
                average: avg,
                minimum: min,
                maximum: max,
                std_deviation: std_dev,
            }
        } else {
            ConfidenceStats {
                average: 0.0,
                minimum: 0.0,
                maximum: 0.0,
                std_deviation: 0.0,
            }
        };

        // Build response metadata
        let metadata = SearchResponseMetadata {
            collection: request.collection.unwrap_or_else(|| "default".to_string()),
            processing_time_ms: processing_time,
            semantic_search_used: request.semantic_search.unwrap_or(true),
            confidence_stats,
            cache_hit: false, // PENDING: Cache hit detection requires result cache infrastructure
        };

        // PENDING: Result refinements and groupings require advanced query analysis (Phase 5)
        let refinements = if request.enable_refinements.unwrap_or(false) {
            Some(vec![
                "Try more specific terms".to_string(),
                "Use synonyms".to_string(),
                "Check spelling".to_string(),
            ])
        } else {
            None
        };

        let groupings = if request.enable_grouping.unwrap_or(false) {
            let mut groups = HashMap::new();
            groups.insert(
                "High Confidence".to_string(),
                results
                    .iter()
                    .filter(|r| r.confidence_score > 0.8)
                    .map(|r| r.id.clone())
                    .collect(),
            );
            groups.insert(
                "Medium Confidence".to_string(),
                results
                    .iter()
                    .filter(|r| r.confidence_score >= 0.5 && r.confidence_score <= 0.8)
                    .map(|r| r.id.clone())
                    .collect(),
            );
            Some(groups)
        } else {
            None
        };

        Ok(ApiEnhancedSearchResponse {
            total_results: results.len(),
            results,
            metadata,
            refinements,
            groupings,
        })
    }

    /// Create a new collection through the API
    pub async fn create_collection(
        &self,
        request: ApiCreateCollectionRequest,
    ) -> Result<ApiCollectionInfo, ApiErrorResponse> {
        if !self.config.enable_collection_management {
            return Err(ApiErrorResponse {
                code: "FEATURE_DISABLED".to_string(),
                message: "Collection management is not enabled".to_string(),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            });
        }

        // Convert API request to internal request
        let internal_request = CreateCollectionRequest {
            name: request.name,
            description: request.description,
            vector_config: crate::infrastructure::collection_management::VectorStoreConfig {
                dimension: request.vector_dimension.unwrap_or(384),
                metric: request
                    .distance_metric
                    .unwrap_or_else(|| "cosine".to_string()),
                index_type: "hnsw".to_string(),
                index_params: std::collections::HashMap::new(),
                max_vectors: None,
            },
            search_config: Some(crate::infrastructure::collection_management::SearchConfig {
                default_limit: request.default_search_limit.unwrap_or(10),
                max_limit: 1000,
                enable_metadata_filtering: true,
                ranking_weights: std::collections::HashMap::new(),
            }),
            metadata: request.metadata,
        };

        // Create the collection
        let config = match self
            .collection_manager
            .create_collection(internal_request)
            .await
        {
            Ok(config) => config,
            Err(error) => {
                return Err(ApiErrorResponse {
                    code: "COLLECTION_CREATION_FAILED".to_string(),
                    message: format!("Failed to create collection: {}", error),
                    details: None,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    request_id: None,
                });
            }
        };

        // Get collection statistics (will be empty for new collection)
        let stats = self
            .collection_manager
            .get_collection_stats(&config.id)
            .await;
        let api_stats = stats.map(|s| CollectionStats {
            document_count: s.document_count,
            total_size_bytes: s.total_size_bytes,
            avg_document_size: s.avg_document_size,
            last_updated: s.last_updated,
        });

        Ok(ApiCollectionInfo {
            config,
            stats: api_stats,
            health: CollectionHealthStatus::Healthy,
        })
    }

    /// Perform cross-collection search
    pub async fn cross_collection_search(
        &self,
        request: ApiCrossCollectionSearchRequest,
    ) -> Result<ApiCrossCollectionSearchResponse, ApiErrorResponse> {
        if !self.config.enable_enhanced_search {
            return Err(ApiErrorResponse {
                code: "FEATURE_DISABLED".to_string(),
                message: "Enhanced search is not enabled".to_string(),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            });
        }

        // Convert API request to internal request
        let internal_search_request = EnhancedSearchRequest {
            query: request.search_request.query,
            limit: request.search_request.limit,
            min_confidence: request.search_request.min_confidence,
            collections: request
                .search_request
                .collection
                .as_ref()
                .map(|c| vec![c.clone()]),
            metadata_filters: None, // PENDING: Cross-collection metadata filtering requires unified schema
            ranking: None,
            include_scores: true,
            include_explanations: false,
        };

        let internal_request = CrossCollectionSearchRequest {
            search_request: internal_search_request,
            target_collections: request.target_collections,
            merge_results: request.merge_results.unwrap_or(true),
            max_results_per_collection: request.max_results_per_collection,
        };

        // Perform the search
        let response = match self
            .collection_manager
            .cross_collection_search(internal_request)
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return Err(ApiErrorResponse {
                    code: "CROSS_COLLECTION_SEARCH_FAILED".to_string(),
                    message: format!("Cross-collection search failed: {}", error),
                    details: None,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    request_id: None,
                });
            }
        };

        // Convert internal response to API response
        let search_stats = CrossCollectionStats {
            collections_searched: response.search_stats.collections_searched,
            total_results: response.search_stats.total_results,
            results_per_collection: response.search_stats.results_per_collection,
            total_processing_time_ms: response.search_stats.total_processing_time_ms,
        };

        Ok(ApiCrossCollectionSearchResponse {
            merged_results: response.merged_results,
            collection_results: response.collection_results,
            search_stats,
        })
    }

    /// Submit a batch operation
    pub async fn submit_batch_operation(
        &self,
        request: ApiBatchOperationRequest,
    ) -> Result<ApiBatchOperationResponse, ApiErrorResponse> {
        if !self.config.enable_batch_operations {
            return Err(ApiErrorResponse {
                code: "FEATURE_DISABLED".to_string(),
                message: "Batch operations are not enabled".to_string(),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            });
        }

        // Convert API request to internal request
        let internal_request = BatchOperationRequest {
            operation_id: None, // Let the processor generate it
            operation: request.operation,
            config: crate::infrastructure::batch_operations::BatchProcessingConfig {
                max_concurrency: Some(self.config.max_concurrent_batch_ops),
                batch_size: None, // Use default
                timeout_seconds: request.timeout_seconds,
                continue_on_error: request.continue_on_error,
                max_retries: None,       // Use default
                progress_interval: None, // Use default
                memory_limit_mb: None,   // Use default
            },
            priority: request.priority,
            callback_url: request.callback_url,
            metadata: request.metadata,
        };

        // Submit the operation
        let operation_id = match self
            .batch_processor
            .submit_operation(internal_request)
            .await
        {
            Ok(id) => id,
            Err(error) => {
                return Err(ApiErrorResponse {
                    code: "BATCH_OPERATION_SUBMISSION_FAILED".to_string(),
                    message: format!("Failed to submit batch operation: {}", error),
                    details: None,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    request_id: None,
                });
            }
        };

        // Get the operation status
        let result = self
            .batch_processor
            .get_operation_status(&operation_id)
            .await
            .ok_or_else(|| ApiErrorResponse {
                code: "OPERATION_NOT_FOUND".to_string(),
                message: format!("Operation {} not found after submission", operation_id),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            })?;

        // Create summary
        let summary = BatchOperationSummary {
            status: format!("{:?}", result.status),
            completion_percentage: result.progress.completion_percentage,
            processed_items: result.progress.processed_items,
            total_items: result.progress.total_items,
            estimated_remaining_seconds: result.progress.estimated_remaining_seconds,
            has_errors: !result.errors.is_empty(),
            error_count: result.errors.len(),
        };

        Ok(ApiBatchOperationResponse {
            operation_id,
            result,
            summary,
        })
    }

    /// Get batch operation status
    pub async fn get_batch_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<ApiBatchOperationResponse, ApiErrorResponse> {
        if !self.config.enable_batch_operations {
            return Err(ApiErrorResponse {
                code: "FEATURE_DISABLED".to_string(),
                message: "Batch operations are not enabled".to_string(),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            });
        }

        let result = self
            .batch_processor
            .get_operation_status(operation_id)
            .await
            .ok_or_else(|| ApiErrorResponse {
                code: "OPERATION_NOT_FOUND".to_string(),
                message: format!("Operation {} not found", operation_id),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            })?;

        let summary = BatchOperationSummary {
            status: format!("{:?}", result.status),
            completion_percentage: result.progress.completion_percentage,
            processed_items: result.progress.processed_items,
            total_items: result.progress.total_items,
            estimated_remaining_seconds: result.progress.estimated_remaining_seconds,
            has_errors: !result.errors.is_empty(),
            error_count: result.errors.len(),
        };

        Ok(ApiBatchOperationResponse {
            operation_id: operation_id.to_string(),
            result,
            summary,
        })
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<ApiCollectionInfo>, ApiErrorResponse> {
        if !self.config.enable_collection_management {
            return Err(ApiErrorResponse {
                code: "FEATURE_DISABLED".to_string(),
                message: "Collection management is not enabled".to_string(),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            });
        }

        let collections = self.collection_manager.list_collections().await;
        let mut api_collections = Vec::new();

        for config in collections {
            let stats = self
                .collection_manager
                .get_collection_stats(&config.id)
                .await;
            let api_stats = stats.map(|s| CollectionStats {
                document_count: s.document_count,
                total_size_bytes: s.total_size_bytes,
                avg_document_size: s.avg_document_size,
                last_updated: s.last_updated,
            });

            api_collections.push(ApiCollectionInfo {
                config,
                stats: api_stats,
                health: CollectionHealthStatus::Healthy, // TODO: Implement health checking
            });
        }

        Ok(api_collections)
    }

    /// Get collection information
    pub async fn get_collection(
        &self,
        collection_id: &str,
    ) -> Result<ApiCollectionInfo, ApiErrorResponse> {
        if !self.config.enable_collection_management {
            return Err(ApiErrorResponse {
                code: "FEATURE_DISABLED".to_string(),
                message: "Collection management is not enabled".to_string(),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            });
        }

        let config = self
            .collection_manager
            .get_collection(collection_id)
            .await
            .ok_or_else(|| ApiErrorResponse {
                code: "COLLECTION_NOT_FOUND".to_string(),
                message: format!("Collection {} not found", collection_id),
                details: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                request_id: None,
            })?;

        let stats = self
            .collection_manager
            .get_collection_stats(collection_id)
            .await;
        let api_stats = stats.map(|s| CollectionStats {
            document_count: s.document_count,
            total_size_bytes: s.total_size_bytes,
            avg_document_size: s.avg_document_size,
            last_updated: s.last_updated,
        });

        Ok(ApiCollectionInfo {
            config,
            stats: api_stats,
            health: CollectionHealthStatus::Healthy, // TODO: Implement health checking
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_service_config_default() {
        let config = ApiServiceConfig::default();
        assert!(config.enable_enhanced_search);
        assert!(config.enable_collection_management);
        assert!(config.enable_batch_operations);
        assert_eq!(config.max_search_results, 1000);
    }

    #[test]
    fn test_confidence_stats_calculation() {
        let scores = vec![0.9, 0.8, 0.7, 0.6, 0.5];
        let sum: f32 = scores.iter().sum();
        let avg = sum / scores.len() as f32;

        assert_eq!(avg, 0.7);

        let min = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        assert_eq!(min, 0.5);
        assert_eq!(max, 0.9);
    }

    #[test]
    fn test_api_error_response_serialization() {
        let error = ApiErrorResponse {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            details: None,
            timestamp: 1234567890,
            request_id: Some("req_123".to_string()),
        };

        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ApiErrorResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(error.code, deserialized.code);
        assert_eq!(error.message, deserialized.message);
        assert_eq!(error.timestamp, deserialized.timestamp);
    }
}
