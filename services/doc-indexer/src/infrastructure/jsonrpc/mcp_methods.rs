/// MCP (Model Context Protocol) specific method implementations
/// 
/// This module implements the standard MCP methods that enable integration
/// with MCP-compatible clients and tools. This will be fully implemented
/// in Phase 2 of the JSON-RPC/MCP compliance implementation.

use serde_json::Value;
use crate::infrastructure::http::handlers::AppState;
use crate::infrastructure::jsonrpc::{JsonRpcResponse, JsonRpcError};
use crate::infrastructure::jsonrpc::types::*;

/// Handle MCP tools/list method
/// Returns the list of available tools that this service provides
pub async fn handle_tools_list(
    _params: Option<Value>,
    id: Option<Value>,
    _state: &AppState,
) -> JsonRpcResponse {
    // Phase 2: Implement MCP tools/list
    // This will return the available tools like "search_documents", "index_document", etc.
    
    let tools = vec![
        Tool {
            name: "search_documents".to_string(),
            description: "Search for documents using semantic search".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "index_document".to_string(),
            description: "Index a new document for search".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "The document content to index"
                    },
                    "title": {
                        "type": "string",
                        "description": "Optional document title"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional metadata"
                    }
                },
                "required": ["content"]
            }),
        },
        Tool {
            name: "get_document".to_string(),
            description: "Retrieve a document by ID".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The document ID"
                    }
                },
                "required": ["id"]
            }),
        },
    ];
    
    let result = ListToolsResult { tools };
    JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Handle MCP tools/call method
/// Executes a specific tool with the provided arguments
pub async fn handle_tools_call(
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    match params {
        Some(params_value) => {
            match serde_json::from_value::<CallToolParams>(params_value) {
                Ok(call_params) => {
                    match call_params.name.as_str() {
                        "search_documents" => {
                            handle_tool_search_documents(call_params.arguments, id, state).await
                        }
                        "index_document" => {
                            handle_tool_index_document(call_params.arguments, id, state).await
                        }
                        "get_document" => {
                            handle_tool_get_document(call_params.arguments, id, state).await
                        }
                        _ => JsonRpcResponse::error(
                            id,
                            JsonRpcError {
                                code: -32601,
                                message: "Tool not found".to_string(),
                                data: Some(serde_json::json!({
                                    "tool_name": call_params.name
                                })),
                            },
                        ),
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

/// Tool implementation: search_documents
async fn handle_tool_search_documents(
    arguments: std::collections::HashMap<String, Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    let query = match arguments.get("query") {
        Some(Value::String(q)) => q.clone(),
        _ => {
            return JsonRpcResponse::error(
                id,
                JsonRpcError::invalid_params(Some("Missing or invalid 'query' parameter".to_string())),
            );
        }
    };

    let limit = arguments
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(10);

    // Delegate to the search handler
    let search_params = SearchDocumentsParams {
        query: query.clone(),
        limit: Some(limit),
        filters: None,
        include_content: Some(true),
    };

    match state.document_service
        .search_documents(&search_params.query, search_params.limit.unwrap_or(10))
        .await
    {
        Ok(search_response) => {
            let results_text = if search_response.results.is_empty() {
                format!("No documents found for query: '{}'", query)
            } else {
                let mut text = format!("Found {} documents for query '{}':\n\n", search_response.results.len(), query);
                for (i, result) in search_response.results.iter().enumerate() {
                    text.push_str(&format!("{}. Document ID: {}\n", i + 1, result.document_id));
                    text.push_str(&format!("   Score: {:.3}\n", result.final_score.value()));
                    text.push_str(&format!("   Content: {}\n\n", 
                        if result.content.len() > 200 {
                            format!("{}...", &result.content[..200])
                        } else {
                            result.content.clone()
                        }
                    ));
                }
                text
            };

            let result = CallToolResult {
                content: vec![ToolResponseContent {
                    r#type: "text".to_string(),
                    text: Some(results_text),
                    data: None,
                    mime_type: Some("text/plain".to_string()),
                }],
                is_error: Some(false),
            };

            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
        Err(err) => {
            let result = CallToolResult {
                content: vec![ToolResponseContent {
                    r#type: "text".to_string(),
                    text: Some(format!("Search error: {}", err)),
                    data: None,
                    mime_type: Some("text/plain".to_string()),
                }],
                is_error: Some(true),
            };
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
    }
}

/// Tool implementation: index_document
async fn handle_tool_index_document(
    arguments: std::collections::HashMap<String, Value>,
    id: Option<Value>,
    state: &AppState,
) -> JsonRpcResponse {
    let content = match arguments.get("content") {
        Some(Value::String(c)) => c.clone(),
        _ => {
            return JsonRpcResponse::error(
                id,
                JsonRpcError::invalid_params(Some("Missing or invalid 'content' parameter".to_string())),
            );
        }
    };

    let title = arguments
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Generate a UUID for the new document
    let document_id = zero_latency_core::Uuid::new_v4();

    let index_params = IndexDocumentParams {
        id: document_id.to_string(),
        title,
        content,
        path: None,
        metadata: None,
    };

    // Delegate to the index handler logic
    let document = zero_latency_core::models::Document {
        id: document_id,
        title: index_params.title.unwrap_or_else(|| "Untitled".to_string()),
        content: index_params.content,
        path: std::path::PathBuf::from("/tmp/mcp_document"),
        last_modified: chrono::Utc::now(),
        size: 0,
        metadata: zero_latency_core::models::DocumentMetadata {
            custom: std::collections::HashMap::new(),
            ..Default::default()
        },
    };

    match state.document_service.index_document(document).await {
        Ok(_) => {
            let result = CallToolResult {
                content: vec![ToolResponseContent {
                    r#type: "text".to_string(),
                    text: Some(format!("Document indexed successfully with ID: {}", document_id)),
                    data: None,
                    mime_type: Some("text/plain".to_string()),
                }],
                is_error: Some(false),
            };
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
        Err(err) => {
            let result = CallToolResult {
                content: vec![ToolResponseContent {
                    r#type: "text".to_string(),
                    text: Some(format!("Indexing error: {}", err)),
                    data: None,
                    mime_type: Some("text/plain".to_string()),
                }],
                is_error: Some(true),
            };
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
    }
}

/// Tool implementation: get_document
async fn handle_tool_get_document(
    arguments: std::collections::HashMap<String, Value>,
    id: Option<Value>,
    _state: &AppState,
) -> JsonRpcResponse {
    let doc_id = match arguments.get("id") {
        Some(Value::String(id)) => id.clone(),
        _ => {
            return JsonRpcResponse::error(
                id,
                JsonRpcError::invalid_params(Some("Missing or invalid 'id' parameter".to_string())),
            );
        }
    };

    // In a real implementation, this would retrieve from the index
    // For now, return a placeholder response
    let result = CallToolResult {
        content: vec![ToolResponseContent {
            r#type: "text".to_string(),
            text: Some(format!("Document retrieval not yet implemented for ID: {}", doc_id)),
            data: None,
            mime_type: Some("text/plain".to_string()),
        }],
        is_error: Some(false),
    };

    JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Add MCP method routing to the main router
pub async fn add_mcp_methods_to_router(
    method: &str,
    params: Option<Value>,
    id: Option<Value>,
    state: &AppState,
) -> Option<JsonRpcResponse> {
    match method {
        "tools/list" => Some(handle_tools_list(params, id, state).await),
        "tools/call" => Some(handle_tools_call(params, id, state).await),
        _ => None,
    }
}

// Note: This is Phase 2 implementation
// Additional MCP methods to be implemented:
// - initialize
// - notifications/initialized  
// - notifications/cancelled
// - completion/complete
// - logging/setLevel
// - prompts/list
// - prompts/get
// - resources/list
// - resources/read
// - resources/subscribe
// - resources/unsubscribe
