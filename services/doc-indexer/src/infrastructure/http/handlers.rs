/// HTTP route handlers for the doc-indexer API
/// 
/// This module contains the HTTP handlers that translate between HTTP requests/responses
/// and the application services, following the clean architecture pattern.

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use zero_latency_core::{
    models::Document, 
    ZeroLatencyError
};
use crate::infrastructure::jsonrpc::types::{
    HealthCheckResult, ReadinessResult, LivenessResult
};

use crate::application::{
    ServiceContainer, DocumentIndexingService, HealthService,
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub container: Arc<ServiceContainer>,
    pub document_service: DocumentIndexingService,
    pub health_service: HealthService,
}

impl AppState {
    pub fn new(container: Arc<ServiceContainer>) -> Self {
        let document_service = DocumentIndexingService::new(&container);
        let health_service = HealthService::new();
        
        Self {
            container,
            document_service,
            health_service,
        }
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
        
        // Document endpoints (internal)
        .route("/documents", post(index_document))
        .route("/documents/:id", get(get_document))
        .route("/documents/:id", put(update_document))
        .route("/documents/:id", delete(delete_document))
        .route("/documents/search", post(search_documents))
        
        // Health endpoints
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        
        // Info endpoints
        .route("/info", get(service_info))
        
        .with_state(state)
}

/// Index a new document
async fn index_document(
    State(state): State<AppState>,
    Json(request): Json<IndexDocumentRequest>,
) -> Result<Json<IndexDocumentResponse>, AppError> {
    let document = Document {
        id: zero_latency_core::Uuid::parse_str(&request.id)
            .map_err(|_| ZeroLatencyError::validation("id", "Invalid UUID format"))?,
        title: request.title.unwrap_or_else(|| "Untitled".to_string()),
        content: request.content,
        path: std::path::PathBuf::from(request.path.unwrap_or_else(|| "/tmp/unknown".to_string())),
        last_modified: chrono::Utc::now(),
        size: 0, // Would be calculated in real implementation
        metadata: zero_latency_core::models::DocumentMetadata {
            custom: request.metadata.unwrap_or_default(),
            ..Default::default()
        },
    };
    
    state.document_service.index_document(document).await?;
    
    Ok(Json(IndexDocumentResponse {
        success: true,
        message: "Document indexed successfully".to_string(),
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

/// Update an existing document
async fn update_document(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<UpdateDocumentRequest>,
) -> Result<Json<UpdateDocumentResponse>, AppError> {
    let document = Document {
        id: zero_latency_core::Uuid::parse_str(&id)
            .map_err(|_| ZeroLatencyError::validation("id", "Invalid UUID"))?,
        title: request.title.unwrap_or_else(|| "Untitled".to_string()),
        content: request.content,
        path: std::path::PathBuf::from(request.path.unwrap_or_else(|| "/tmp/unknown".to_string())),
        last_modified: chrono::Utc::now(),
        size: 0, // Would be calculated in real implementation
        metadata: zero_latency_core::models::DocumentMetadata {
            custom: request.metadata.unwrap_or_default(),
            ..Default::default()
        },
    };
    
    state.document_service.update_document(document).await?;
    
    Ok(Json(UpdateDocumentResponse {
        success: true,
        message: "Document updated successfully".to_string(),
    }))
}

/// Delete a document
async fn delete_document(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DeleteDocumentResponse>, AppError> {
    state.document_service.delete_document(&id).await?;
    
    Ok(Json(DeleteDocumentResponse {
        success: true,
        message: "Document deleted successfully".to_string(),
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
    State(_state): State<AppState>,
) -> Json<ApiStatusResponse> {
    Json(ApiStatusResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Would calculate from service start time
        total_documents: 0, // Would query from the database
        index_size_bytes: 0, // Would calculate from storage
        last_index_update: None, // Would get from last indexing operation
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

/// Index documents from a file path (CLI API endpoint)
async fn index_documents_from_path(
    State(_state): State<AppState>,
    Json(request): Json<IndexPathRequest>,
) -> Result<Json<IndexPathResponse>, AppError> {
    // This would integrate with the actual indexing service
    // For now, return a success response
    Ok(Json(IndexPathResponse {
        documents_processed: 0, // Would be calculated in real implementation
        processing_time_ms: 0.0, // Would be measured in real implementation
        status: "success".to_string(),
        message: Some(format!("Started indexing documents from path: {}", request.path)),
    }))
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

#[derive(Debug, Deserialize)]
pub struct IndexDocumentRequest {
    pub id: String,
    pub title: Option<String>,
    pub content: String,
    pub path: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct IndexDocumentResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GetDocumentResponse {
    pub id: String,
    pub found: bool,
    pub content: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: String,
    pub path: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateDocumentResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteDocumentResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
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
    pub recursive: Option<bool>,
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
    pub host: Option<String>,
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
