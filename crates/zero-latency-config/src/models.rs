use serde::{Deserialize, Serialize};

/// Main configuration structure for Zero-Latency applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server URL for API communication
    pub server_url: String,
    
    /// Timeout for API requests in seconds
    pub timeout_seconds: u64,
    
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    
    /// Logging level (error, warn, info, debug, trace)
    pub log_level: String,
    
    /// Default output format (table, json, simple)
    pub output_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8081".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            log_level: "info".to_string(),
            output_format: "table".to_string(),
        }
    }
}
