use thiserror::Error;

/// Common error types used across Zero-Latency services
#[derive(Error, Debug)]
pub enum ZeroLatencyError {
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("External service error: {service}: {message}")]
    ExternalService { service: String, message: String },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl ZeroLatencyError {
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    pub fn permission_denied(operation: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn search(message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: "search".to_string(),
            message: message.into(),
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }
}

/// Common Result type used across Zero-Latency services
pub type Result<T> = std::result::Result<T, ZeroLatencyError>;

/// Convert common error types
impl From<serde_json::Error> for ZeroLatencyError {
    fn from(error: serde_json::Error) -> Self {
        Self::serialization(error.to_string())
    }
}

impl From<std::io::Error> for ZeroLatencyError {
    fn from(error: std::io::Error) -> Self {
        Self::network(error.to_string())
    }
}

impl From<String> for ZeroLatencyError {
    fn from(error: String) -> Self {
        Self::internal(error)
    }
}
