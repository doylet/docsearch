use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Stable document identifier shared across BM25 and vector stores
/// 
/// This provides a consistent identification scheme that works across
/// both tantivy BM25 indexes and vector databases, ensuring documents
/// can be uniquely identified and synchronized between storage systems.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocId {
    /// Collection namespace this document belongs to
    pub collection: String,
    
    /// External identifier (from source system)
    pub external_id: String,
    
    /// Document version for handling updates
    pub version: u64,
}

impl DocId {
    /// Create a new document ID
    pub fn new(collection: impl Into<String>, external_id: impl Into<String>, version: u64) -> Self {
        Self {
            collection: collection.into(),
            external_id: external_id.into(),
            version,
        }
    }
    
    /// Create from UUID (backward compatibility)
    pub fn from_uuid(collection: impl Into<String>, uuid: Uuid, version: u64) -> Self {
        Self::new(collection, uuid.to_string(), version)
    }
    
    /// Generate a stable string representation for indexing
    pub fn to_index_key(&self) -> String {
        format!("{}:{}:{}", self.collection, self.external_id, self.version)
    }
    
    /// Parse from index key format
    pub fn from_index_key(key: &str) -> Option<Self> {
        let parts: Vec<&str> = key.splitn(3, ':').collect();
        if parts.len() == 3 {
            if let Ok(version) = parts[2].parse::<u64>() {
                return Some(Self {
                    collection: parts[0].to_string(),
                    external_id: parts[1].to_string(),
                    version,
                });
            }
        }
        None
    }
    
    /// Get the base ID without version (for deduplication)
    pub fn base_id(&self) -> String {
        format!("{}:{}", self.collection, self.external_id)
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_index_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_id_creation() {
        let doc_id = DocId::new("docs", "file_123", 1);
        assert_eq!(doc_id.collection, "docs");
        assert_eq!(doc_id.external_id, "file_123");
        assert_eq!(doc_id.version, 1);
    }

    #[test]
    fn test_index_key_roundtrip() {
        let original = DocId::new("test_collection", "doc_456", 42);
        let key = original.to_index_key();
        let parsed = DocId::from_index_key(&key).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_base_id() {
        let doc_id = DocId::new("collection", "doc123", 5);
        assert_eq!(doc_id.base_id(), "collection:doc123");
    }
}
