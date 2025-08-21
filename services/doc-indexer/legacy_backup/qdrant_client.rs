use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::{debug, warn, error};
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollection, Distance, VectorParams, VectorsConfig,
        PointStruct, Value, Condition, Filter, UpsertPoints,
        SearchPoints, DeletePoints, FieldCondition, Match, WithPayloadSelector,
        PointId, PointsSelector, ScrollPoints
    },
};
use serde_json;
use std::collections::HashMap;

use crate::document::Document;
use crate::vector_db_trait::{VectorDatabase, SearchResult, CollectionInfo};

/// Production Qdrant client with proper collection management
pub struct QdrantVectorDB {
    client: Qdrant,
    collection_name: String,
}

impl QdrantVectorDB {
    async fn ensure_collection_exists_impl(&self) -> Result<()> {
        // Check if collection exists
        let collections = self.client.list_collections().await
            .context("Failed to list collections")?
            .collections;

        let collection_exists = collections
            .iter()
            .any(|c| c.name == self.collection_name);

        if !collection_exists {
            // Create collection with 384-dimensional vectors for all-MiniLM-L6-v2
            let collection_config = CreateCollection {
                collection_name: self.collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(qdrant_client::qdrant::vectors_config::Config::Params(VectorParams {
                        size: 384,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                        datatype: None,
                        multivector_config: None,
                    })),
                }),
                hnsw_config: None,
                wal_config: None,
                optimizers_config: None,
                shard_number: None,
                on_disk_payload: None,
                timeout: None,
                replication_factor: None,
                write_consistency_factor: None,
                init_from_collection: None,
                quantization_config: None,
                sharding_method: None,
                sparse_vectors_config: None,
                strict_mode_config: None,
            };

            self.client
                .create_collection(collection_config)
                .await
                .context("Failed to create collection")?;

            println!("Created Qdrant collection: {}", self.collection_name);
        }

        Ok(())
    }

    fn point_id_from_chunk_id(chunk_id: &str) -> u64 {
        // Convert chunk_id to a stable numeric ID using hash
        let mut hasher = DefaultHasher::new();
        chunk_id.hash(&mut hasher);
        hasher.finish()
    }

    fn create_chunk_metadata(doc_id: &str, chunk_index: usize, chunk_content: &str, document: &Document) -> HashMap<String, Value> {
        let mut payload = HashMap::new();
        
        payload.insert("document_id".to_string(), Value::from(doc_id));
        payload.insert("chunk_index".to_string(), Value::from(chunk_index as i64));
        payload.insert("content".to_string(), Value::from(chunk_content));
        payload.insert("abs_path".to_string(), Value::from(document.abs_path.clone()));
        payload.insert("rel_path".to_string(), Value::from(document.rel_path.clone()));
        payload.insert("title".to_string(), Value::from(document.title.clone()));
        payload.insert("size".to_string(), Value::from(document.metadata.size as i64));
        payload.insert("doc_id".to_string(), Value::from(document.doc_id.clone()));
        payload.insert("rev_id".to_string(), Value::from(document.rev_id.clone()));
        payload.insert("section".to_string(), Value::from(document.metadata.section.clone()));
        payload.insert("doc_type".to_string(), Value::from(document.metadata.doc_type.clone()));
        
        // Add chunk-specific metadata
        if let Some(chunk) = document.chunks.get(chunk_index) {
            payload.insert("start_byte".to_string(), Value::from(chunk.start_byte as i64));
            payload.insert("end_byte".to_string(), Value::from(chunk.end_byte as i64));
            payload.insert("chunk_id".to_string(), Value::from(chunk.chunk_id.clone()));
        }
        
        payload
    }
}

#[async_trait]
impl VectorDatabase for QdrantVectorDB {
    async fn ensure_collection_exists(&self) -> Result<()> {
        self.ensure_collection_exists_impl().await
    }

    async fn needs_reprocessing(&self, doc_id: &str, rev_id: &str) -> Result<bool> {
        // First, check if any chunks exist for this document ID
        let doc_filter = Filter {
            must: vec![
                Condition {
                    condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                        FieldCondition {
                            key: "document_id".to_string(),
                            r#match: Some(Match {
                                match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(doc_id.to_string())),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        },
                    )),
                }
            ],
            should: vec![],
            must_not: vec![],
            min_should: None,
        };

        let search_points = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: vec![0.0; 384], // Dummy vector for existence check
            filter: Some(doc_filter),
            limit: 1,
            offset: None,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: Some(false.into()),
            params: None,
            score_threshold: None,
            read_consistency: None,
            shard_key_selector: None,
            sparse_indices: None,
            timeout: None,
            vector_name: None,
        };

        let search_result = self.client
            .search_points(search_points)
            .await
            .context("Failed to check document existence")?;

        // If no chunks exist for this document, we need to process it
        if search_result.result.is_empty() {
            return Ok(true);
        }

        // Check if any existing chunk has the same rev_id
        for point in &search_result.result {
            if let Some(stored_rev_id) = point.payload.get("rev_id") {
                if let Some(stored_rev_id_str) = stored_rev_id.as_str() {
                    if stored_rev_id_str == rev_id {
                        // Found a chunk with the same rev_id, document is up to date
                        return Ok(false);
                    }
                }
            }
        }

        // Document exists but with different rev_id, needs reprocessing
        Ok(true)
    }

    async fn upsert_document(&self, document: &Document, embeddings: &[Vec<f32>]) -> Result<()> {
        if embeddings.len() != document.chunks.len() {
            return Err(anyhow::anyhow!(
                "Embeddings count ({}) doesn't match chunks count ({})",
                embeddings.len(),
                document.chunks.len()
            ));
        }

        debug!("Upserting document '{}' with {} chunks", document.doc_id, document.chunks.len());

        // Create points for all chunks in this document
        let mut points = Vec::new();
        for (chunk_index, (chunk, embedding)) in document.chunks.iter().zip(embeddings.iter()).enumerate() {
            let chunk_id = format!("{}#{}", document.doc_id, chunk_index);
            let point_id = Self::point_id_from_chunk_id(&chunk_id);
            let payload = Self::create_chunk_metadata(&document.doc_id, chunk_index, &chunk.content, document);

            debug!("Creating point for chunk {}: id={}, embedding_dim={}, payload_keys={:?}", 
                chunk_index, point_id, embedding.len(), payload.keys().collect::<Vec<_>>());

            let point = PointStruct {
                id: Some(PointId::from(point_id)),
                vectors: Some(embedding.clone().into()),
                payload,
            };
            points.push(point);
        }

        // Batch upsert all points for this document
        if !points.is_empty() {
            debug!("Upserting {} points to collection '{}'", points.len(), self.collection_name);
            
            let upsert_points = UpsertPoints {
                collection_name: self.collection_name.clone(),
                points,
                wait: Some(true),
                ordering: None,
                shard_key_selector: None,
            };

            match self.client.upsert_points(upsert_points).await {
                Ok(response) => {
                    debug!("Upsert successful: {:?}", response);
                    Ok(())
                }
                Err(e) => {
                    error!("Upsert failed for document '{}': {}", document.doc_id, e);
                    Err(anyhow::anyhow!("Failed to upsert document chunks: {}", e))
                }
            }
        } else {
            warn!("No points to upsert for document '{}'", document.doc_id);
            Ok(())
        }
    }

    async fn delete_document(&self, document_id: &str) -> Result<()> {
        // Create filter to delete all chunks for this document
        let filter = Filter {
            should: vec![],
            must: vec![Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: "document_id".to_string(),
                        r#match: Some(Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(document_id.to_string())),
                        }),
                        range: None,
                        geo_bounding_box: None,
                        geo_radius: None,
                        values_count: None,
                        geo_polygon: None,
                        datetime_range: None,
                        is_empty: None,
                        is_null: None,
                    },
                )),
            }],
            must_not: vec![],
            min_should: None,
        };

        let delete_points = DeletePoints {
            collection_name: self.collection_name.clone(),
            points: Some(PointsSelector {
                points_selector_one_of: Some(qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Filter(filter)),
            }),
            wait: Some(true),
            ordering: None,
            shard_key_selector: None,
        };

        self.client
            .delete_points(delete_points)
            .await
            .context("Failed to delete document")?;

        Ok(())
    }

    async fn search(&self, query_embedding: &[f32], limit: usize, filters: Option<HashMap<String, serde_json::Value>>) -> Result<Vec<SearchResult>> {
        let mut filter_conditions = vec![];

        // Apply filters if provided
        if let Some(filters) = filters {
            for (key, value) in filters {
                let value_str = match value {
                    serde_json::Value::String(s) => s,
                    v => v.to_string(),
                };
                let condition = Condition {
                    condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                        FieldCondition {
                            key,
                            r#match: Some(Match {
                                match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(value_str)),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            values_count: None,
                            geo_polygon: None,
                            datetime_range: None,
                            is_empty: None,
                            is_null: None,
                        },
                    )),
                };
                filter_conditions.push(condition);
            }
        }

        let search_filter = if !filter_conditions.is_empty() {
            Some(Filter {
                must: filter_conditions,
                should: vec![],
                must_not: vec![],
                min_should: None,
            })
        } else {
            None
        };

        let search_points = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: query_embedding.to_vec(),
            filter: search_filter,
            limit: limit as u64,
            offset: None,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: Some(false.into()),
            params: None,
            score_threshold: None,
            read_consistency: None,
            shard_key_selector: None,
            sparse_indices: None,
            timeout: None,
            vector_name: None,
        };

        let search_result = self.client
            .search_points(search_points)
            .await
            .context("Failed to search vectors")?;

        let mut results = Vec::new();
        for scored_point in search_result.result {
            let payload = scored_point.payload;
            
            if let (Some(content), Some(chunk_id), Some(document_id), Some(title)) = (
                payload.get("content"),
                payload.get("chunk_id"),
                payload.get("document_id"),
                payload.get("title")
            ) {
                let heading = payload.get("heading").map(|v| v.to_string());
                let section = payload.get("section").map(|v| v.to_string()).unwrap_or_default();
                let doc_type = payload.get("doc_type").map(|v| v.to_string()).unwrap_or_default();

                results.push(SearchResult {
                    score: scored_point.score,
                    chunk_id: chunk_id.to_string(),
                    document_id: document_id.to_string(),
                    document_title: title.to_string(),
                    content: content.to_string(),
                    heading,
                    section,
                    doc_type,
                });
            }
        }

        Ok(results)
    }

    async fn get_collection_info(&self) -> Result<CollectionInfo> {
        let info = self.client
            .collection_info(&self.collection_name)
            .await
            .context("Failed to get collection info")?
            .result
            .ok_or_else(|| anyhow::anyhow!("No collection info returned"))?;

        Ok(CollectionInfo {
            name: self.collection_name.clone(),
            vectors_count: info.vectors_count.unwrap_or(0),
            points_count: info.points_count.unwrap_or(0),
            active_documents: info.points_count.unwrap_or(0),
            tombstoned_documents: 0, // Qdrant doesn't directly expose this
        })
    }

    async fn list_documents(&self, page: usize, page_size: usize) -> Result<Vec<crate::vector_db_trait::DocumentSummary>> {
        use std::collections::HashMap;
        use crate::vector_db_trait::DocumentSummary;

        // Use scroll to get all points with pagination
        let offset = (page - 1) * page_size;
        
        let scroll_points = ScrollPoints {
            collection_name: self.collection_name.clone(),
            filter: None,
            offset: Some(PointId::from(offset as u64)),
            limit: Some(page_size as u32),
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: Some(false.into()),
            read_consistency: None,
            shard_key_selector: None,
            order_by: None,
            timeout: None,
        };

        let response = self.client.scroll(scroll_points).await
            .context("Failed to scroll points")?;

        // Group points by document_id and aggregate
        let mut doc_map: HashMap<String, DocumentSummary> = HashMap::new();
        
        for point in response.result {
            let payload = &point.payload;
            let doc_id = payload.get("document_id")
                .and_then(|v| v.as_str())
                .map_or("unknown".to_string(), |s| s.to_string());
            
            let title = payload.get("title")
                .and_then(|v| v.as_str())
                .map_or("Untitled".to_string(), |s| s.to_string());
            
            let rel_path = payload.get("rel_path")
                .and_then(|v| v.as_str())
                .map_or("".to_string(), |s| s.to_string());
            
            let doc_type = payload.get("doc_type")
                .and_then(|v| v.as_str())
                .map_or("unknown".to_string(), |s| s.to_string());
            
            let size = payload.get("size")
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as u64;

            let entry = doc_map.entry(doc_id.clone()).or_insert_with(|| DocumentSummary {
                doc_id: doc_id.clone(),
                title: title.clone(),
                rel_path: rel_path.clone(),
                doc_type: doc_type.clone(),
                chunk_count: 0,
                size,
            });
            
            entry.chunk_count += 1;
        }

        Ok(doc_map.into_values().collect())
    }

    async fn get_document_details(&self, doc_id: &str) -> Result<Option<crate::vector_db_trait::DocumentDetails>> {
        use crate::vector_db_trait::{DocumentDetails, ChunkInfo};

        // Create filter for this specific document
        let filter = Filter {
            should: vec![],
            min_should: None,
            must: vec![Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: "document_id".to_string(),
                        r#match: Some(Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(doc_id.to_string())),
                        }),
                        range: None,
                        geo_bounding_box: None,
                        geo_radius: None,
                        values_count: None,
                        geo_polygon: None,
                        datetime_range: None,
                        is_empty: None,
                        is_null: None,
                    }
                )),
            }],
            must_not: vec![],
        };

        let scroll_points = ScrollPoints {
            collection_name: self.collection_name.clone(),
            filter: Some(filter),
            offset: None,
            limit: Some(1000), // Get all chunks for this document
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: Some(false.into()),
            read_consistency: None,
            shard_key_selector: None,
            order_by: None,
            timeout: None,
        };

        let response = self.client.scroll(scroll_points).await
            .context("Failed to scroll document points")?;

        if response.result.is_empty() {
            return Ok(None);
        }

        // Get document metadata from first point
        let first_point = &response.result[0];
        let payload = &first_point.payload;
        
        let title = payload.get("title")
            .and_then(|v| v.as_str())
            .map_or("Untitled".to_string(), |s| s.to_string());
        
        let rel_path = payload.get("rel_path")
            .and_then(|v| v.as_str())
            .map_or("".to_string(), |s| s.to_string());
        
        let abs_path = payload.get("abs_path")
            .and_then(|v| v.as_str())
            .map_or("".to_string(), |s| s.to_string());
        
        let doc_type = payload.get("doc_type")
            .and_then(|v| v.as_str())
            .map_or("unknown".to_string(), |s| s.to_string());
        
        let section = payload.get("section")
            .and_then(|v| v.as_str())
            .map_or("".to_string(), |s| s.to_string());
        
        let size = payload.get("size")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u64;

        // Collect all chunks
        let mut chunks = Vec::new();
        for point in response.result {
            let payload = &point.payload;
                let chunk_id = payload.get("chunk_id")
                    .and_then(|v| v.as_str())
                    .map_or("".to_string(), |s| s.to_string());
                
                let content = payload.get("content")
                    .and_then(|v| v.as_str())
                    .map_or("".to_string(), |s| s.to_string());
                
                let start_byte = payload.get("start_byte")
                    .and_then(|v| v.as_integer())
                    .map(|v| v as u64);
                
                let end_byte = payload.get("end_byte")
                    .and_then(|v| v.as_integer())
                    .map(|v| v as u64);

                chunks.push(ChunkInfo {
                    chunk_id,
                    content,
                    start_byte,
                    end_byte,
                });
            }

        Ok(Some(DocumentDetails {
            doc_id: doc_id.to_string(),
            title,
            rel_path,
            abs_path,
            doc_type,
            section,
            size,
            chunks,
        }))
    }
}

impl QdrantVectorDB {
    pub async fn new(url: &str, collection_name: String) -> Result<Self> {
        let client = Qdrant::from_url(url)
            .skip_compatibility_check()  // Disable version compatibility check
            .build()
            .context("Failed to create Qdrant client")?;

        let instance = Self { client, collection_name };
        instance.ensure_collection_exists().await?;
        Ok(instance)
    }
}
