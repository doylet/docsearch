/// Type definitions for JSON-RPC 2.0 tool service compliance
///
/// This module defines the request/response types for JSON-RPC wrapped methods
/// that correspond to the existing REST API endpoints.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Document Management Types

#[derive(Debug, Deserialize)]
pub struct IndexDocumentParams {
    pub id: String,
    pub title: Option<String>,
    pub content: String,
    pub path: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct IndexDocumentResult {
    pub success: bool,
    pub message: String,
    pub document_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GetDocumentParams {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct GetDocumentResult {
    pub id: String,
    pub found: bool,
    pub content: Option<String>,
    pub title: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentParams {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub path: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateDocumentResult {
    pub success: bool,
    pub message: String,
    pub document_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDocumentParams {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteDocumentResult {
    pub success: bool,
    pub message: String,
    pub document_id: String,
}

// Search Types

#[derive(Debug, Deserialize)]
pub struct SearchDocumentsParams {
    pub query: String,
    pub limit: Option<usize>,
    pub filters: Option<HashMap<String, String>>,
    pub include_content: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SearchDocumentsResult {
    pub query: String,
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub took_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub id: String,
    pub content: Option<String>,
    pub title: Option<String>,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

// Health Check Types

#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub status: String,
    pub timestamp: String,
    pub checks: HashMap<String, HealthCheckItem>,
}

#[derive(Debug, Serialize)]
pub struct HealthCheckItem {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReadinessResult {
    pub ready: bool,
    pub checks: HashMap<String, HealthCheckItem>,
}

#[derive(Debug, Serialize)]
pub struct LivenessResult {
    pub alive: bool,
    pub uptime_seconds: u64,
}

// Service Info Types

#[derive(Debug, Serialize)]
pub struct ServiceInfoResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
    pub protocol_version: String,
    pub capabilities: ServiceCapabilities,
}

#[derive(Debug, Serialize)]
pub struct ServiceCapabilities {
    pub document_indexing: bool,
    pub vector_search: bool,
    pub health_monitoring: bool,
    pub realtime_updates: bool,
}

// Error conversion utilities

impl From<zero_latency_core::ZeroLatencyError> for crate::infrastructure::jsonrpc::JsonRpcError {
    fn from(err: zero_latency_core::ZeroLatencyError) -> Self {
        use crate::infrastructure::jsonrpc::error_codes;
        use zero_latency_core::ZeroLatencyError;

        match err {
            ZeroLatencyError::Validation { field, message } => {
                crate::infrastructure::jsonrpc::JsonRpcError::validation_error(&field, &message)
            }
            ZeroLatencyError::NotFound { resource } => {
                if resource.contains("document") {
                    crate::infrastructure::jsonrpc::JsonRpcError::document_not_found(&resource)
                } else {
                    crate::infrastructure::jsonrpc::JsonRpcError {
                        code: error_codes::INTERNAL_ERROR,
                        message: format!("Resource not found: {}", resource),
                        data: None,
                    }
                }
            }
            ZeroLatencyError::Configuration { message }
            | ZeroLatencyError::Internal { message }
            | ZeroLatencyError::Database { message }
            | ZeroLatencyError::Network { message }
            | ZeroLatencyError::Serialization { message } => {
                crate::infrastructure::jsonrpc::JsonRpcError::internal_error(Some(message))
            }
            ZeroLatencyError::ExternalService { service, message } => {
                crate::infrastructure::jsonrpc::JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("External service error: {}", service),
                    data: Some(serde_json::json!({
                        "service": service,
                        "details": message
                    })),
                }
            }
            ZeroLatencyError::PermissionDenied { operation } => {
                crate::infrastructure::jsonrpc::JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("Permission denied: {}", operation),
                    data: None,
                }
            }
        }
    }
}
