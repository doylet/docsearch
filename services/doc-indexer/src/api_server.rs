use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::search_service::{SearchService, SearchRequest, SearchResponse, HealthResponse};

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
            .route("/api/health", get(health_handler))
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
        "name": "Doc-Indexer Search API",
        "version": "0.1.0",
        "status": "running",
        "endpoints": {
            "search": "POST /api/search",
            "health": "GET /api/health"
        },
        "docs": "https://github.com/your-org/zero-latency/tree/main/services/doc-indexer"
    }))
}

/// API error types
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    InternalServerError(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = json!({
            "error": {
                "type": match status {
                    StatusCode::BAD_REQUEST => "bad_request",
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
