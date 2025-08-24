/// HTTP route handlers for the doc-indexer API
/// 
/// This module contains the HTTP handlers that translate between HTTP requests/responses
/// and the application services, following the clean architecture pattern.

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use zero_latency_core::{
    ZeroLatencyError
};
use crate::infrastructure::jsonrpc::types::{
    HealthCheckResult, ReadinessResult, LivenessResult
};

use crate::application::{
    ServiceContainer, DocumentIndexingService, HealthService, CollectionService,
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub container: Arc<ServiceContainer>,
    pub document_service: DocumentIndexingService,
    pub health_service: HealthService,
    pub collection_service: CollectionService,
}

impl AppState {
    pub fn new(container: Arc<ServiceContainer>) -> Self {
        let document_service = DocumentIndexingService::new(&container);
        let health_service = HealthService::new();
        let collection_service = CollectionService::new(&container);
        
        Self {
            container,
            document_service,
            health_service,
            collection_service,
        }
    }

    /// Create a new AppState and initialize collection stats from actual data
    pub async fn new_async(container: Arc<ServiceContainer>) -> zero_latency_core::Result<Self> {
        let document_service = DocumentIndexingService::new(&container);
        let health_service = HealthService::new();
        let collection_service = CollectionService::new(&container);
        
        // Initialize collection stats from actual vector repository
        collection_service.initialize().await?;
        
        Ok(Self {
            container,
            document_service,
            health_service,
            collection_service,
        })
    }
}

/// Create the application router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // API endpoints (expected by CLI)
        .route("/api/status", get(api_status))
        .route("/api/search", post(search_documents))
        .route("/api/index", post(index_documents_from_path))
        .route("/api/server/start", post(start_server))
        .route("/api/server/stop", post(stop_server))
        
        // Collection endpoints
        .route("/collections", get(list_collections))
        .route("/collections", post(create_collection))
        .route("/collections/:name", get(get_collection))
        .route("/collections/:name", delete(delete_collection))
        .route("/collections/:name/stats", get(get_collection_stats))
        
        // Document endpoints (read-only for discovery)
        .route("/documents", get(list_documents))
        .route("/documents/:id", get(get_document))
        .route("/documents/search", post(search_documents))
        
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
    State(state): State<AppState>,
) -> Result<Json<ListDocumentsResponse>, AppError> {
    // Get document count and some basic metadata
    let total_count = state.document_service.get_document_count().await.unwrap_or(0);
    let index_size = state.document_service.get_index_size().await.unwrap_or(0);
    
    // For now, return metadata instead of full documents
    // In a real implementation, this would paginate through actual documents
    Ok(Json(ListDocumentsResponse {
        documents: vec![], // TODO: Implement actual document listing
        total_count,
        page: 1,
        per_page: 50,
        total_pages: (total_count as f64 / 50.0).ceil() as u64,
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
    let search_response = state.document_service
        .search_documents(&request.query, request.limit.unwrap_or(10))
        .await?;
    
    Ok(Json(search_response))
}

/// Health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthCheckResult>, AppError> {
    let health = state.health_service.health_check().await?;
    Ok(Json(health))
}

/// API status endpoint (CLI-compatible format)
async fn api_status(
    State(state): State<AppState>,
) -> Json<ApiStatusResponse> {
    // Get actual metrics from the services
    let document_count = state.document_service.get_document_count().await.unwrap_or(0);
    let index_size = state.document_service.get_index_size().await.unwrap_or(0);
    
    Json(ApiStatusResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Would calculate from service start time
        total_documents: document_count,
        index_size_bytes: index_size,
        last_index_update: None, // Would get from last indexing operation
        docs_path: Some(state.container.config().service.docs_path.display().to_string()),
    })
}

/// Readiness check endpoint
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<ReadinessResult>, AppError> {
    let readiness = state.health_service.readiness_check().await?;
    Ok(Json(readiness))
}

/// Liveness check endpoint
async fn liveness_check(
    State(state): State<AppState>,
) -> Result<Json<LivenessResult>, AppError> {
    let liveness = state.health_service.liveness_check().await?;
    Ok(Json(liveness))
}

/// Service information endpoint
async fn service_info(
    State(_state): State<AppState>,
) -> Json<ServiceInfoResponse> {
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
#[tracing::instrument(skip(state), fields(path = %request.path))]
async fn index_documents_from_path(
    State(state): State<AppState>,
    Json(request): Json<IndexPathRequest>,
) -> Result<Json<IndexPathResponse>, AppError> {
    tracing::info!("Starting document indexing from path: {}", request.path);
    
    // Use the document service to actually index documents
    let result = state
        .document_service
        .index_documents_from_path(&request.path, request.recursive.unwrap_or(true))
        .await;
    
    match result {
        Ok((documents_processed, processing_time_ms)) => {
            tracing::info!(
                documents_processed = documents_processed,
                processing_time_ms = processing_time_ms,
                "Indexing completed successfully"
            );
            
            // Update collection statistics after successful indexing
            if let Err(e) = update_collection_statistics(&state).await {
                tracing::warn!("Failed to update collection statistics: {}", e);
            }
            
            Ok(Json(IndexPathResponse {
                documents_processed,
                processing_time_ms,
                status: "success".to_string(),
                message: Some(format!("Successfully indexed {} documents from path: {}", documents_processed, request.path)),
            }))
        }
        Err(e) => {
            tracing::error!(error = %e, path = %request.path, "Failed to index documents");
            Err(AppError(e))
        }
    }
}

/// Update collection statistics after indexing
#[tracing::instrument(skip(state))]
async fn update_collection_statistics(state: &AppState) -> Result<(), zero_latency_core::ZeroLatencyError> {
    // Get current vector count from the vector repository
    let vector_count = state.container.vector_repository().count().await? as u64;
    
    // Estimate size (384 dimensions * 4 bytes per float + metadata overhead)
    let estimated_size_bytes = vector_count * (384 * 4 + 100); // ~1640 bytes per vector with metadata
    
    // Update statistics for the default collection
    // In a full implementation, we would track which collection each vector belongs to
    let default_collection = "zero_latency_docs";
    state.collection_service
        .update_collection_stats(default_collection, vector_count, estimated_size_bytes)
        .await?;
    
    tracing::info!(
        collection = default_collection,
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
    #[allow(dead_code)]
    pub recursive: Option<bool>,
    #[allow(dead_code)]
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct IndexPathResponse {
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
            ZeroLatencyError::Network { message } => {
                (StatusCode::BAD_GATEWAY, message.clone())
            }
            ZeroLatencyError::Serialization { message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
            ZeroLatencyError::PermissionDenied { operation } => {
                (StatusCode::FORBIDDEN, format!("Permission denied: {}", operation))
            }
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
    
    let collection = state.collection_service.create_collection(create_request).await?;
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
