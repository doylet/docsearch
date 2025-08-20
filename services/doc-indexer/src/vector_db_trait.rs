use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::document::Document;

/// Trait for vector database operations - allows both mock and real implementations
#[async_trait]
pub trait VectorDatabase: Send + Sync {
    async fn ensure_collection_exists(&self) -> Result<()>;
    async fn needs_reprocessing(&self, doc_id: &str, rev_id: &str) -> Result<bool>;
    async fn upsert_document(&self, document: &Document, embeddings: &[Vec<f32>]) -> Result<()>;
    async fn delete_document(&self, doc_id: &str) -> Result<()>;
    async fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
        filters: Option<HashMap<String, Value>>,
    ) -> Result<Vec<SearchResult>>;
    async fn get_collection_info(&self) -> Result<CollectionInfo>;
    async fn list_documents(&self, page: usize, page_size: usize) -> Result<Vec<DocumentSummary>>;
    async fn get_document_details(&self, doc_id: &str) -> Result<Option<DocumentDetails>>;
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub chunk_id: String,
    pub document_id: String,
    pub document_title: String,
    pub content: String,
    pub heading: Option<String>,
    pub section: String,
    pub doc_type: String,
}

#[derive(Debug)]
pub struct CollectionInfo {
    pub name: String,
    pub vectors_count: u64,
    pub points_count: u64,
    pub active_documents: u64,
    pub tombstoned_documents: u64,
}

#[derive(Debug, Clone)]
pub struct DocumentSummary {
    pub doc_id: String,
    pub title: String,
    pub rel_path: String,
    pub doc_type: String,
    pub chunk_count: usize,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct DocumentDetails {
    pub doc_id: String,
    pub title: String,
    pub rel_path: String,
    pub abs_path: String,
    pub doc_type: String,
    pub section: String,
    pub size: u64,
    pub chunks: Vec<ChunkInfo>,
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub chunk_id: String,
    pub content: String,
    pub start_byte: Option<u64>,
    pub end_byte: Option<u64>,
}
