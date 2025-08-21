/// JSON-RPC 2.0 implementation for tool service compliance
/// 
/// This module provides JSON-RPC 2.0 wrapper around the existing REST API handlers,
/// enabling standardized tool service interface while maintaining backward
/// compatibility with the existing REST endpoints.

pub mod server;
pub mod handlers;
pub mod types;

pub use server::create_dual_protocol_router;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response structure
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Error structure
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// Standard JSON-RPC error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    // Application-specific error codes
    pub const DOCUMENT_NOT_FOUND: i32 = -32000;
    pub const VALIDATION_ERROR: i32 = -32001;
    pub const SEARCH_ERROR: i32 = -32002;
    pub const INDEXING_ERROR: i32 = -32003;
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: error_codes::PARSE_ERROR,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: error_codes::INVALID_REQUEST,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: error_codes::METHOD_NOT_FOUND,
            message: "Method not found".to_string(),
            data: Some(serde_json::json!({
                "method": method
            })),
        }
    }

    pub fn invalid_params(details: Option<String>) -> Self {
        Self {
            code: error_codes::INVALID_PARAMS,
            message: "Invalid params".to_string(),
            data: details.map(|d| serde_json::json!({
                "details": d
            })),
        }
    }

    pub fn internal_error(message: Option<String>) -> Self {
        Self {
            code: error_codes::INTERNAL_ERROR,
            message: message.unwrap_or_else(|| "Internal error".to_string()),
            data: None,
        }
    }

    pub fn document_not_found(id: &str) -> Self {
        Self {
            code: error_codes::DOCUMENT_NOT_FOUND,
            message: "Document not found".to_string(),
            data: Some(serde_json::json!({
                "document_id": id
            })),
        }
    }

    pub fn validation_error(field: &str, message: &str) -> Self {
        Self {
            code: error_codes::VALIDATION_ERROR,
            message: "Validation error".to_string(),
            data: Some(serde_json::json!({
                "field": field,
                "details": message
            })),
        }
    }
}
