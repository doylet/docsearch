use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::search_service::{
    SearchService, SearchRequest, SearchResponse, HealthResponse, 
    StatusResponse, DocumentListResponse, DocumentDetailResponse, ReindexResponse
};

/// Pagination query parameters
#[derive(Debug, Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_page_size")]
    page_size: usize,
}

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 20 }

/// HTTP API server state
#[derive(Clone)]
pub struct ApiState {
    pub search_service: Arc<Mutex<SearchService>>,
}

/// API server
pub struct ApiServer {
    app: Router,
    state: ApiState,
}

impl ApiServer {
    pub fn new(search_service: SearchService) -> Self {
        let state = ApiState {
            search_service: Arc::new(Mutex::new(search_service)),
        };

        let app = Router::new()
            .route("/api/search", post(search_handler))
            .route("/api/status", get(status_handler))
            .route("/api/health", get(health_handler))
            .route("/api/docs", get(list_documents_handler))
            .route("/api/docs/:id", get(get_document_handler))
            .route("/api/docs/:id", delete(delete_document_handler))
            .route("/api/reindex", post(reindex_handler))
            .route("/", get(root_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
                    .into_inner(),
            )
            .with_state(state.clone());

        Self { app, state }
    }

    pub fn router(&self) -> Router {
        self.app.clone()
    }

    pub async fn serve(self, port: u16) -> Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        info!("Starting HTTP API server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Failed to bind to {}", addr))?;

        axum::serve(listener, self.app)
            .await
            .context("HTTP server error")?;

        Ok(())
    }
}

/// GET /api/status - System status and statistics
async fn status_handler(State(state): State<ApiState>) -> Result<Json<StatusResponse>, ApiError> {
    let search_service = state.search_service.lock().await;
    let status = search_service
        .status()
        .await
        .map_err(|e| {
            error!("Status check error: {}", e);
            ApiError::InternalServerError("Status check failed".to_string())
        })?;

    Ok(Json(status))
}

/// GET /api/docs - List all indexed documents with pagination
async fn list_documents_handler(
    State(state): State<ApiState>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<DocumentListResponse>, ApiError> {
    // Validate pagination parameters
    if params.page == 0 {
        return Err(ApiError::BadRequest("Page must be >= 1".to_string()));
    }
    
    if params.page_size == 0 || params.page_size > 100 {
        return Err(ApiError::BadRequest("Page size must be between 1 and 100".to_string()));
    }

    let search_service = state.search_service.lock().await;
    let documents = search_service
        .list_documents(params.page, params.page_size)
        .await
        .map_err(|e| {
            error!("Document listing error: {}", e);
            ApiError::InternalServerError("Failed to list documents".to_string())
        })?;

    Ok(Json(documents))
}

/// GET /api/docs/{id} - Get specific document details
async fn get_document_handler(
    State(state): State<ApiState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentDetailResponse>, ApiError> {
    if doc_id.trim().is_empty() {
        return Err(ApiError::BadRequest("Document ID cannot be empty".to_string()));
    }

    let search_service = state.search_service.lock().await;
    let document = search_service
        .get_document(&doc_id)
        .await
        .map_err(|e| {
            error!("Document retrieval error: {}", e);
            ApiError::InternalServerError("Failed to retrieve document".to_string())
        })?;

    match document {
        Some(doc) => Ok(Json(doc)),
        None => Err(ApiError::NotFound(format!("Document not found: {}", doc_id))),
    }
}

/// DELETE /api/docs/{id} - Remove document from index
async fn delete_document_handler(
    State(state): State<ApiState>,
    Path(doc_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if doc_id.trim().is_empty() {
        return Err(ApiError::BadRequest("Document ID cannot be empty".to_string()));
    }

    let search_service = state.search_service.lock().await;
    search_service
        .delete_document(&doc_id)
        .await
        .map_err(|e| {
            error!("Document deletion error: {}", e);
            ApiError::InternalServerError("Failed to delete document".to_string())
        })?;

    info!("Document deleted: {}", doc_id);
    
    Ok(Json(json!({
        "status": "deleted",
        "document_id": doc_id,
        "message": "Document successfully removed from index"
    })))
}

/// POST /api/reindex - Rebuild the entire search index
async fn reindex_handler(State(state): State<ApiState>) -> Result<Json<ReindexResponse>, ApiError> {
    info!("Reindex operation started");
    
    let search_service = state.search_service.lock().await;
    let result = search_service
        .reindex()
        .await
        .map_err(|e| {
            error!("Reindex error: {}", e);
            ApiError::InternalServerError("Reindex operation failed".to_string())
        })?;

    info!("Reindex operation completed: {} documents processed in {}ms", 
          result.documents_processed, result.total_time_ms);
    
    Ok(Json(result))
}

/// POST /api/search - Semantic search endpoint
async fn search_handler(
    State(state): State<ApiState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    info!("Search request: query={}, k={}", request.query, request.k);

    // Validate request
    if request.query.trim().is_empty() {
        return Err(ApiError::BadRequest("Query cannot be empty".to_string()));
    }

    if request.k == 0 || request.k > 100 {
        return Err(ApiError::BadRequest("k must be between 1 and 100".to_string()));
    }

    // Perform search
    let mut search_service = state.search_service.lock().await;
    let response = search_service
        .search(request)
        .await
        .map_err(|e| {
            error!("Search error: {}", e);
            ApiError::InternalServerError("Search failed".to_string())
        })?;

    info!(
        "Search completed: {} results in {}ms",
        response.total_results, response.search_metadata.total_time_ms
    );

    Ok(Json(response))
}

/// GET /api/health - Health check endpoint
async fn health_handler(State(state): State<ApiState>) -> Result<Json<HealthResponse>, ApiError> {
    let search_service = state.search_service.lock().await;
    let health = search_service
        .health()
        .await
        .map_err(|e| {
            error!("Health check error: {}", e);
            ApiError::InternalServerError("Health check failed".to_string())
        })?;

    Ok(Json(health))
}

/// GET / - Root endpoint with API information
async fn root_handler() -> Json<Value> {
    Json(json!({
        "name": "Zero-Latency Documentation Search API",
        "version": "0.2.0",
        "status": "running",
        "endpoints": {
            "search": "POST /api/search",
            "status": "GET /api/status",
            "health": "GET /api/health",
            "list_documents": "GET /api/docs",
            "get_document": "GET /api/docs/{id}",
            "delete_document": "DELETE /api/docs/{id}",
            "reindex": "POST /api/reindex"
        },
        "docs": "https://github.com/your-org/zero-latency/tree/main/services/doc-indexer"
    }))
}

/// API error types
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = json!({
            "error": {
                "type": match status {
                    StatusCode::BAD_REQUEST => "bad_request",
                    StatusCode::NOT_FOUND => "not_found",
                    StatusCode::INTERNAL_SERVER_ERROR => "internal_server_error",
                    _ => "unknown_error",
                },
                "message": message
            }
        });

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding_provider::{MockEmbedder, EmbeddingConfig};
    use crate::vectordb_simple::VectorDB;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let vectordb = Box::new(
            VectorDB::new("mock://localhost", "test".to_string())
                .await
                .unwrap(),
        );
        let embedder = Box::new(MockEmbedder::new(EmbeddingConfig::default()));
        let search_service = SearchService::new(vectordb, embedder);
        
        ApiServer::new(search_service).router()
    }

    #[tokio::test]
    async fn test_root_endpoint() {
        let app = create_test_app().await;

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_search_endpoint_bad_request() {
        let app = create_test_app().await;

        let request_body = json!({
            "query": "",
            "k": 10
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/search")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
