/// JSON-RPC 2.0 handlers that wrap existing REST API functionality
/// 
/// This module provides JSON-RPC method handlers that delegate to the existing
/// application services, maintaining the same business logic while adding 
/// JSON-RPC 2.0 protocol compliance.

use serde_json::Value;
use zero_latency_core::models::Document;

use crate::infrastructure::http::handlers::AppState;
use crate::infrastructure::jsonrpc::{JsonRpcError, JsonRpcResponse};
use crate::infrastructure::jsonrpc::types::*;

/// JSON-RPC method handler for document indexing
/// Method: "document.index"
pub async fn handle_index_document(
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match params {
        Some(params_value) => {
            match serde_json::from_value::<IndexDocumentParams>(params_value) {
                Ok(params) => {
                    // Create document from parameters
                    let document_id = match zero_latency_core::Uuid::parse_str(&params.id) {
                        Ok(uuid) => uuid,
                        Err(_) => {
                            return JsonRpcResponse::error(
                                id,
                                JsonRpcError::validation_error("id", "Invalid UUID format"),
                            );
                        }
                    };

                    let document = Document {
                        id: document_id,
                        title: params.title.unwrap_or_else(|| "Untitled".to_string()),
                        content: params.content,
                        path: std::path::PathBuf::from(
                            params.path.unwrap_or_else(|| "/tmp/unknown".to_string())
                        ),
                        last_modified: chrono::Utc::now(),
                        size: 0, // Would be calculated in real implementation
                        metadata: zero_latency_core::models::DocumentMetadata {
                            custom: params.metadata.unwrap_or_default(),
                            ..Default::default()
                        },
                    };

                    // Delegate to application service
                    match state.document_service.index_document(document).await {
                        Ok(_) => {
                            let result = IndexDocumentResult {
                                success: true,
                                message: "Document indexed successfully".to_string(),
                                document_id: params.id,
                            };
                            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
                        }
                        Err(err) => JsonRpcResponse::error(id, err.into()),
                    }
                }
                Err(err) => JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(Some(err.to_string())),
                ),
            }
        }
        None => JsonRpcResponse::error(id, JsonRpcError::invalid_params(None)),
    }
}

/// JSON-RPC method handler for getting a document
/// Method: "document.get"
pub async fn handle_get_document(
    params: Option<Value>,
    id: Option<Value>,
    _state: &AppState,
) -> JsonRpcResponse {
    match params {
        Some(params_value) => {
            match serde_json::from_value::<GetDocumentParams>(params_value) {
                Ok(params) => {
                    // In a real implementation, this would retrieve from the index
                    // For now, return a placeholder indicating not found
                    let result = GetDocumentResult {
                        id: params.id,
                        found: false,
                        content: None,
                        title: None,
                        metadata: None,
                    };
                    JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
                }
                Err(err) => JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(Some(err.to_string())),
                ),
            }
        }
        None => JsonRpcResponse::error(id, JsonRpcError::invalid_params(None)),
    }
}

/// JSON-RPC method handler for updating a document
/// Method: "document.update"
pub async fn handle_update_document(
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match params {
        Some(params_value) => {
            match serde_json::from_value::<UpdateDocumentParams>(params_value) {
                Ok(params) => {
                    let document_id = match zero_latency_core::Uuid::parse_str(&params.id) {
                        Ok(uuid) => uuid,
                        Err(_) => {
                            return JsonRpcResponse::error(
                                id,
                                JsonRpcError::validation_error("id", "Invalid UUID format"),
                            );
                        }
                    };

                    let document = Document {
                        id: document_id,
                        title: params.title.unwrap_or_else(|| "Untitled".to_string()),
                        content: params.content.unwrap_or_default(),
                        path: std::path::PathBuf::from(
                            params.path.unwrap_or_else(|| "/tmp/unknown".to_string())
                        ),
                        last_modified: chrono::Utc::now(),
                        size: 0,
                        metadata: zero_latency_core::models::DocumentMetadata {
                            custom: params.metadata.unwrap_or_default(),
                            ..Default::default()
                        },
                    };

                    match state.document_service.update_document(document).await {
                        Ok(_) => {
                            let result = UpdateDocumentResult {
                                success: true,
                                message: "Document updated successfully".to_string(),
                                document_id: params.id,
                            };
                            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
                        }
                        Err(err) => JsonRpcResponse::error(id, err.into()),
                    }
                }
                Err(err) => JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(Some(err.to_string())),
                ),
            }
        }
        None => JsonRpcResponse::error(id, JsonRpcError::invalid_params(None)),
    }
}

/// JSON-RPC method handler for deleting a document
/// Method: "document.delete"
pub async fn handle_delete_document(
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match params {
        Some(params_value) => {
            match serde_json::from_value::<DeleteDocumentParams>(params_value) {
                Ok(params) => {
                    match state.document_service.delete_document(&params.id).await {
                        Ok(_) => {
                            let result = DeleteDocumentResult {
                                success: true,
                                message: "Document deleted successfully".to_string(),
                                document_id: params.id,
                            };
                            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
                        }
                        Err(err) => JsonRpcResponse::error(id, err.into()),
                    }
                }
                Err(err) => JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(Some(err.to_string())),
                ),
            }
        }
        None => JsonRpcResponse::error(id, JsonRpcError::invalid_params(None)),
    }
}

/// JSON-RPC method handler for searching documents
/// Method: "document.search"
pub async fn handle_search_documents(
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match params {
        Some(params_value) => {
            match serde_json::from_value::<SearchDocumentsParams>(params_value) {
                Ok(params) => {
                    let start_time = std::time::Instant::now();
                    
                    match state.document_service
                        .search_documents(&params.query, params.limit.unwrap_or(10))
                        .await
                    {
                        Ok(search_response) => {
                            let took_ms = start_time.elapsed().as_millis() as u64;
                            
                            let results: Vec<SearchResultItem> = search_response.results
                                .into_iter()
                                .map(|result| SearchResultItem {
                                    id: result.document_id.to_string(),
                                    content: if params.include_content.unwrap_or(true) {
                                        Some(result.content)
                                    } else {
                                        None
                                    },
                                    title: None, // Not available in SearchResult yet
                                    score: result.final_score.value(),
                                    metadata: std::collections::HashMap::new(),
                                })
                                .collect();

                            let result = SearchDocumentsResult {
                                query: params.query,
                                total: results.len(),
                                results,
                                took_ms: Some(took_ms),
                            };
                            
                            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
                        }
                        Err(err) => JsonRpcResponse::error(id, err.into()),
                    }
                }
                Err(err) => JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(Some(err.to_string())),
                ),
            }
        }
        None => JsonRpcResponse::error(id, JsonRpcError::invalid_params(None)),
    }
}

/// JSON-RPC method handler for health check
/// Method: "health.check"
pub async fn handle_health_check(
    _params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match state.health_service.health_check().await {
        Ok(health) => {
            let result = HealthCheckResult {
                status: format!("{:?}", health.status),
                timestamp: health.timestamp.to_rfc3339(),
                checks: health.checks.into_iter().map(|check| {
                    (check.component, HealthCheckItem {
                        status: format!("{:?}", check.status),
                        message: check.message,
                    })
                }).collect(),
            };
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
        Err(err) => JsonRpcResponse::error(id, err.into()),
    }
}

/// JSON-RPC method handler for readiness check
/// Method: "health.ready"
pub async fn handle_readiness_check(
    _params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match state.health_service.readiness_check().await {
        Ok(readiness) => {
            let result = ReadinessResult {
                ready: readiness.ready,
                checks: readiness.checks.into_iter().map(|check| {
                    (check.component, HealthCheckItem {
                        status: format!("{:?}", check.status),
                        message: check.message,
                    })
                }).collect(),
            };
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
        Err(err) => JsonRpcResponse::error(id, err.into()),
    }
}

/// JSON-RPC method handler for liveness check
/// Method: "health.live"
pub async fn handle_liveness_check(
    _params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match state.health_service.liveness_check().await {
        Ok(liveness) => {
            let result = LivenessResult {
                alive: liveness.alive,
                uptime_seconds: liveness.uptime.as_secs(),
            };
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
        Err(err) => JsonRpcResponse::error(id, err.into()),
    }
}

/// JSON-RPC method handler for service information
/// Method: "service.info"
pub async fn handle_service_info(
    _params: Option<Value>,
    id: Option<Value>,
    _state: &AppState,
) -> JsonRpcResponse {
    let result = ServiceInfoResult {
        name: "doc-indexer".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Document indexing and search service with JSON-RPC 2.0 and MCP support".to_string(),
        features: vec![
            "document_indexing".to_string(),
            "vector_search".to_string(),
            "health_monitoring".to_string(),
            "json_rpc".to_string(),
        ],
        protocol_version: "2.0".to_string(),
        capabilities: ServiceCapabilities {
            document_indexing: true,
            vector_search: true,
            health_monitoring: true,
            mcp_tools: false, // Will be enabled in Phase 2
            realtime_updates: false, // Future enhancement
        },
    };
    
    JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Route JSON-RPC method calls to appropriate handlers
pub async fn route_method(
    method: &str,
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    // Check for MCP methods first
    if let Some(response) = crate::infrastructure::jsonrpc::mcp_methods::add_mcp_methods_to_router(method, params.clone(), id.clone(), state).await {
        return response;
    }
    
    // Handle standard JSON-RPC methods
    match method {
        // Document methods
        "document.index" => handle_index_document(params, id, state).await,
        "document.get" => handle_get_document(params, id, state).await,
        "document.update" => handle_update_document(params, id, state).await,
        "document.delete" => handle_delete_document(params, id, state).await,
        "document.search" => handle_search_documents(params, id, state).await,
        
        // Health methods
        "health.check" => handle_health_check(params, id, state).await,
        "health.ready" => handle_readiness_check(params, id, state).await,
        "health.live" => handle_liveness_check(params, id, state).await,
        
        // Service methods
        "service.info" => handle_service_info(params, id, state).await,
        
        // Unknown method
        _ => JsonRpcResponse::error(id, JsonRpcError::method_not_found(method)),
    }
}
