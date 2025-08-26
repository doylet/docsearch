/// JSON-RPC 2.0 server implementation
///
/// This module provides an HTTP server that handles JSON-RPC 2.0 requests,
/// enabling standardized tool service interface while maintaining compatibility with
/// the existing REST API through dual endpoints.
use axum::{extract::State, response::Json, routing::post, Router};
use serde_json::Value;

use crate::infrastructure::http::handlers::AppState;
use crate::infrastructure::jsonrpc::handlers::route_method;
use crate::infrastructure::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

/// JSON-RPC server state (wraps the existing AppState)
#[derive(Clone)]
pub struct JsonRpcServer {
    pub app_state: AppState,
}

impl JsonRpcServer {
    /// Create a new JSON-RPC server with the given application state
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Create the JSON-RPC router with all endpoints
    pub fn create_router(self) -> Router {
        Router::new()
            // Main JSON-RPC endpoint
            .route("/jsonrpc", post(handle_jsonrpc_request))
            // Batch endpoint for multiple requests
            .route("/jsonrpc/batch", post(handle_batch_jsonrpc_request))
            .with_state(self)
    }
}

/// Handle single JSON-RPC request
async fn handle_jsonrpc_request(
    State(server): State<JsonRpcServer>,
    Json(payload): Json<Value>,
) -> Json<JsonRpcResponse> {
    // Parse the JSON-RPC request
    let request = match serde_json::from_value::<JsonRpcRequest>(payload) {
        Ok(req) => req,
        Err(_) => {
            return Json(JsonRpcResponse::error(None, JsonRpcError::parse_error()));
        }
    };

    // Validate JSON-RPC version
    if request.jsonrpc != "2.0" {
        return Json(JsonRpcResponse::error(
            request.id,
            JsonRpcError::invalid_request(),
        ));
    }

    // Route the method call
    let response = route_method(
        &request.method,
        request.params,
        request.id,
        &server.app_state,
    )
    .await;

    Json(response)
}

/// Handle batch JSON-RPC requests
async fn handle_batch_jsonrpc_request(
    State(server): State<JsonRpcServer>,
    Json(payload): Json<Value>,
) -> Json<Vec<JsonRpcResponse>> {
    // Parse batch request
    let requests = match payload {
        Value::Array(requests) => requests,
        _ => {
            return Json(vec![JsonRpcResponse::error(
                None,
                JsonRpcError::invalid_request(),
            )]);
        }
    };

    if requests.is_empty() {
        return Json(vec![JsonRpcResponse::error(
            None,
            JsonRpcError::invalid_request(),
        )]);
    }

    let mut responses = Vec::new();

    for request_value in requests {
        let request = match serde_json::from_value::<JsonRpcRequest>(request_value) {
            Ok(req) => req,
            Err(_) => {
                responses.push(JsonRpcResponse::error(None, JsonRpcError::parse_error()));
                continue;
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            responses.push(JsonRpcResponse::error(
                request.id,
                JsonRpcError::invalid_request(),
            ));
            continue;
        }

        // Route the method call
        let response = route_method(
            &request.method,
            request.params,
            request.id,
            &server.app_state,
        )
        .await;

        responses.push(response);
    }

    Json(responses)
}

/// Create a combined router that includes REST, JSON-RPC, and streaming endpoints
pub fn create_dual_protocol_router(app_state: AppState) -> Router {
    let rest_router = crate::infrastructure::http::handlers::create_router(app_state.clone());
    let jsonrpc_server = JsonRpcServer::new(app_state.clone());
    let jsonrpc_router = jsonrpc_server.create_router();
    let streaming_router =
        crate::infrastructure::streaming::create_streaming_router().with_state(app_state);

    // Combine all routers
    rest_router.merge(jsonrpc_router).merge(streaming_router)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper function to create test app state
    fn create_test_app_state() -> AppState {
        // This would be replaced with proper test setup
        unimplemented!("Test setup needed")
    }

    #[tokio::test]
    async fn test_valid_jsonrpc_request() {
        let _request = json!({
            "jsonrpc": "2.0",
            "method": "service.info",
            "id": 1
        });

        // Test would validate that the request is processed correctly
        // This is a placeholder for proper integration tests
    }

    #[tokio::test]
    async fn test_invalid_jsonrpc_version() {
        let request = json!({
            "jsonrpc": "1.0",
            "method": "service.info",
            "id": 1
        });

        // Test would validate that invalid version returns appropriate error
    }

    #[tokio::test]
    async fn test_batch_request() {
        let _batch_request = json!([
            {
                "jsonrpc": "2.0",
                "method": "service.info",
                "id": 1
            },
            {
                "jsonrpc": "2.0",
                "method": "health.check",
                "id": 2
            }
        ]);

        // Test would validate batch processing
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let _request = json!({
            "jsonrpc": "2.0",
            "method": "nonexistent.method",
            "id": 1
        });

        // Test would validate proper error response for unknown methods
    }
}
