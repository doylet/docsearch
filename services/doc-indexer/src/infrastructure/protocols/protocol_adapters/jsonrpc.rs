//! JSON-RPC Protocol Adapter
//! 
//! Implements the JSON-RPC 2.0 protocol adapter using generated types from the OpenAPI specification.
//! This adapter provides MCP-compliant JSON-RPC methods that delegate to domain services.

use crate::application::ServiceContainer;
use crate::infrastructure::api::jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use zero_latency_api::types::*;
use zero_latency_core::ZeroLatencyError;

/// JSON-RPC protocol adapter
#[derive(Clone)]
pub struct JsonRpcAdapter {
    container: Arc<ServiceContainer>,
}

impl JsonRpcAdapter {
    /// Create new JSON-RPC adapter
    pub fn new(container: Arc<ServiceContainer>) -> Self {
        Self { container }
    }
    
    /// Create the JSON-RPC router
    pub fn create_router(self) -> Router {
        Router::new()
            .route("/jsonrpc", post(Self::handle_jsonrpc_request))
            .route("/mcp", post(Self::handle_jsonrpc_request))  // MCP alias
            .route("/jsonrpc/batch", post(Self::handle_batch_request))
            .with_state(self)
    }
    
    /// Handle individual JSON-RPC request
    #[instrument(skip(state, request))]
    async fn handle_jsonrpc_request(
        State(state): State<JsonRpcAdapter>,
        Json(request): Json<JsonRpcRequest>,
    ) -> impl IntoResponse {
        debug!("Processing JSON-RPC request: {}", request.method);
        
        let response = state.process_request(request).await;
        Json(response)
    }
    
    /// Handle batch JSON-RPC requests
    #[instrument(skip(state, requests))]
    async fn handle_batch_request(
        State(state): State<JsonRpcAdapter>,
        Json(requests): Json<Vec<JsonRpcRequest>>,
    ) -> impl IntoResponse {
        debug!("Processing JSON-RPC batch request with {} items", requests.len());
        
        let mut responses = Vec::new();
        for request in requests {
            let response = state.process_request(request).await;
            responses.push(response);
        }
        
        Json(responses)
    }
    
    /// Process a single JSON-RPC request
    async fn process_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse::error(
                request.id,
                JsonRpcError {
                    code: -32600, // Invalid Request
                    message: "Invalid JSON-RPC version".to_string(),
                    data: Some(json!({"required": "2.0", "received": request.jsonrpc})),
                },
            );
        }
        
        match request.method.as_str() {
            // Health endpoints
            "health.check" => self.health_check(request).await,
            "health.ready" => self.readiness_check(request).await,
            "health.live" => self.liveness_check(request).await,
            
            // Service info
            "service.info" => self.service_info(request).await,
            
            // Document management
            "document.search" => self.search_documents(request).await,
            "document.index" => self.index_document(request).await,
            "document.get" => self.get_document(request).await,
            
            // Collection management
            "collection.list" => self.list_collections(request).await,
            "collection.create" => self.create_collection(request).await,
            "collection.get" => self.get_collection(request).await,
            "collection.delete" => self.delete_collection(request).await,
            "collection.stats" => self.get_collection_stats(request).await,
            
            // MCP Tools interface
            "tools/list" => self.list_tools(request).await,
            "tools/call" => self.call_tool(request).await,
            
            // Unknown method
            _ => {
                warn!("Unknown JSON-RPC method: {}", request.method);
                JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32601, // Method not found
                        message: format!("Method '{}' not found", request.method),
                        data: None,
                    },
                )
            }
        }
    }
    
    /// Convert domain error to JSON-RPC error
    fn map_error(error: ZeroLatencyError) -> JsonRpcError {
        JsonRpcError {
            code: -32000, // Server error
            message: "Internal server error".to_string(),
            data: Some(json!({
                "error": error.to_string(),
                "type": "ZeroLatencyError"
            })),
        }
    }
}

// Health endpoints
impl JsonRpcAdapter {
    async fn health_check(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match self.container.health_service.check_health().await {
            Ok(result) => {
                let response = json!({
                    "status": result.status,
                    "timestamp": result.timestamp,
                    "version": result.version
                });
                JsonRpcResponse::success(request.id, response)
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn readiness_check(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match self.container.health_service.check_readiness().await {
            Ok(result) => {
                let response = json!({
                    "status": result.status,
                    "timestamp": result.timestamp,
                    "version": result.version
                });
                JsonRpcResponse::success(request.id, response)
            }
            Err(e) => {
                error!("Readiness check failed: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn liveness_check(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match self.container.health_service.check_liveness().await {
            Ok(result) => {
                let response = json!({
                    "status": result.status,
                    "timestamp": result.timestamp,
                    "version": result.version
                });
                JsonRpcResponse::success(request.id, response)
            }
            Err(e) => {
                error!("Liveness check failed: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn service_info(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let info = json!({
            "name": "zero-latency-doc-indexer",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Zero-Latency Document Indexing and Search Service",
            "protocols": ["REST", "JSON-RPC", "MCP"],
            "capabilities": {
                "document_indexing": true,
                "semantic_search": true,
                "collection_management": true,
                "multi_tenant": true
            }
        });
        
        JsonRpcResponse::success(request.id, info)
    }
}

// Document management
impl JsonRpcAdapter {
    async fn search_documents(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Parse search request from params
        let search_request: zero_latency_api::SearchRequest = match request.params {
            Some(params) => match serde_json::from_value(params) {
                Ok(req) => req,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError {
                            code: -32602, // Invalid params
                            message: "Invalid search parameters".to_string(),
                            data: Some(json!({"error": e.to_string()})),
                        },
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32602, // Invalid params
                        message: "Search parameters required".to_string(),
                        data: None,
                    },
                );
            }
        };
        
        // Convert generated API request to domain request
        let domain_request = zero_latency_search::SearchRequest {
            query: search_request.query.clone(),
            limit: search_request.limit.map(|l| l as usize).unwrap_or(10),
            offset: search_request.offset.map(|o| o as usize).unwrap_or(0),
        };
        
        match self.container.document_service.search(domain_request).await {
            Ok(results) => {
                let response = json!({
                    "results": results.results.into_iter().map(|r| json!({
                        "id": r.id,
                        "title": r.title,
                        "content": r.content,
                        "score": r.score
                    })).collect::<Vec<_>>(),
                    "total": results.total,
                    "query_time_ms": results.query_time_ms,
                    "query": search_request.query
                });
                
                info!("Search completed successfully, found {} results", results.total);
                JsonRpcResponse::success(request.id, response)
            }
            Err(e) => {
                error!("Search failed: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn index_document(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Parse index request from params
        let index_request: IndexRequest = match request.params {
            Some(params) => match serde_json::from_value(params) {
                Ok(req) => req,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError {
                            code: -32602, // Invalid params
                            message: "Invalid index parameters".to_string(),
                            data: Some(json!({"error": e.to_string()})),
                        },
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32602, // Invalid params
                        message: "Index parameters required".to_string(),
                        data: None,
                    },
                );
            }
        };
        
        // Convert to domain request
        let domain_request = crate::application::IndexRequest {
            path: index_request.path.clone(),
            collection: index_request.collection.clone(),
            recursive: index_request.recursive.unwrap_or(false),
        };
        
        match self.container.document_service.index_documents(domain_request).await {
            Ok(result) => {
                let response = json!({
                    "indexed_documents": result.indexed_documents,
                    "errors": result.errors,
                    "path": index_request.path
                });
                
                info!("Indexing completed, {} documents indexed", result.indexed_documents);
                JsonRpcResponse::success(request.id, response)
            }
            Err(e) => {
                error!("Indexing failed: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn get_document(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // TODO: Implement document retrieval
        JsonRpcResponse::error(
            request.id,
            JsonRpcError {
                code: -32601, // Method not found
                message: "Document retrieval not yet implemented".to_string(),
                data: None,
            },
        )
    }
}

// Collection management
impl JsonRpcAdapter {
    async fn list_collections(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match self.container.collection_service.list_collections().await {
            Ok(collections) => {
                let response = json!({
                    "collections": collections.into_iter().map(|c| json!({
                        "name": c.name,
                        "description": c.description,
                        "document_count": c.document_count,
                        "created_at": c.created_at
                    })).collect::<Vec<_>>()
                });
                
                JsonRpcResponse::success(request.id, response)
            }
            Err(e) => {
                error!("Failed to list collections: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn create_collection(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Parse collection request from params
        let collection: Collection = match request.params {
            Some(params) => match serde_json::from_value(params) {
                Ok(req) => req,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError {
                            code: -32602, // Invalid params
                            message: "Invalid collection parameters".to_string(),
                            data: Some(json!({"error": e.to_string()})),
                        },
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32602, // Invalid params
                        message: "Collection parameters required".to_string(),
                        data: None,
                    },
                );
            }
        };
        
        let domain_request = crate::application::CreateCollectionRequest {
            name: collection.name.clone(),
            description: collection.description.clone(),
        };
        
        match self.container.collection_service.create_collection(domain_request).await {
            Ok(result) => {
                let response = json!({
                    "name": result.name,
                    "description": result.description,
                    "document_count": result.document_count,
                    "created_at": result.created_at
                });
                
                info!("Created collection: {}", result.name);
                JsonRpcResponse::success(request.id, response)
            }
            Err(e) => {
                error!("Failed to create collection: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn get_collection(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Parse collection name from params
        let name: String = match request.params {
            Some(params) => {
                if let Some(name_val) = params.get("name") {
                    match serde_json::from_value(name_val.clone()) {
                        Ok(name) => name,
                        Err(_) => {
                            return JsonRpcResponse::error(
                                request.id,
                                JsonRpcError {
                                    code: -32602, // Invalid params
                                    message: "Collection name must be a string".to_string(),
                                    data: None,
                                },
                            );
                        }
                    }
                } else {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError {
                            code: -32602, // Invalid params
                            message: "Collection name required".to_string(),
                            data: None,
                        },
                    );
                }
            }
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32602, // Invalid params
                        message: "Parameters required".to_string(),
                        data: None,
                    },
                );
            }
        };
        
        match self.container.collection_service.get_collection(&name).await {
            Ok(Some(collection)) => {
                let response = json!({
                    "name": collection.name,
                    "description": collection.description,
                    "document_count": collection.document_count,
                    "created_at": collection.created_at
                });
                
                JsonRpcResponse::success(request.id, response)
            }
            Ok(None) => {
                JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32000, // Server error
                        message: "Collection not found".to_string(),
                        data: Some(json!({"collection": name})),
                    },
                )
            }
            Err(e) => {
                error!("Failed to get collection: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn delete_collection(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Parse collection name from params
        let name: String = match request.params {
            Some(params) => {
                if let Some(name_val) = params.get("name") {
                    match serde_json::from_value(name_val.clone()) {
                        Ok(name) => name,
                        Err(_) => {
                            return JsonRpcResponse::error(
                                request.id,
                                JsonRpcError {
                                    code: -32602, // Invalid params
                                    message: "Collection name must be a string".to_string(),
                                    data: None,
                                },
                            );
                        }
                    }
                } else {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError {
                            code: -32602, // Invalid params
                            message: "Collection name required".to_string(),
                            data: None,
                        },
                    );
                }
            }
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32602, // Invalid params
                        message: "Parameters required".to_string(),
                        data: None,
                    },
                );
            }
        };
        
        match self.container.collection_service.delete_collection(&name).await {
            Ok(true) => {
                let response = json!({
                    "deleted": true,
                    "collection": name
                });
                
                info!("Deleted collection: {}", name);
                JsonRpcResponse::success(request.id, response)
            }
            Ok(false) => {
                JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32000, // Server error
                        message: "Collection not found".to_string(),
                        data: Some(json!({"collection": name})),
                    },
                )
            }
            Err(e) => {
                error!("Failed to delete collection: {}", e);
                JsonRpcResponse::error(request.id, Self::map_error(e))
            }
        }
    }
    
    async fn get_collection_stats(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // NOTE: Collection stats implementation pending - needs metrics infrastructure integration
        JsonRpcResponse::error(
            request.id,
            JsonRpcError {
                code: -32601, // Method not found
                message: "Collection stats not yet implemented".to_string(),
                data: None,
            },
        )
    }
}

// MCP Tools interface
impl JsonRpcAdapter {
    async fn list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = json!({
            "tools": [
                {
                    "name": "search_documents",
                    "description": "Search for documents using semantic search",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results"
                            },
                            "offset": {
                                "type": "integer", 
                                "description": "Offset for pagination"
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "index_document",
                    "description": "Index documents from a path",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to index"
                            },
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "recursive": {
                                "type": "boolean",
                                "description": "Index recursively"
                            }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "list_collections",
                    "description": "List all available collections",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }
            ]
        });
        
        JsonRpcResponse::success(request.id, tools)
    }
    
    async fn call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Parse tool call from params
        let tool_call = match request.params {
            Some(params) => params,
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32602, // Invalid params
                        message: "Tool call parameters required".to_string(),
                        data: None,
                    },
                );
            }
        };
        
        let tool_name = match tool_call.get("name") {
            Some(name) => match name.as_str() {
                Some(s) => s,
                None => {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError {
                            code: -32602, // Invalid params
                            message: "Tool name must be a string".to_string(),
                            data: None,
                        },
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    JsonRpcError {
                        code: -32602, // Invalid params
                        message: "Tool name required".to_string(),
                        data: None,
                    },
                );
            }
        };
        
        let tool_args = tool_call.get("arguments").unwrap_or(&json!({}));
        
        // Create a new JSON-RPC request for the tool
        let tool_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: match tool_name {
                "search_documents" => "document.search".to_string(),
                "index_document" => "document.index".to_string(),
                "list_collections" => "collection.list".to_string(),
                _ => {
                    return JsonRpcResponse::error(
                        request.id,
                        JsonRpcError {
                            code: -32601, // Method not found
                            message: format!("Unknown tool: {}", tool_name),
                            data: None,
                        },
                    );
                }
            },
            params: Some(tool_args.clone()),
            id: request.id.clone(),
        };
        
        // Execute the tool by delegating to the appropriate method
        self.process_request(tool_request).await
    }
}
