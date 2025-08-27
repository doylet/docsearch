/// API Contract Constants
///
/// Shared constants between CLI and server to prevent endpoint mismatches.

/// Default configuration values
pub mod defaults {
    /// Default server port for doc-indexer service
    pub const SERVER_PORT: u16 = 8081;

    /// Default server host
    pub const SERVER_HOST: &str = "localhost";

    /// Default collection name
    pub const COLLECTION_NAME: &str = "zero_latency_docs";

    /// Default request timeout in milliseconds
    pub const REQUEST_TIMEOUT_MS: u64 = 30000;

    /// API URL prefix
    pub const API_PREFIX: &str = "/api";
}

/// API endpoint paths
pub mod endpoints {
    // Server management endpoints
    pub const STATUS: &str = "/api/status";
    pub const SERVER_START: &str = "/api/server/start";
    pub const SERVER_STOP: &str = "/api/server/stop";

    // Search endpoints
    pub const SEARCH: &str = "/api/search";

    // Indexing endpoints
    pub const INDEX: &str = "/api/index";
    pub const REINDEX: &str = "/api/reindex";

    // Collection management endpoints
    pub const COLLECTIONS: &str = "/api/collections";
    pub const COLLECTION_BY_NAME: &str = "/api/collections/:name";
    pub const COLLECTION_STATS: &str = "/api/collections/:name/stats";

    // Document endpoints
    pub const DOCUMENTS: &str = "/api/documents";
    pub const DOCUMENT_BY_ID: &str = "/api/documents/:id";
    pub const DOCUMENTS_SEARCH: &str = "/api/documents/search";

    // Analytics endpoints
    pub const ANALYTICS_SUMMARY: &str = "/api/analytics/summary";
    pub const ANALYTICS_POPULAR_QUERIES: &str = "/api/analytics/popular-queries";
    pub const ANALYTICS_SEARCH_TRENDS: &str = "/api/analytics/search-trends";

    // Helper functions for dynamic endpoints
    pub fn collection_by_name(name: &str) -> String {
        COLLECTION_BY_NAME.replace(":name", name)
    }

    pub fn collection_stats(name: &str) -> String {
        COLLECTION_STATS.replace(":name", name)
    }

    pub fn document_by_id(id: &str) -> String {
        DOCUMENT_BY_ID.replace(":id", id)
    }
}

/// URL generation utilities
pub mod urls {
    use super::defaults::{SERVER_HOST, SERVER_PORT};

    /// Generate default server URL
    pub fn default_server_url() -> String {
        format!("http://{}:{}", SERVER_HOST, SERVER_PORT)
    }

    /// Generate server URL with custom host and port
    pub fn server_url(host: &str, port: u16) -> String {
        format!("http://{}:{}", host, port)
    }

    /// Generate full endpoint URL
    pub fn endpoint_url(base_url: &str, endpoint: &str) -> String {
        format!("{}{}", base_url.trim_end_matches('/'), endpoint)
    }

    /// Generate collection by name URL
    pub fn collection_by_name(base_url: &str, name: &str) -> String {
        endpoint_url(base_url, &super::endpoints::collection_by_name(name))
    }

    /// Generate collection stats URL
    pub fn collection_stats(base_url: &str, name: &str) -> String {
        endpoint_url(base_url, &super::endpoints::collection_stats(name))
    }

    /// Generate document by ID URL
    pub fn document_by_id(base_url: &str, id: &str) -> String {
        endpoint_url(base_url, &super::endpoints::document_by_id(id))
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
            super::endpoints::collection_by_name("test"),
            "/api/collections/test"
        );
        assert_eq!(
            super::endpoints::collection_stats("test"),
            "/api/collections/test/stats"
        );
        assert_eq!(
            super::endpoints::document_by_id("123"),
            "/api/documents/123"
        );
    }

    #[test]
    fn test_url_generation() {
        assert_eq!(urls::default_server_url(), "http://localhost:8081");
        assert_eq!(urls::server_url("127.0.0.1", 9000), "http://127.0.0.1:9000");
        assert_eq!(
            urls::endpoint_url("http://localhost:8081", "/api/status"),
            "http://localhost:8081/api/status"
        );
    }
}
