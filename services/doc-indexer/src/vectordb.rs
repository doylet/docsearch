use anyhow::Result;
use qdrant_client::{
    qdrant::{
        vectors_config::Config, CreateCollection, Distance,
        PointStruct, SearchPoints, VectorParams, VectorsConfig,
        Filter, Condition, FieldCondition, Match as QdrantMatch, Value as QdrantValue,
        PointsSelector, HasIdCondition, WithPayloadSelector, WithVectorsSelector,
    },
    Qdrant,
};
use serde_json::Value;
use std::collections::HashMap;

use crate::document::Document;

pub struct VectorDB {
    client: Qdrant,
    collection_name: String,
}

impl VectorDB {
    pub async fn new(url: &str, collection_name: String) -> Result<Self> {
        let client = Qdrant::from_url(url).build()?;
        
        Ok(Self {
            client,
            collection_name,
        })
    }

    pub async fn ensure_collection_exists(&self) -> Result<()> {
        // Check if collection exists
        match self.client.collection_info(&self.collection_name).await {
            Ok(_) => {
                tracing::info!("Collection '{}' already exists", self.collection_name);
                return Ok(());
            }
            Err(_) => {
                tracing::info!("Creating collection '{}'", self.collection_name);
            }
        }

        // Create collection with appropriate vector configuration
        let create_collection = CreateCollection {
            collection_name: self.collection_name.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: 1536, // OpenAI text-embedding-ada-002 dimensions
                    distance: Distance::Cosine as i32,
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        self.client.create_collection(&create_collection).await?;
        tracing::info!("Collection '{}' created successfully", self.collection_name);

        Ok(())
    }

    pub async fn upsert_document(&self, document: &Document, embeddings: &[Vec<f32>]) -> Result<()> {
        let mut points = Vec::new();

        // Create points for each chunk with its embedding
        for (chunk, embedding) in document.chunks.iter().zip(embeddings.iter()) {
            let mut payload = HashMap::new();
            
            // Document-level metadata
            payload.insert("document_id".to_string(), QdrantValue::from(document.id.clone()));
            payload.insert("document_path".to_string(), QdrantValue::from(document.path.clone()));
            payload.insert("document_title".to_string(), QdrantValue::from(document.title.clone()));
            payload.insert("section".to_string(), QdrantValue::from(document.metadata.section.clone()));
            payload.insert("doc_type".to_string(), QdrantValue::from(document.metadata.doc_type.clone()));
            
            // Chunk-level metadata
            payload.insert("chunk_id".to_string(), QdrantValue::from(chunk.id.clone()));
            payload.insert("content".to_string(), QdrantValue::from(chunk.content.clone()));
            payload.insert("start_line".to_string(), QdrantValue::from(chunk.start_line as i64));
            payload.insert("end_line".to_string(), QdrantValue::from(chunk.end_line as i64));
            payload.insert("chunk_type".to_string(), QdrantValue::from(format!("{:?}", chunk.chunk_type)));
            
            if let Some(heading) = &chunk.heading {
                payload.insert("heading".to_string(), QdrantValue::from(heading.clone()));
            }

            let point = PointStruct::new(
                chunk.id.clone(),
                embedding.clone(),
                payload,
            );
            
            points.push(point);
        }

        // Upsert all points for this document
        self.client
            .upsert_points(&self.collection_name, None, points, None)
            .await?;

        tracing::debug!("Upserted {} chunks for document: {}", document.chunks.len(), document.title);
        Ok(())
    }

    pub async fn delete_document(&self, document_id: &str) -> Result<()> {
        let condition = Condition {
            condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                FieldCondition {
                    key: "document_id".to_string(),
                    r#match: Some(QdrantMatch {
                        match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(document_id.to_string())),
                    }),
                    ..Default::default()
                }
            )),
        };

        let filter = Filter {
            must: vec![condition],
            ..Default::default()
        };

        let points_selector = PointsSelector {
            points_selector_one_of: Some(
                qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Filter(filter)
            ),
        };
        
        self.client
            .delete_points(&self.collection_name, None, &points_selector, None)
            .await?;

        tracing::debug!("Deleted chunks for document: {}", document_id);
        Ok(())
    }

    pub async fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
        _filters: Option<HashMap<String, Value>>,
    ) -> Result<Vec<SearchResult>> {
        let search_points = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: query_vector.to_vec(),
            limit: limit as u64,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: Some(WithVectorsSelector {
                selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(false)),
            }),
            ..Default::default()
        };

        let response = self.client.search_points(&search_points).await?;
        
        let mut results = Vec::new();
        for scored_point in response.result {
            let payload = scored_point.payload;
            
            let result = SearchResult {
                score: scored_point.score,
                chunk_id: payload.get("chunk_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                document_id: payload.get("document_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                document_title: payload.get("document_title")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                content: payload.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                heading: payload.get("heading")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                section: payload.get("section")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                doc_type: payload.get("doc_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            };
            results.push(result);
        }

        Ok(results)
    }

    pub async fn get_collection_info(&self) -> Result<CollectionInfo> {
        let info = self.client.collection_info(&self.collection_name).await?;
        
        Ok(CollectionInfo {
            name: self.collection_name.clone(),
            vectors_count: info.result.map(|r| r.vectors_count.unwrap_or(0)).unwrap_or(0),
            points_count: info.result.map(|r| r.points_count.unwrap_or(0)).unwrap_or(0),
        })
    }
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
}

impl QdrantValue {
    fn as_str(&self) -> Option<&str> {
        match &self.kind {
            Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => Some(s),
            _ => None,
        }
    }
}
