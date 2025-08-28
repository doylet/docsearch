//! REST Protocol Adapter
//! 
//! Implements the REST API protocol adapter using generated types from the OpenAPI specification.
//! This adapter provides HTTP/REST endpoint handlers that validate requests/responses against
//! the schema and delegate to domain services.

use crate::application::ServiceContainer;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;
use zero_latency_api::{endpoints, types::*};
use zero_latency_core::ZeroLatencyError;

/// REST protocol adapter
#[derive(Clone)]
pub struct RestAdapter {
    container: Arc<ServiceContainer>,
}

impl RestAdapter {
    /// Create new REST adapter
    pub fn new(container: Arc<ServiceContainer>) -> Self {
        Self { container }
    }
    
    /// Create the complete REST API router
    pub fn create_router(self) -> Router {
        Router::new()
            // Health endpoints
            .route(endpoints::HEALTH, get(Self::health_check))
            .route(endpoints::HEALTH_READY, get(Self::readiness_check))
            .route(endpoints::HEALTH_LIVE, get(Self::liveness_check))
            
            // API status
            .route(endpoints::STATUS, get(Self::api_status))
            
            // Search endpoints
            .route(endpoints::SEARCH, post(Self::search_documents))
            .route(endpoints::DOCUMENTS_SEARCH, post(Self::search_documents))
            
            // Indexing endpoints
            .route(endpoints::INDEX, post(Self::index_documents))
            .route(endpoints::REINDEX, post(Self::reindex_documents))
            
            // Collection management
            .route(endpoints::COLLECTIONS, get(Self::list_collections))
            .route(endpoints::COLLECTIONS, post(Self::create_collection))
            .route(endpoints::COLLECTION_BY_NAME, get(Self::get_collection))
            .route(endpoints::COLLECTION_BY_NAME, delete(Self::delete_collection))
            .route(endpoints::COLLECTION_STATS, get(Self::get_collection_stats))
            
            // Document management
            .route(endpoints::DOCUMENTS, get(Self::list_documents))
            .route(endpoints::DOCUMENT_BY_ID, get(Self::get_document))
            
            // Server management
            .route(endpoints::SERVER_START, post(Self::start_server))
            .route(endpoints::SERVER_STOP, post(Self::stop_server))
            
            // Analytics
            .route(endpoints::ANALYTICS_SUMMARY, get(Self::analytics_summary))
            .route(endpoints::ANALYTICS_POPULAR_QUERIES, get(Self::popular_queries))
            .route(endpoints::ANALYTICS_SEARCH_TRENDS, get(Self::search_trends))
            
            .with_state(self)
    }
    
    /// Extract tenant context from headers
    fn extract_tenant_context(&self, headers: &HeaderMap) -> TenantContext {
        headers
            .get("X-Tenant-ID")
            .and_then(|h| h.to_str().ok())
            .map(|id| TenantContext::new(id.to_string()))
            .unwrap_or_default()
    }

    /// Create router with all REST endpoints
    pub fn create_router(&self) -> Router {
        Router::new()
            .route("/health", get(Self::health_check))
            .route("/search", post(Self::search_documents))
            .route("/index", post(Self::index_documents))
            .route("/collections", get(Self::list_collections))
            .route("/collections", post(Self::create_collection))
            .route("/collections/:collection_id", get(Self::get_collection))
            .route("/collections/:collection_id", put(Self::update_collection))
            .route("/collections/:collection_id", delete(Self::delete_collection))
            .route("/collections/:collection_id/documents", get(Self::list_documents))
            .route("/collections/:collection_id/documents", post(Self::add_document))
            .route("/collections/:collection_id/documents/:document_id", get(Self::get_document))
            .route("/collections/:collection_id/documents/:document_id", put(Self::update_document))
            .route("/collections/:collection_id/documents/:document_id", delete(Self::delete_document))
            .route("/server/status", get(Self::api_status))
            .route("/server/restart", post(Self::restart_server))
            .route("/server/shutdown", post(Self::shutdown_server))
            .route("/analytics/search", get(Self::analytics_summary))
            .route("/analytics/usage", get(Self::analytics_summary))
            .with_state(self.clone())
    }    /// Convert domain errors to API errors
    fn map_error(error: ZeroLatencyError) -> ApiError {
        ApiError {
            error: "Internal server error".to_string(),
            message: error.to_string(),
            code: "INTERNAL_ERROR".to_string(),
            trace_id: None,
            details: None,
        }
    }
}

// Health endpoints
impl RestAdapter {
    #[instrument(skip(state))]
    async fn health_check(State(state): State<RestAdapter>) -> impl IntoResponse {
        debug!("Processing health check request");
        
        // Basic health check - service is running if we can respond
        let response = HealthCheckResult {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        Json(response).into_response()
    }
    
    #[instrument(skip(state))]
    async fn readiness_check(State(state): State<RestAdapter>) -> impl IntoResponse {
        debug!("Processing readiness check request");
        
        // Basic readiness check - verify we can access core services
        let response = HealthCheckResult {
            status: "ready".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        Json(response).into_response()
    }
    
    #[instrument(skip(state))]
    async fn liveness_check(State(state): State<RestAdapter>) -> impl IntoResponse {
        debug!("Processing liveness check request");
        
        // Basic liveness check - service is alive if we can respond
        let response = HealthCheckResult {
            status: "alive".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        Json(response).into_response()
    }
    
    #[instrument(skip(state))]
    async fn api_status(State(state): State<RestAdapter>) -> impl IntoResponse {
        debug!("Processing API status request");
        
        // For now, create a simple status response
        let response = ApiStatusResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 0, // TODO: Track actual uptime
        };
        
        Json(response).into_response()
    }
}

// Search endpoints
impl RestAdapter {
    #[instrument(skip(state, headers))]
    async fn search_documents(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
        Json(request): Json<SearchRequest>,
    ) -> impl IntoResponse {
        debug!("Processing search request: {:?}", request);
        
        // Extract tenant context
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        // Convert to domain search request
        let domain_request = zero_latency_search::SearchRequest {
            query: request.query.clone(),
            limit: request.limit.unwrap_or(10),
            offset: request.offset.unwrap_or(0),
        };
        
        match state.container.document_service.search(domain_request).await {
            Ok(results) => {
                let response = SearchResponse {
                    results: results.results.into_iter().map(|r| SearchResult {
                        id: r.id,
                        title: r.title,
                        content: r.content,
                        score: r.score,
                    }).collect(),
                    total: results.total,
                    query_time_ms: results.query_time_ms,
                };
                
                info!("Search completed successfully, found {} results", response.total);
                Json(response).into_response()
            }
            Err(e) => {
                error!("Search failed: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
}

// Indexing endpoints
impl RestAdapter {
    #[instrument(skip(state, headers))]
    async fn index_documents(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
        Json(request): Json<IndexRequest>,
    ) -> impl IntoResponse {
        debug!("Processing index request: {:?}", request);
        
        // Extract tenant context
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        // Convert to domain index request
        let domain_request = crate::application::IndexRequest {
            path: request.path.clone(),
            collection: request.collection.clone(),
            recursive: request.recursive.unwrap_or(false),
        };
        
        match state.container.document_service.index_documents(domain_request).await {
            Ok(result) => {
                let response = IndexResponse {
                    indexed_documents: result.indexed_documents,
                    errors: result.errors,
                };
                
                info!("Indexing completed, {} documents indexed", response.indexed_documents);
                Json(response).into_response()
            }
            Err(e) => {
                error!("Indexing failed: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
    
    #[instrument(skip(state, headers))]
    async fn reindex_documents(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
        Json(request): Json<IndexRequest>,
    ) -> impl IntoResponse {
        debug!("Processing reindex request: {:?}", request);
        
        // Extract tenant context  
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        // For reindexing, we can use the same logic but with force flag
        let domain_request = crate::application::IndexRequest {
            path: request.path.clone(),
            collection: request.collection.clone(),
            recursive: request.recursive.unwrap_or(false),
        };
        
        match state.container.document_service.reindex_documents(domain_request).await {
            Ok(result) => {
                let response = IndexResponse {
                    indexed_documents: result.indexed_documents,
                    errors: result.errors,
                };
                
                info!("Reindexing completed, {} documents processed", response.indexed_documents);
                Json(response).into_response()
            }
            Err(e) => {
                error!("Reindexing failed: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
}

// Collection endpoints
impl RestAdapter {
    #[instrument(skip(state, headers))]
    async fn list_collections(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        debug!("Processing list collections request");
        
        // Extract tenant context
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        match state.container.collection_service.list_collections().await {
            Ok(collections) => {
                let response: Vec<Collection> = collections.into_iter().map(|c| Collection {
                    name: c.name,
                    description: c.description,
                    document_count: c.document_count,
                    created_at: c.created_at,
                }).collect();
                
                info!("Listed {} collections", response.len());
                Json(response).into_response()
            }
            Err(e) => {
                error!("Failed to list collections: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
    
    #[instrument(skip(state, headers))]
    async fn create_collection(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
        Json(request): Json<Collection>,
    ) -> impl IntoResponse {
        debug!("Processing create collection request: {:?}", request.name);
        
        // Extract tenant context
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        let domain_request = crate::application::CreateCollectionRequest {
            name: request.name.clone(),
            description: request.description.clone(),
        };
        
        match state.container.collection_service.create_collection(domain_request).await {
            Ok(collection) => {
                let response = Collection {
                    name: collection.name,
                    description: collection.description,
                    document_count: collection.document_count,
                    created_at: collection.created_at,
                };
                
                info!("Created collection: {}", response.name);
                (StatusCode::CREATED, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to create collection: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
    
    #[instrument(skip(state, headers, path))]
    async fn get_collection(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
        Path(name): Path<String>,
    ) -> impl IntoResponse {
        debug!("Processing get collection request: {}", name);
        
        // Extract tenant context
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        match state.container.collection_service.get_collection(&name).await {
            Ok(Some(collection)) => {
                let response = Collection {
                    name: collection.name,
                    description: collection.description,
                    document_count: collection.document_count,
                    created_at: collection.created_at,
                };
                
                Json(response).into_response()
            }
            Ok(None) => {
                let error = ApiError {
                    error: "Collection not found".to_string(),
                    message: format!("Collection '{}' does not exist", name),
                    code: "COLLECTION_NOT_FOUND".to_string(),
                    trace_id: None,
                    details: None,
                };
                (StatusCode::NOT_FOUND, Json(error)).into_response()
            }
            Err(e) => {
                error!("Failed to get collection: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
    
    #[instrument(skip(state, headers, path))]
    async fn delete_collection(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
        Path(name): Path<String>,
    ) -> impl IntoResponse {
        debug!("Processing delete collection request: {}", name);
        
        // Extract tenant context
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        match state.container.collection_service.delete_collection(&name).await {
            Ok(true) => {
                info!("Deleted collection: {}", name);
                StatusCode::NO_CONTENT.into_response()
            }
            Ok(false) => {
                let error = ApiError {
                    error: "Collection not found".to_string(),
                    message: format!("Collection '{}' does not exist", name),
                    code: "COLLECTION_NOT_FOUND".to_string(),
                    trace_id: None,
                    details: None,
                };
                (StatusCode::NOT_FOUND, Json(error)).into_response()
            }
            Err(e) => {
                error!("Failed to delete collection: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
    
    #[instrument(skip(state, headers, path))]
    async fn get_collection_stats(
        State(state): State<RestAdapter>,
        headers: HeaderMap,
        Path(name): Path<String>,
    ) -> impl IntoResponse {
        debug!("Processing get collection stats request: {}", name);
        
        // Extract tenant context
        let _tenant_id = match state.extract_tenant_context(&headers) {
            Ok(tenant) => tenant,
            Err(error) => return (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        };
        
        // For now, return basic stats - this would be extended with actual analytics
        match state.container.collection_service.get_collection(&name).await {
            Ok(Some(collection)) => {
                #[derive(Serialize)]
                struct CollectionStats {
                    name: String,
                    document_count: i64,
                    total_size_bytes: i64,
                    last_updated: chrono::DateTime<chrono::Utc>,
                }
                
                let stats = CollectionStats {
                    name: collection.name,
                    document_count: collection.document_count,
                    total_size_bytes: 0, // TODO: Calculate actual size
                    last_updated: collection.created_at,
                };
                
                Json(stats).into_response()
            }
            Ok(None) => {
                let error = ApiError {
                    error: "Collection not found".to_string(),
                    message: format!("Collection '{}' does not exist", name),
                    code: "COLLECTION_NOT_FOUND".to_string(),
                    trace_id: None,
                    details: None,
                };
                (StatusCode::NOT_FOUND, Json(error)).into_response()
            }
            Err(e) => {
                error!("Failed to get collection stats: {}", e);
                let error = Self::map_error(e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
            }
        }
    }
}

// Document endpoints - placeholder implementations
impl RestAdapter {
    async fn list_documents(State(_state): State<RestAdapter>) -> impl IntoResponse {
        // TODO: Implement document listing
        StatusCode::NOT_IMPLEMENTED
    }
    
    async fn get_document(State(_state): State<RestAdapter>, Path(_id): Path<String>) -> impl IntoResponse {
        // TODO: Implement document retrieval
        StatusCode::NOT_IMPLEMENTED
    }
}

// Server management endpoints - placeholder implementations
impl RestAdapter {
    async fn start_server(State(_state): State<RestAdapter>) -> impl IntoResponse {
        // TODO: Implement server start
        StatusCode::NOT_IMPLEMENTED
    }
    
    async fn stop_server(State(_state): State<RestAdapter>) -> impl IntoResponse {
        // TODO: Implement server stop
        StatusCode::NOT_IMPLEMENTED
    }
}

// Analytics endpoints - placeholder implementations
impl RestAdapter {
    async fn analytics_summary(State(_state): State<RestAdapter>) -> impl IntoResponse {
        // TODO: Implement analytics summary
        StatusCode::NOT_IMPLEMENTED
    }
    
    async fn popular_queries(State(_state): State<RestAdapter>) -> impl IntoResponse {
        // TODO: Implement popular queries
        StatusCode::NOT_IMPLEMENTED
    }
    
    async fn search_trends(State(_state): State<RestAdapter>) -> impl IntoResponse {
        // TODO: Implement search trends
        StatusCode::NOT_IMPLEMENTED
    }
}
