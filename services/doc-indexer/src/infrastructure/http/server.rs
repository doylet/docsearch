/// HTTP server implementation using Axum
/// 
/// This module contains the HTTP server setup and configuration,
/// including middleware, CORS, and graceful shutdown handling.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::Request,
    http::{HeaderValue, Method},
    middleware::{self, Next},
    response::Response,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tokio::signal;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

use crate::application::ServiceContainer;
use super::handlers::AppState;

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub enable_cors: bool,
    pub cors_origins: Vec<String>,
}

/// HTTP server wrapper
pub struct HttpServer {
    config: ServerConfig,
    app_state: AppState,
}

impl HttpServer {
    /// Create a new HTTP server
    pub fn new(config: ServerConfig, container: Arc<ServiceContainer>) -> Self {
        let app_state = AppState::new(container);
        
        Self {
            config,
            app_state,
        }
    }
    
    /// Build the complete router with middleware
    pub fn build_router(&self) -> Router {
        // Use the dual protocol router that includes both REST and JSON-RPC endpoints
        let router = crate::infrastructure::jsonrpc::create_dual_protocol_router(self.app_state.clone());
        
        // Build middleware stack
        let middleware_stack = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_secs(self.config.timeout_seconds)))
            .layer(middleware::from_fn(request_logging_middleware));
        
        let mut app = router.layer(middleware_stack);
        
        // Add CORS if enabled
        if self.config.enable_cors {
            let cors = self.build_cors_layer();
            app = app.layer(cors);
        }
        
        app
    }
    
    /// Start the HTTP server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from((
            self.config.host.parse::<std::net::IpAddr>()?,
            self.config.port,
        ));
        
        let app = self.build_router();
        
        info!("Starting HTTP server on {}", addr);
        
        // Create the server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        // Start server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        
        info!("HTTP server stopped");
        Ok(())
    }
    
    /// Build CORS layer
    fn build_cors_layer(&self) -> CorsLayer {
        let mut cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ]);
        
        // Configure origins
        if self.config.cors_origins.is_empty() {
            cors = cors.allow_origin(axum::http::header::HeaderValue::from_static("*"));
        } else {
            let origins: Result<Vec<HeaderValue>, _> = self.config.cors_origins
                .iter()
                .map(|origin| origin.parse())
                .collect();
            
            if let Ok(origins) = origins {
                cors = cors.allow_origin(origins);
            } else {
                warn!("Invalid CORS origins configured, allowing all origins");
                cors = cors.allow_origin(axum::http::header::HeaderValue::from_static("*"));
            }
        }
        
        cors
    }
}

/// Request logging middleware
async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = std::time::Instant::now();
    
    // Process the request
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    info!(
        method = %method,
        path = %path,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "HTTP request processed"
    );
    
    response
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM, starting graceful shutdown");
        },
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            timeout_seconds: 300, // 5 minutes for large indexing operations
            enable_cors: true,
            cors_origins: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    
    
    
    #[tokio::test]
    async fn test_router_creation() {
        // Create test configuration and container
        let config = Config::default();
        let container = Arc::new(
            ServiceContainer::new(config).await.unwrap_or_else(|_| {
                // Use a mock container for testing
                panic!("Failed to create container for test")
            })
        );
        
        let server_config = ServerConfig::default();
        let server = HttpServer::new(server_config, container);
        
        // Should be able to build router without panicking
        let _router = server.build_router();
    }
    
    #[tokio::test]
    async fn test_cors_layer_creation() {
        let config = Config::default();
        let container = Arc::new(
            ServiceContainer::new(config).await.unwrap_or_else(|_| {
                panic!("Failed to create container for test")
            })
        );
        
        let server_config = ServerConfig {
            enable_cors: true,
            cors_origins: vec!["http://localhost:3000".to_string()],
            ..Default::default()
        };
        
        let server = HttpServer::new(server_config, container);
        let _cors_layer = server.build_cors_layer();
    }
}
