//! Protocol Adapters
//! 
//! This module implements the protocol adapter pattern, providing thin wrappers
//! around domain services to expose them through different communication protocols.
//! Each adapter translates protocol-specific requests/responses while maintaining
//! the same underlying business logic.

pub mod rest;
pub mod jsonrpc;

pub use rest::RestAdapter;
pub use jsonrpc::JsonRpcAdapter;

use crate::application::ServiceContainer;
use std::sync::Arc;
use axum::Router;

/// Protocol adapter factory
pub struct ProtocolAdapterFactory {
    container: Arc<ServiceContainer>,
}

impl ProtocolAdapterFactory {
    pub fn new(container: Arc<ServiceContainer>) -> Self {
        Self { container }
    }
    
    /// Get the service container
    pub fn container(&self) -> Arc<ServiceContainer> {
        self.container.clone()
    }
    
    /// Create REST protocol adapter
    pub fn create_rest_adapter(&self) -> RestAdapter {
        RestAdapter::new(self.container.clone())
    }
    
    /// Create JSON-RPC protocol adapter
    pub fn create_jsonrpc_adapter(&self) -> JsonRpcAdapter {
        JsonRpcAdapter::new(self.container.clone())
    }

    /// Create combined router with both REST and JSON-RPC endpoints
    pub fn create_combined_router(&self) -> Router {
        let rest_adapter = self.create_rest_adapter();
        let jsonrpc_adapter = self.create_jsonrpc_adapter();

        // Create REST routes
        let rest_router = rest_adapter.create_router();
        
        // Create JSON-RPC routes
        let jsonrpc_router = jsonrpc_adapter.create_router();

        // Combine both routers
        rest_router.merge(jsonrpc_router)
    }
}
