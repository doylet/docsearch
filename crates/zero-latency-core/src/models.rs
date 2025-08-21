use crate::{DateTime, Utc, Uuid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Core document representation used across services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub path: PathBuf,
    pub last_modified: DateTime<Utc>,
    pub size: u64,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub language: Option<String>,
    pub author: Option<String>,
    pub custom: HashMap<String, String>,
}

/// Chunk of a document for vector processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub chunk_index: usize,
    pub heading_path: Vec<String>,
    pub start_offset: usize,
    pub end_offset: usize,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkMetadata {
    pub section_type: Option<String>,
    pub importance_score: Option<f32>,
    pub keywords: Vec<String>,
    pub custom: HashMap<String, String>,
}

/// Health status for service components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { message: String },
    Unhealthy { message: String },
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy { .. })
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub details: Option<HashMap<String, String>>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: usize,
    pub size: usize,
    pub total: Option<usize>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            size: 20,
            total: None,
        }
    }
}

/// Common metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub service: String,
    pub version: String,
}

impl ResponseMetadata {
    pub fn new(service: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            duration_ms: 0,
            service: service.into(),
            version: version.into(),
        }
    }
}
