use crate::infrastructure::jsonrpc::types::{HealthCheckResult, LivenessResult, ReadinessResult};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
/// HTTP route handlers for the doc-indexer API
///
/// This module contains the HTTP handlers that translate between HTTP requests/responses
/// and the application services, following the clean architecture pattern.
use std::sync::Arc;
use std::time::Instant;
use zero_latency_contracts::api::endpoints;
use zero_latency_core::ZeroLatencyError;
use zero_latency_search::traits::{PopularQuery, SearchAnalytics, SearchTrends};

use crate::application::{
    CollectionService, DocumentIndexingService, HealthService, ServiceContainer,
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub container: Arc<ServiceContainer>,
    pub document_service: DocumentIndexingService,
    pub health_service: HealthService,
    pub collection_service: CollectionService,
    pub analytics_service: Arc<crate::infrastructure::analytics::ProductionSearchAnalytics>,
    pub start_time: Instant,
}

impl AppState {
    /// Create a new AppState and initialize collection stats from actual data
    pub async fn new_async(container: Arc<ServiceContainer>) -> zero_latency_core::Result<Self> {
        // Check configuration for enhanced search features
        let config = container.config();
        let enable_query_enhancement = config.service.enable_query_enhancement;
        let enable_result_ranking = config.service.enable_result_ranking;

        // Create enhanced search components if enabled
        use crate::infrastructure::search_enhancement::{
            MultiFactorResultRanker, SimpleQueryEnhancer,
        };
        use std::sync::Arc;

        let query_enhancer = if enable_query_enhancement {
            tracing::info!("[AdvancedSearch] Creating SimpleQueryEnhancer");
            Some(Arc::new(SimpleQueryEnhancer::new()) as Arc<dyn zero_latency_search::QueryEnhancer>)
        } else {
            tracing::info!("[AdvancedSearch] Query enhancement disabled");
            None
        };

        let result_ranker = if enable_result_ranking {
            tracing::info!("[AdvancedSearch] Creating MultiFactorResultRanker");
            Some(Arc::new(MultiFactorResultRanker::new())
                as Arc<dyn zero_latency_search::ResultRanker>)
        } else {
            tracing::info!("[AdvancedSearch] Result ranking disabled");
            None
        };

        // Create document service with enhanced search capabilities
        let document_service = if query_enhancer.is_some() || result_ranker.is_some() {
            tracing::info!(
                "[AdvancedSearch] Initializing enhanced search pipeline - Query Enhancement: {}, Result Ranking: {}",
                enable_query_enhancement,
                enable_result_ranking
            );
            tracing::info!(
                "[AdvancedSearch] Enhancer available: {}, Ranker available: {}",
                query_enhancer.is_some(),
                result_ranker.is_some()
            );

            use crate::application::services::filter_service::IndexingFilters;
            let default_filters = IndexingFilters::new();
            DocumentIndexingService::with_enhanced_search(
                &container,
                default_filters,
                query_enhancer,
                result_ranker,
            )
        } else {
            tracing::info!("[AdvancedSearch] Using basic document service (no enhancements)");
            DocumentIndexingService::new(&container)
        };

        let health_service = HealthService::new();
        let collection_service = CollectionService::new(&container);

        // Initialize collection stats from actual vector repository
        collection_service.initialize().await?;

        // Use the analytics service from the container (shared with search pipeline)
        let analytics_service = container.analytics();

        Ok(Self {
            container,
            document_service,
            health_service,
            collection_service,
            analytics_service,
            start_time: Instant::now(),
        })
    }
}

/// Create the application router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // API endpoints (expected by CLI)
        .route(endpoints::STATUS, get(api_status))
        .route(endpoints::SEARCH, post(search_documents))
        .route(endpoints::INDEX, post(index_documents_from_path))
        .route(endpoints::REINDEX, post(reindex_documents))
        .route(endpoints::SERVER_START, post(start_server))
        .route(endpoints::SERVER_STOP, post(stop_server))
        // Collection endpoints
        .route(endpoints::COLLECTIONS, get(list_collections))
        .route(endpoints::COLLECTIONS, post(create_collection))
        .route(endpoints::COLLECTION_BY_NAME, get(get_collection))
        .route(endpoints::COLLECTION_BY_NAME, delete(delete_collection))
        .route(endpoints::COLLECTION_STATS, get(get_collection_stats))
        // Document endpoints (read-only for discovery)
        .route(endpoints::DOCUMENTS, get(list_documents))
        .route(endpoints::DOCUMENT_BY_ID, get(get_document))
        .route(endpoints::DOCUMENTS_SEARCH, post(search_documents))
        // Analytics endpoints - partially enabled for testing
        .route(endpoints::ANALYTICS_SUMMARY, get(get_analytics_summary))
        .route(
            endpoints::ANALYTICS_POPULAR_QUERIES,
            get(get_popular_queries),
        )
        .route(endpoints::ANALYTICS_SEARCH_TRENDS, get(get_search_trends))
        // Health endpoints
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        // Info endpoints
        .route("/info", get(service_info))
        .with_state(state)
}

/// List all documents with pagination
async fn list_documents(
    Query(params): Query<ListDocumentsQuery>,
    State(state): State<AppState>,
) -> Result<Json<ListDocumentsResponse>, AppError> {
    // Extract parameters with defaults
    let collection_name = params
        .collection
        .unwrap_or_else(|| "zero_latency_docs".to_string());
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.limit.unwrap_or(50).min(100);

    // Get collection-specific document count and metadata
    let (total_count, index_size) = match state
        .collection_service
        .get_collection_info(&collection_name)
        .await
    {
        Ok(Some(collection)) => (collection.vector_count, collection.size_bytes),
        Ok(None) | Err(_) => {
            // Collection doesn't exist or error occurred, return empty result
            return Ok(Json(ListDocumentsResponse {
                documents: vec![],
                total_count: 0,
                page,
                per_page,
                total_pages: 0,
                index_size_bytes: 0,
            }));
        }
    };

    // Calculate pagination
    let total_pages = if total_count == 0 {
        0
    } else {
        (total_count as f64 / per_page as f64).ceil() as u64
    };
    let skip = (page - 1) * per_page;

    // Get actual documents from the vector repository using a dummy search
    // Since VectorRepository doesn't have a list method, we'll use search with a zero vector
    // to get real document metadata instead of fake placeholders
    let documents = if total_count > 0 {
        // Get the vector repository to search for real documents
        let vector_repo = state.document_service.vector_repository();

        // Use a zero vector or small query to get real documents
        // This is a workaround since we don't have a direct list method
        let search_limit = std::cmp::min(per_page as usize + skip as usize, 100); // Get a bit more to handle pagination
        let dummy_vector = vec![0.0f32; 384]; // Assuming 384-dimensional embeddings

        match vector_repo.search(dummy_vector, search_limit).await {
            Ok(search_results) => {
                // Extract real document metadata from search results
                let documents_to_return = std::cmp::min(per_page, total_count.saturating_sub(skip));
                search_results
                    .into_iter()
                    .skip(skip as usize)
                    .take(documents_to_return as usize)
                    .map(|result| {
                        let metadata = &result.metadata;
                        DocumentSummary {
                            id: metadata.document_id.to_string(),
                            title: if metadata.title.is_empty() {
                                "Untitled Document".to_string()
                            } else {
                                // Extract just the filename from the title, removing .md extension if present
                                let title = metadata.title.clone();
                                if title.ends_with(".md") {
                                    title[..title.len() - 3].to_string()
                                } else {
                                    title
                                }
                            },
                            path: metadata.url.clone().unwrap_or_else(|| {
                                // Use the title as the path since it contains the filename
                                metadata.title.clone()
                            }),
                            size: metadata.content.len() as u64,
                            last_modified: metadata
                                .custom
                                .get("modified_date")
                                .or_else(|| metadata.custom.get("last_modified"))
                                .cloned()
                                .unwrap_or_else(|| "Unknown".to_string()),
                        }
                    })
                    .collect()
            }
            Err(e) => {
                eprintln!("Failed to get documents from vector store: {}", e);
                vec![] // Return empty on error
            }
        }
    } else {
        vec![]
    };

    Ok(Json(ListDocumentsResponse {
        documents,
        total_count,
        page,
        per_page,
        total_pages,
        index_size_bytes: index_size,
    }))
}

/// Get a document by ID (this would typically get from index)
async fn get_document(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<GetDocumentResponse>, AppError> {
    // In a real implementation, this would retrieve the document from the index
    // For now, return a placeholder response
    Ok(Json(GetDocumentResponse {
        id,
        found: false,
        content: None,
        metadata: None,
    }))
}

/// Search for documents
async fn search_documents(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<zero_latency_search::SearchResponse>, AppError> {
    let default_collection = &state.container.config().service.default_collection;
    let collection_name = request.collection.as_deref().unwrap_or(default_collection);
    let limit = request.limit.unwrap_or(10);

    let search_response = state
        .document_service
        .search_documents_in_collection(&request.query, collection_name, limit)
        .await?;

    Ok(Json(search_response))
}

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthCheckResult>, AppError> {
    let health = state.health_service.health_check().await?;
    Ok(Json(health))
}

/// API status endpoint (CLI-compatible format)
async fn api_status(State(state): State<AppState>) -> Json<ApiStatusResponse> {
    // Get actual metrics from the services
    let document_count = state
        .document_service
        .get_document_count()
        .await
        .unwrap_or(0);
    let index_size = state.document_service.get_index_size().await.unwrap_or(0);
    let uptime_seconds = state.start_time.elapsed().as_secs();

    Json(ApiStatusResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        total_documents: document_count,
        index_size_bytes: index_size,
        last_index_update: None, // Would get from last indexing operation
        docs_path: Some(
            state
                .container
                .config()
                .service
                .docs_path
                .display()
                .to_string(),
        ),
    })
}

/// Readiness check endpoint
async fn readiness_check(State(state): State<AppState>) -> Result<Json<ReadinessResult>, AppError> {
    let readiness = state.health_service.readiness_check().await?;
    Ok(Json(readiness))
}

/// Liveness check endpoint
async fn liveness_check(State(state): State<AppState>) -> Result<Json<LivenessResult>, AppError> {
    let liveness = state.health_service.liveness_check().await?;
    Ok(Json(liveness))
}

/// Service information endpoint
async fn service_info(State(_state): State<AppState>) -> Json<ServiceInfoResponse> {
    Json(ServiceInfoResponse {
        name: "doc-indexer".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Document indexing and search service".to_string(),
        features: vec![
            "document_indexing".to_string(),
            "vector_search".to_string(),
            "health_monitoring".to_string(),
        ],
    })
}

/// Index documents from a specified path
#[tracing::instrument(skip(state), fields(path = %request.path, collection = %request.collection.as_ref().unwrap_or(&"zero_latency_docs".to_string())))]
async fn index_documents_from_path(
    State(state): State<AppState>,
    Json(request): Json<IndexPathRequest>,
) -> Result<Json<IndexPathResponse>, AppError> {
    let collection_name = request.collection.as_deref().unwrap_or("zero_latency_docs");
    tracing::info!(
        "Starting document indexing from path: {} into collection: {}",
        request.path,
        collection_name
    );

    // Create filtering configuration from request
    let filters = if request.safe_patterns.is_some()
        || request.ignore_patterns.is_some()
        || request.clear_default_ignores.is_some()
        || request.follow_symlinks.is_some()
        || request.case_sensitive.is_some()
    {
        use crate::application::services::filter_service::IndexingFilters;

        let mut ignore_list = if request.clear_default_ignores.unwrap_or(false) {
            Vec::new() // Start with empty ignore list
        } else {
            IndexingFilters::new().ignore_list // Use default ignores
        };

        // Add any additional ignore patterns
        if let Some(additional_ignores) = request.ignore_patterns {
            ignore_list.extend(additional_ignores);
        }

        Some(IndexingFilters {
            safe_list: request.safe_patterns.unwrap_or_default(),
            ignore_list,
            case_sensitive: request.case_sensitive.unwrap_or(false),
            follow_symlinks: request.follow_symlinks.unwrap_or(false),
        })
    } else {
        None
    };

    // Ensure the collection exists (create if it doesn't exist)
    if collection_name != "zero_latency_docs" {
        let collections = state.collection_service.list_collections().await?;
        if !collections.iter().any(|c| c.name == collection_name) {
            use crate::application::services::collection_service::CreateCollectionRequest;
            tracing::info!("Creating new collection: {}", collection_name);
            state
                .collection_service
                .create_collection(CreateCollectionRequest {
                    name: collection_name.to_string(),
                    description: Some(format!("Collection created for indexing {}", request.path)),
                    vector_size: 384, // Default embedding size
                    distance_metric: Some("cosine".to_string()),
                })
                .await?;
        }
    }

    // Use the document service to actually index documents with collection context
    let result = state
        .document_service
        .index_documents_from_path_with_filters_and_collection(
            &request.path,
            request.recursive.unwrap_or(true),
            filters,
            collection_name,
        )
        .await;

    match result {
        Ok((documents_processed, processing_time_ms)) => {
            tracing::info!(
                documents_processed = documents_processed,
                processing_time_ms = processing_time_ms,
                "Indexing completed successfully"
            );

            // Update collection statistics after successful indexing
            if let Err(e) = update_collection_statistics(&state, collection_name).await {
                tracing::warn!("Failed to update collection statistics: {}", e);
            }

            Ok(Json(IndexPathResponse {
                documents_processed,
                processing_time_ms,
                status: "success".to_string(),
                message: Some(format!(
                    "Successfully indexed {} documents from path: {}",
                    documents_processed, request.path
                )),
            }))
        }
        Err(e) => {
            tracing::error!(error = %e, path = %request.path, "Failed to index documents");
            Err(AppError(e))
        }
    }
}

/// Reindex all documents (equivalent to clearing and re-indexing)
#[tracing::instrument(skip(state), fields(collection = ?request.collection))]
async fn reindex_documents(
    State(state): State<AppState>,
    Json(request): Json<ReindexRequest>,
) -> Result<Json<ReindexResponse>, AppError> {
    tracing::info!("Starting full reindex operation");

    // Get the current collection name (either from request or default)
    let collection_name = request.collection.as_deref().unwrap_or("zero_latency_docs"); // Default collection

    // Step 1: Delete the existing collection to clear all vectors
    tracing::info!("Clearing existing collection: {}", collection_name);
    let _deleted = state
        .collection_service
        .delete_collection(collection_name)
        .await?;

    // Step 2: Recreate the collection with default settings
    tracing::info!("Recreating collection: {}", collection_name);
    use crate::application::services::collection_service::CreateCollectionRequest;
    let create_request = CreateCollectionRequest {
        name: collection_name.to_string(),
        vector_size: 384, // Default embedding size
        distance_metric: Some("cosine".to_string()),
        description: Some(format!("Reindexed collection: {}", collection_name)),
    };
    let _collection = state
        .collection_service
        .create_collection(create_request)
        .await?;

    // Step 3: Use the same path as configured in the CLI
    // In a full implementation, we'd store the original indexing paths
    let default_path = std::env::current_dir()
        .map_err(|e| {
            zero_latency_core::ZeroLatencyError::external_service("filesystem", e.to_string())
        })?
        .to_string_lossy()
        .to_string();

    // Create filtering configuration from request
    let filters = if request.safe_patterns.is_some()
        || request.ignore_patterns.is_some()
        || request.clear_default_ignores.is_some()
        || request.follow_symlinks.is_some()
        || request.case_sensitive.is_some()
    {
        use crate::application::services::filter_service::IndexingFilters;

        let mut ignore_list = if request.clear_default_ignores.unwrap_or(false) {
            Vec::new() // Start with empty ignore list
        } else {
            IndexingFilters::new().ignore_list // Use default ignores
        };

        // Add any additional ignore patterns
        if let Some(additional_ignores) = request.ignore_patterns {
            ignore_list.extend(additional_ignores);
        }

        Some(IndexingFilters {
            safe_list: request.safe_patterns.unwrap_or_default(),
            ignore_list,
            case_sensitive: request.case_sensitive.unwrap_or(false),
            follow_symlinks: request.follow_symlinks.unwrap_or(false),
        })
    } else {
        None
    };

    // For reindexing, we first clear the existing index and then rebuild it
    // TODO: In production, implement atomic reindexing with backup/restore
    let result = state
        .document_service
        .index_documents_from_path_with_filters_and_collection(
            &default_path,
            true,
            filters,
            collection_name,
        )
        .await;

    match result {
        Ok((documents_processed, processing_time_ms)) => {
            tracing::info!(
                documents_processed = documents_processed,
                processing_time_ms = processing_time_ms,
                "Reindexing completed successfully"
            );

            // Update collection statistics after successful reindexing
            if let Err(e) = update_collection_statistics(&state, collection_name).await {
                tracing::warn!("Failed to update collection statistics: {}", e);
            }

            Ok(Json(ReindexResponse {
                documents_processed,
                processing_time_ms,
                status: "completed".to_string(),
                message: Some(format!(
                    "Successfully reindexed {} documents",
                    documents_processed
                )),
            }))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to reindex documents");
            Err(AppError(e))
        }
    }
}

/// Update collection statistics after indexing
#[tracing::instrument(skip(state))]
async fn update_collection_statistics(
    state: &AppState,
    collection_name: &str,
) -> Result<(), zero_latency_core::ZeroLatencyError> {
    // Get current vector count from the vector repository
    let vector_count = state.container.vector_repository().count().await? as u64;

    // Estimate size (384 dimensions * 4 bytes per float + metadata overhead)
    let estimated_size_bytes = vector_count * (384 * 4 + 100); // ~1640 bytes per vector with metadata

    // Update statistics for the specified collection
    state
        .collection_service
        .update_collection_stats(collection_name, vector_count, estimated_size_bytes)
        .await?;

    tracing::info!(
        collection = collection_name,
        vector_count = vector_count,
        size_bytes = estimated_size_bytes,
        "Updated collection statistics"
    );

    Ok(())
}

/// Start server endpoint (for CLI compatibility)
async fn start_server(
    State(_state): State<AppState>,
    Json(_request): Json<ServerStartRequest>,
) -> Result<Json<ServerResponse>, AppError> {
    // Server is already running if this endpoint is being called
    Ok(Json(ServerResponse {
        success: true,
        message: "Server is already running".to_string(),
        status: "running".to_string(),
    }))
}

/// Stop server endpoint (for CLI compatibility)
async fn stop_server(
    State(_state): State<AppState>,
    Json(_request): Json<ServerStopRequest>,
) -> Result<Json<ServerResponse>, AppError> {
    // Note: This would typically initiate graceful shutdown
    // For now, just return a response
    Ok(Json(ServerResponse {
        success: true,
        message: "Server shutdown initiated".to_string(),
        status: "stopping".to_string(),
    }))
}

// Request/Response types

#[derive(Debug, Serialize)]
pub struct GetDocumentResponse {
    pub id: String,
    pub found: bool,
    pub content: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub collection: Option<String>,
    pub limit: Option<usize>,
    #[allow(dead_code)]
    pub filters: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct IndexPathRequest {
    pub path: String,
    pub collection: Option<String>,
    #[allow(dead_code)]
    pub recursive: Option<bool>,
    #[allow(dead_code)]
    pub force: Option<bool>,
    #[allow(dead_code)]
    pub safe_patterns: Option<Vec<String>>,
    #[allow(dead_code)]
    pub ignore_patterns: Option<Vec<String>>,
    #[allow(dead_code)]
    pub clear_default_ignores: Option<bool>,
    #[allow(dead_code)]
    pub follow_symlinks: Option<bool>,
    #[allow(dead_code)]
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct IndexPathResponse {
    pub documents_processed: u64,
    pub processing_time_ms: f64,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReindexRequest {
    pub collection: Option<String>,
    #[allow(dead_code)]
    pub force: Option<bool>,
    #[allow(dead_code)]
    pub safe_patterns: Option<Vec<String>>,
    #[allow(dead_code)]
    pub ignore_patterns: Option<Vec<String>>,
    #[allow(dead_code)]
    pub clear_default_ignores: Option<bool>,
    #[allow(dead_code)]
    pub follow_symlinks: Option<bool>,
    #[allow(dead_code)]
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ReindexResponse {
    pub documents_processed: u64,
    pub processing_time_ms: f64,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerStartRequest {
    #[allow(dead_code)]
    pub host: Option<String>,
    #[allow(dead_code)]
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct ServerStopRequest {
    // Empty for now
}

#[derive(Debug, Serialize)]
pub struct ServerResponse {
    pub success: bool,
    pub message: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ApiStatusResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub total_documents: u64,
    pub index_size_bytes: u64,
    pub last_index_update: Option<String>,
    pub docs_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentSummary>,
    pub total_count: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
    pub index_size_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub collection: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct DocumentSummary {
    pub id: String,
    pub title: String,
    pub path: String,
    pub size: u64,
    pub last_modified: String,
}

// Error handling

#[derive(Debug)]
pub struct AppError(ZeroLatencyError);

impl From<ZeroLatencyError> for AppError {
    fn from(err: ZeroLatencyError) -> Self {
        Self(err)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self.0 {
            ZeroLatencyError::Validation { field, message } => {
                (StatusCode::BAD_REQUEST, format!("{}: {}", field, message))
            }
            ZeroLatencyError::NotFound { resource } => {
                (StatusCode::NOT_FOUND, format!("Not found: {}", resource))
            }
            ZeroLatencyError::Configuration { message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            ZeroLatencyError::Internal { message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            ZeroLatencyError::ExternalService { service, message } => {
                (StatusCode::BAD_GATEWAY, format!("{}: {}", service, message))
            }
            ZeroLatencyError::Database { message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            ZeroLatencyError::Network { message } => (StatusCode::BAD_GATEWAY, message.clone()),
            ZeroLatencyError::Serialization { message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            ZeroLatencyError::PermissionDenied { operation } => (
                StatusCode::FORBIDDEN,
                format!("Permission denied: {}", operation),
            ),
        };

        let error_response = serde_json::json!({
            "error": {
                "message": message,
                "type": format!("{:?}", self.0)
            }
        });

        (status, Json(error_response)).into_response()
    }
}

//
// Collection API Handlers
//

/// List all collections
async fn list_collections(
    State(state): State<AppState>,
) -> Result<Json<ListCollectionsResponse>, AppError> {
    let collections = state.collection_service.list_collections().await?;
    Ok(Json(ListCollectionsResponse { collections }))
}

/// Get a specific collection
async fn get_collection(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<GetCollectionResponse>, AppError> {
    if let Some(collection) = state.collection_service.get_collection_info(&name).await? {
        Ok(Json(GetCollectionResponse {
            found: true,
            collection: Some(collection),
        }))
    } else {
        Ok(Json(GetCollectionResponse {
            found: false,
            collection: None,
        }))
    }
}

/// Create a new collection
async fn create_collection(
    State(state): State<AppState>,
    Json(request): Json<CreateCollectionApiRequest>,
) -> Result<Json<CreateCollectionResponse>, AppError> {
    use crate::application::services::collection_service::CreateCollectionRequest;

    let create_request = CreateCollectionRequest {
        name: request.name,
        vector_size: request.vector_size,
        distance_metric: request.distance_metric,
        description: request.description,
    };

    let collection = state
        .collection_service
        .create_collection(create_request)
        .await?;
    Ok(Json(CreateCollectionResponse {
        success: true,
        collection,
        message: "Collection created successfully".to_string(),
    }))
}

/// Delete a collection
async fn delete_collection(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DeleteCollectionResponse>, AppError> {
    let deleted = state.collection_service.delete_collection(&name).await?;
    Ok(Json(DeleteCollectionResponse {
        success: deleted,
        message: if deleted {
            "Collection deleted successfully".to_string()
        } else {
            "Collection not found".to_string()
        },
    }))
}

/// Get collection statistics
async fn get_collection_stats(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<GetCollectionStatsResponse>, AppError> {
    if let Some(stats) = state.collection_service.get_collection_stats(&name).await? {
        Ok(Json(GetCollectionStatsResponse {
            found: true,
            stats: Some(stats),
        }))
    } else {
        Ok(Json(GetCollectionStatsResponse {
            found: false,
            stats: None,
        }))
    }
}

//
// Collection API Request/Response Types
//

/// Response for listing collections
#[derive(Debug, Serialize, Deserialize)]
pub struct ListCollectionsResponse {
    pub collections: Vec<crate::application::services::collection_service::CollectionInfo>,
}

/// Response for getting a collection
#[derive(Debug, Serialize, Deserialize)]
pub struct GetCollectionResponse {
    pub found: bool,
    pub collection: Option<crate::application::services::collection_service::CollectionInfo>,
}

/// Request for creating a collection
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCollectionApiRequest {
    pub name: String,
    pub vector_size: u64,
    pub distance_metric: Option<String>,
    pub description: Option<String>,
}

/// Response for creating a collection
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCollectionResponse {
    pub success: bool,
    pub collection: crate::application::services::collection_service::CollectionInfo,
    pub message: String,
}

/// Response for deleting a collection
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteCollectionResponse {
    pub success: bool,
    pub message: String,
}

/// Response for getting collection statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct GetCollectionStatsResponse {
    pub found: bool,
    pub stats: Option<crate::application::services::collection_service::CollectionStats>,
}

// ===== Analytics Handlers =====

/// Get comprehensive analytics summary
async fn get_analytics_summary(
    State(state): State<AppState>,
) -> Result<Json<crate::infrastructure::analytics::AnalyticsSummary>, AppError> {
    tracing::info!("[Analytics] Getting analytics summary");

    let summary = state.analytics_service.get_analytics_summary().await;
    tracing::info!(
        "[Analytics] Retrieved analytics summary with {} total searches",
        summary.total_searches
    );
    Ok(Json(summary))
}

/// Get popular queries with limit parameter
async fn get_popular_queries(
    State(state): State<AppState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<Vec<PopularQuery>>, AppError> {
    tracing::info!(
        "[Analytics] Getting popular queries with limit: {}",
        params.limit.unwrap_or(10)
    );

    match state
        .analytics_service
        .get_popular_queries(params.limit.unwrap_or(10))
        .await
    {
        Ok(queries) => {
            tracing::info!("[Analytics] Retrieved {} popular queries", queries.len());
            Ok(Json(queries))
        }
        Err(e) => {
            tracing::error!("[Analytics] Error getting popular queries: {}", e);
            Err(AppError(ZeroLatencyError::internal(format!(
                "Analytics error: {}",
                e
            ))))
        }
    }
}

/// Get search trends data
async fn get_search_trends(State(state): State<AppState>) -> Result<Json<SearchTrends>, AppError> {
    tracing::info!("[Analytics] Getting search trends");

    match state.analytics_service.get_search_trends().await {
        Ok(trends) => {
            tracing::info!(
                "[Analytics] Retrieved search trends with {} total searches",
                trends.total_searches
            );
            Ok(Json(trends))
        }
        Err(e) => {
            tracing::error!("[Analytics] Error getting search trends: {}", e);
            Err(AppError(ZeroLatencyError::internal(format!(
                "Analytics error: {}",
                e
            ))))
        }
    }
}

/// Query parameters for analytics endpoints
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub limit: Option<usize>,
}
