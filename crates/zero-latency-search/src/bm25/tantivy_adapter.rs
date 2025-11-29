#[cfg(feature = "tantivy")]
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, FAST, INDEXED, STORED, TEXT},
    Index, IndexReader, IndexWriter, TantivyDocument,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use zero_latency_core::{DocId, Result, ZeroLatencyError};

use crate::fusion::{FromSignals, ScoreBreakdown, SearchEngine};
use crate::models::SearchResult;
use crate::traits::SearchStep;

/// BM25 search result before conversion to SearchResult
#[derive(Debug, Clone)]
pub struct BM25SearchResult {
    pub doc_id: DocId,
    pub title: String,
    pub content: String,
    pub uri: String,
    pub score: f32,
    pub section_path: Vec<String>,
    pub collection: String,
    pub metadata: HashMap<String, String>,
}

/// Configuration for BM25 search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Config {
    /// Index directory path
    pub index_path: String,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Minimum score threshold
    pub min_score: f32,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            index_path: "tantivy_index".to_string(),
            max_results: 100,
            min_score: 0.0,
        }
    }
}

/// Tantivy BM25 search adapter
#[cfg(feature = "tantivy")]
pub struct TantivyAdapter {
    index: Index,
    reader: IndexReader,
    schema: Schema,
    fields: TantivyFields,
    config: BM25Config,
}

#[cfg(feature = "tantivy")]
struct TantivyFields {
    doc_id: Field,
    title: Field,
    content: Field,
    uri: Field,
    section_path: Field,
    collection: Field,
    metadata: Field,
}

#[cfg(feature = "tantivy")]
impl TantivyAdapter {
    /// Create a new Tantivy BM25 adapter
    pub async fn new(config: BM25Config) -> Result<Self> {
        let mut schema_builder = Schema::builder();

        // Define fields for document indexing
        let doc_id = schema_builder.add_text_field("doc_id", INDEXED | STORED | FAST);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let content = schema_builder.add_text_field("content", TEXT);
        let uri = schema_builder.add_text_field("uri", STORED);
        let section_path = schema_builder.add_text_field("section_path", STORED);
        let collection = schema_builder.add_text_field("collection", INDEXED | STORED | FAST);
        let metadata = schema_builder.add_text_field("metadata", STORED);

        let schema = schema_builder.build();
        let fields = TantivyFields {
            doc_id,
            title,
            content,
            uri,
            section_path,
            collection,
            metadata,
        };

        // Create or open index
        let index_path = Path::new(&config.index_path);
        let index = if index_path.exists() {
            Index::open_in_dir(index_path).map_err(|e| {
                ZeroLatencyError::search(format!("Failed to open Tantivy index: {}", e))
            })?
        } else {
            std::fs::create_dir_all(index_path).map_err(|e| {
                ZeroLatencyError::io(format!("Failed to create index directory: {}", e))
            })?;
            Index::create_in_dir(index_path, schema.clone()).map_err(|e| {
                ZeroLatencyError::search(format!("Failed to create Tantivy index: {}", e))
            })?
        };

        let reader = index.reader().map_err(|e| {
            ZeroLatencyError::search(format!("Failed to create index reader: {}", e))
        })?;

        Ok(Self {
            index,
            reader,
            schema,
            fields,
            config,
        })
    }

    /// Index a document
    pub async fn index_document(&self, result: &BM25SearchResult) -> Result<()> {
        let mut writer = self.index.writer(50_000_000).map_err(|e| {
            ZeroLatencyError::search(format!("Failed to create index writer: {}", e))
        })?;

        let mut doc = TantivyDocument::default();
        doc.add_text(self.fields.doc_id, &result.doc_id.to_index_key());
        doc.add_text(self.fields.title, &result.title);
        doc.add_text(self.fields.content, &result.content);
        doc.add_text(self.fields.uri, &result.uri);
        doc.add_text(self.fields.section_path, &result.section_path.join(" > "));
        doc.add_text(self.fields.collection, &result.collection);

        // Serialize metadata as JSON
        let metadata_json = serde_json::to_string(&result.metadata).unwrap_or_default();
        doc.add_text(self.fields.metadata, &metadata_json);

        writer.add_document(doc).map_err(|e| {
            ZeroLatencyError::search(format!("Failed to add document to index: {}", e))
        })?;

        writer.commit().map_err(|e| {
            ZeroLatencyError::search(format!("Failed to commit index changes: {}", e))
        })?;

        Ok(())
    }

    /// Search the BM25 index
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<BM25SearchResult>> {
        let searcher = self.reader.searcher();

        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.title, self.fields.content]);
        let query = query_parser.parse_query(query).map_err(|e| {
            ZeroLatencyError::search(format!("Failed to parse query: {}", e))
        })?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit)).map_err(|e| {
            ZeroLatencyError::search(format!("Search failed: {}", e))
        })?;

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address).map_err(|e| {
                ZeroLatencyError::search(format!("Failed to retrieve document: {}", e))
            })?;

            // Extract fields from document
            let doc_id_str = retrieved_doc
                .get_first(self.fields.doc_id)
                .and_then(|v| v.as_text())
                .ok_or_else(|| ZeroLatencyError::search("Missing doc_id field".to_string()))?;

            let doc_id = DocId::from_index_key(doc_id_str)
                .ok_or_else(|| ZeroLatencyError::search("Invalid doc_id format".to_string()))?;

            let title = retrieved_doc
                .get_first(self.fields.title)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();

            let content = retrieved_doc
                .get_first(self.fields.content)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();

            let uri = retrieved_doc
                .get_first(self.fields.uri)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();

            let section_path_str = retrieved_doc
                .get_first(self.fields.section_path)
                .and_then(|v| v.as_text())
                .unwrap_or("");

            let section_path = if section_path_str.is_empty() {
                Vec::new()
            } else {
                section_path_str.split(" > ").map(|s| s.to_string()).collect()
            };

            let collection = retrieved_doc
                .get_first(self.fields.collection)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();

            let metadata_str = retrieved_doc
                .get_first(self.fields.metadata)
                .and_then(|v| v.as_text())
                .unwrap_or("{}");

            let metadata: HashMap<String, String> =
                serde_json::from_str(metadata_str).unwrap_or_default();

            if score >= self.config.min_score {
                results.push(BM25SearchResult {
                    doc_id,
                    title,
                    content,
                    uri,
                    score,
                    section_path,
                    collection,
                    metadata,
                });
            }
        }

        Ok(results)
    }

    /// Delete a document from the index
    pub async fn delete_document(&self, doc_id: &DocId) -> Result<()> {
        let mut writer = self.index.writer(50_000_000).map_err(|e| {
            ZeroLatencyError::search(format!("Failed to create index writer: {}", e))
        })?;

        let term = tantivy::Term::from_field_text(self.fields.doc_id, &doc_id.to_index_key());
        writer.delete_term(term);

        writer.commit().map_err(|e| {
            ZeroLatencyError::search(format!("Failed to commit delete: {}", e))
        })?;

        Ok(())
    }
}

/// No-op BM25 adapter for when tantivy feature is disabled
#[cfg(not(feature = "tantivy"))]
pub struct TantivyAdapter {
    config: BM25Config,
}

#[cfg(not(feature = "tantivy"))]
impl TantivyAdapter {
    pub async fn new(config: BM25Config) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn search(&self, _query: &str, _limit: usize) -> Result<Vec<BM25SearchResult>> {
        Err(ZeroLatencyError::search(
            "BM25 search requires tantivy feature to be enabled".to_string()
        ))
    }

    pub async fn index_document(&self, _result: &BM25SearchResult) -> Result<()> {
        Err(ZeroLatencyError::search(
            "BM25 indexing requires tantivy feature to be enabled".to_string()
        ))
    }

    pub async fn delete_document(&self, _doc_id: &DocId) -> Result<()> {
        Err(ZeroLatencyError::search(
            "BM25 deletion requires tantivy feature to be enabled".to_string()
        ))
    }
}

/// BM25 search step for search pipeline
pub struct BM25SearchStep {
    adapter: Arc<TantivyAdapter>,
}

impl BM25SearchStep {
    pub fn new(adapter: Arc<TantivyAdapter>) -> Self {
        Self { adapter }
    }

    /// Convert BM25SearchResult to SearchResult with score breakdown
    fn convert_result(&self, bm25_result: BM25SearchResult, variant_index: usize) -> SearchResult {
        let scores = ScoreBreakdown {
            bm25_raw: Some(bm25_result.score),
            vector_raw: None,
            bm25_normalized: None, // Will be set during fusion
            vector_normalized: None,
            fused: bm25_result.score, // Temporary, will be updated during fusion
            normalization_method: crate::fusion::NormalizationMethod::MinMax,
        };

        let from_signals = FromSignals::from_variant(variant_index, SearchEngine::BM25);

        SearchResult::new(
            bm25_result.doc_id,
            bm25_result.uri,
            bm25_result.title,
            bm25_result.content,
            scores,
            from_signals,
        )
        .with_section_path(bm25_result.section_path)
        .with_collection(bm25_result.collection)
        .with_metadata(bm25_result.metadata)
    }
}

#[async_trait]
impl SearchStep for BM25SearchStep {
    fn name(&self) -> &str {
        "bm25_search"
    }

    async fn execute(&self, context: &mut crate::models::SearchContext) -> Result<()> {
        // Use enhanced query if available, otherwise fall back to original query
        let query_text = if let Some(ref enhanced) = context.enhanced_query {
            &enhanced.enhanced
        } else {
            &context.request.query.raw
        };

        tracing::info!("🔍 BM25SearchStep: Searching with query: '{}'", query_text);

        let bm25_results = self
            .adapter
            .search(query_text, context.request.limit)
            .await?;

        tracing::info!("📊 BM25SearchStep: Found {} BM25 results", bm25_results.len());

        // Convert to SearchResult format
        let search_results: Vec<SearchResult> = bm25_results
            .into_iter()
            .map(|result| self.convert_result(result, 0)) // Variant 0 = original query
            .collect();

        // Add to context (this will be merged with vector results in hybrid step)
        context.raw_results.extend(search_results);
        context.metadata.result_sources.push("bm25".to_string());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    #[cfg(feature = "tantivy")]
    async fn test_tantivy_adapter() {
        let temp_dir = TempDir::new().unwrap();
        let config = BM25Config {
            index_path: temp_dir.path().to_str().unwrap().to_string(),
            max_results: 10,
            min_score: 0.0,
        };

        let adapter = TantivyAdapter::new(config).await.unwrap();

        let doc_id = DocId::new("test", "doc1", 1);
        let test_result = BM25SearchResult {
            doc_id: doc_id.clone(),
            title: "Test Document".to_string(),
            content: "This is a test document for search".to_string(),
            uri: "/test/doc1".to_string(),
            score: 1.0,
            section_path: vec!["Section 1".to_string()],
            collection: "test".to_string(),
            metadata: HashMap::new(),
        };

        // Index the document
        adapter.index_document(&test_result).await.unwrap();

        // Search for it
        let results = adapter.search("test document", 10).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, doc_id);
    }
}
