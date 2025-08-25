/// Request and Response Contracts
/// 
/// Shared data structures for API requests and responses to ensure
/// type safety and compatibility between CLI and server.

use serde::{Deserialize, Serialize};

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
    
    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

/// Standard API error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }
    
    pub fn with_details(code: &str, message: &str, details: serde_json::Value) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }
}

/// Status endpoint response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
    pub collections_count: usize,
    pub documents_count: usize,
}

/// Collection structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub document_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

/// Document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub collection: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Document creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub collection: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

/// Document update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub collection: Option<String>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub query: String,
    pub processing_time_ms: u64,
}

/// Collection list response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionListWrapper {
    pub collections: Vec<Collection>,
}

/// Collection list response
pub type CollectionListResponse = CollectionListWrapper;

/// Document list response
pub type DocumentListResponse = Vec<Document>;

/// Standard error codes
pub mod error_codes {
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const BAD_REQUEST: &str = "BAD_REQUEST";
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const COLLECTION_NOT_FOUND: &str = "COLLECTION_NOT_FOUND";
    pub const DOCUMENT_NOT_FOUND: &str = "DOCUMENT_NOT_FOUND";
    pub const SEARCH_ERROR: &str = "SEARCH_ERROR";
    pub const INDEX_ERROR: &str = "INDEX_ERROR";
}

/// Request validation utilities
pub mod validation {
    use super::*;
    
    pub fn validate_search_request(req: &SearchRequest) -> Result<(), ApiError> {
        if req.query.trim().is_empty() {
            return Err(ApiError::new(
                error_codes::VALIDATION_ERROR,
                "Search query cannot be empty"
            ));
        }
        
        if let Some(limit) = req.limit {
            if limit == 0 || limit > 1000 {
                return Err(ApiError::new(
                    error_codes::VALIDATION_ERROR,
                    "Search limit must be between 1 and 1000"
                ));
            }
        }
        
        if let Some(min_score) = req.min_score {
            if min_score < 0.0 || min_score > 1.0 {
                return Err(ApiError::new(
                    error_codes::VALIDATION_ERROR,
                    "Minimum score must be between 0.0 and 1.0"
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn validate_create_document_request(req: &CreateDocumentRequest) -> Result<(), ApiError> {
        if req.collection.trim().is_empty() {
            return Err(ApiError::new(
                error_codes::VALIDATION_ERROR,
                "Collection name cannot be empty"
            ));
        }
        
        if req.content.trim().is_empty() {
            return Err(ApiError::new(
                error_codes::VALIDATION_ERROR,
                "Document content cannot be empty"
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
        assert!(response.timestamp.is_some());
    }
    
    #[test]
    fn test_api_response_error() {
        let error = ApiError::new("TEST_ERROR", "Test error message");
        let response: ApiResponse<String> = ApiResponse::error(error);
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert!(response.timestamp.is_some());
    }
    
    #[test]
    fn test_search_request_validation() {
        let valid_request = SearchRequest {
            query: "test query".to_string(),
            collection: None,
            limit: Some(10),
            min_score: Some(0.5),
        };
        assert!(validation::validate_search_request(&valid_request).is_ok());
        
        let invalid_request = SearchRequest {
            query: "".to_string(),
            collection: None,
            limit: None,
            min_score: None,
        };
        assert!(validation::validate_search_request(&invalid_request).is_err());
    }
    
    #[test]
    fn test_create_document_validation() {
        let valid_request = CreateDocumentRequest {
            collection: "test_collection".to_string(),
            title: Some("Test Title".to_string()),
            content: "Test content".to_string(),
            metadata: None,
        };
        assert!(validation::validate_create_document_request(&valid_request).is_ok());
        
        let invalid_request = CreateDocumentRequest {
            collection: "".to_string(),
            title: None,
            content: "".to_string(),
            metadata: None,
        };
        assert!(validation::validate_create_document_request(&invalid_request).is_err());
    }
}
