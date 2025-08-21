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
    models::{Document, HealthCheckResult, ReadinessResult, LivenessResult}, 
    ZeroLatencyError
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
        // Document endpoints
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
) -> Result<Json<SearchResponse>, AppError> {
    let search_response = state.document_service
        .search_documents(&request.query, request.limit.unwrap_or(10))
        .await?;
    
    let results: Vec<SearchResultItem> = search_response.results.into_iter().map(|result| {
        SearchResultItem {
            id: result.document_id.to_string(),
            content: result.content,
            score: result.final_score.value(),
            metadata: std::collections::HashMap::new(), // No custom metadata in SearchResult
        }
    }).collect();
    
    let total = results.len();
    
    Ok(Json(SearchResponse {
        query: request.query,
        results,
        total,
    }))
}

/// Health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthCheckResult>, AppError> {
    let health = state.health_service.health_check().await?;
    Ok(Json(health))
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
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResultItem>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
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
