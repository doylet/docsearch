//! API endpoint constants generated from OpenAPI specification
//! 
//! This module provides endpoint constants that match the OpenAPI specification
//! and replaces the manual constants from zero-latency-contracts.

/// API endpoint constants
pub mod endpoints {
    // Health endpoints
    pub const HEALTH: &str = "/health";
    pub const HEALTH_READY: &str = "/health/ready";
    pub const HEALTH_LIVE: &str = "/health/live";
    
    // API status
    pub const STATUS: &str = "/api/status";
    
    // Search endpoints
    pub const SEARCH: &str = "/api/search";
    pub const DOCUMENTS_SEARCH: &str = "/api/documents/search";
    
    // Indexing endpoints
    pub const INDEX: &str = "/api/index";
    pub const REINDEX: &str = "/api/reindex";
    
    // Collection management endpoints
    pub const COLLECTIONS: &str = "/api/collections";
    pub const COLLECTION_BY_NAME: &str = "/api/collections/{name}";
    pub const COLLECTION_STATS: &str = "/api/collections/{name}/stats";
    
    // Document endpoints
    pub const DOCUMENTS: &str = "/api/documents";
    pub const DOCUMENT_BY_ID: &str = "/api/documents/{id}";
    
    // Server management
    pub const SERVER_START: &str = "/api/server/start";
    pub const SERVER_STOP: &str = "/api/server/stop";
    
    // Analytics endpoints
    pub const ANALYTICS_SUMMARY: &str = "/api/analytics/summary";
    pub const ANALYTICS_POPULAR_QUERIES: &str = "/api/analytics/popular-queries";
    pub const ANALYTICS_SEARCH_TRENDS: &str = "/api/analytics/search-trends";
    
    /// Helper functions for dynamic endpoints
    pub fn collection_by_name(name: &str) -> String {
        COLLECTION_BY_NAME.replace("{name}", name)
    }
    
    pub fn collection_stats(name: &str) -> String {
        COLLECTION_STATS.replace("{name}", name)
    }
    
    pub fn document_by_id(id: &str) -> String {
        DOCUMENT_BY_ID.replace("{id}", id)
    }
}

/// URL generation utilities
pub mod urls {
    use super::endpoints;
    
    /// Generate collection by name URL
    pub fn collection_by_name(base_url: &str, name: &str) -> String {
        format!("{}{}", base_url.trim_end_matches('/'), &endpoints::collection_by_name(name))
    }
    
    /// Generate collection stats URL
    pub fn collection_stats(base_url: &str, name: &str) -> String {
        format!("{}{}", base_url.trim_end_matches('/'), &endpoints::collection_stats(name))
    }
    
    /// Generate document by ID URL
    pub fn document_by_id(base_url: &str, id: &str) -> String {
        format!("{}{}", base_url.trim_end_matches('/'), &endpoints::document_by_id(id))
    }
    
    /// Generate full endpoint URL
    pub fn endpoint_url(base_url: &str, endpoint: &str) -> String {
        format!("{}{}", base_url.trim_end_matches('/'), endpoint)
    }
}

/// Configuration defaults from OpenAPI specification
pub mod defaults {
    /// Default server port for doc-indexer service
    pub const SERVER_PORT: u16 = 8081;
    
    /// Default server host
    pub const SERVER_HOST: &str = "localhost";
    
    /// Default collection name
    pub const COLLECTION_NAME: &str = "zero_latency_docs";
    
    /// Default request timeout in milliseconds
    pub const REQUEST_TIMEOUT_MS: u64 = 30000;
    
    /// Generate default server URL
    pub fn default_server_url() -> String {
        format!("http://{}:{}", SERVER_HOST, SERVER_PORT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_endpoint_constants() {
        assert_eq!(endpoints::STATUS, "/api/status");
        assert_eq!(endpoints::COLLECTIONS, "/api/collections");
        assert_eq!(endpoints::SEARCH, "/api/search");
    }
    
    #[test]
    fn test_dynamic_endpoints() {
        assert_eq!(
            endpoints::collection_by_name("test"),
            "/api/collections/test"
        );
        assert_eq!(
            endpoints::collection_stats("test"),
            "/api/collections/test/stats"
        );
        assert_eq!(
            endpoints::document_by_id("123"),
            "/api/documents/123"
        );
    }
    
    #[test]
    fn test_url_generation() {
        assert_eq!(
            urls::collection_by_name("http://localhost:8081", "test"),
            "http://localhost:8081/api/collections/test"
        );
        assert_eq!(
            urls::endpoint_url("http://localhost:8081", "/api/status"),
            "http://localhost:8081/api/status"
        );
    }
    
    #[test]
    fn test_defaults() {
        assert_eq!(defaults::default_server_url(), "http://localhost:8081");
        assert_eq!(defaults::SERVER_PORT, 8081);
    }
}
