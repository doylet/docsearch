/// Qdrant vector store adapter
///
/// This adapter implements the VectorRepository trait for Qdrant vector database,
/// providing concrete implementation for vector storage and retrieval operations
/// over Qdrant's REST API.
use async_trait::async_trait;
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::str::FromStr;
use zero_latency_core::{models::HealthStatus, values::Score, Result, Uuid, ZeroLatencyError};
use zero_latency_vector::{SimilarityResult, VectorDocument, VectorMetadata, VectorRepository};

/// Points per upsert request
const UPSERT_BATCH_SIZE: usize = 256;

/// Qdrant-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
}

/// Qdrant vector store adapter
pub struct QdrantAdapter {
    config: QdrantConfig,
    client: Client,
    base_url: String,
}

impl QdrantAdapter {
    /// Create a new Qdrant adapter
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        tracing::info!(
            "QdrantAdapter: Setting up REST client for {} with collection '{}'",
            config.url,
            config.collection_name
        );

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                ZeroLatencyError::database(format!("Failed to create HTTP client: {}", e))
            })?;
        let base_url = config.url.trim_end_matches('/').to_string();

        Ok(Self {
            config,
            client,
            base_url,
        })
    }

    fn collection_path(&self, collection: &str, rest: &str) -> String {
        format!("/collections/{}{}", collection, rest)
    }

    /// Send a request, adding the API key when configured. Returns the status and
    /// raw body; only transport failures are errors here.
    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<(StatusCode, String)> {
        let mut request = self
            .client
            .request(method, format!("{}{}", self.base_url, path));
        if let Some(api_key) = &self.config.api_key {
            request = request.header("api-key", api_key);
        }
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await.map_err(|e| {
            ZeroLatencyError::external_service("qdrant", format!("Request failed: {}", e))
        })?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Ok((status, text))
    }

    /// Send a request and fail on any non-2xx status
    async fn send_ok(&self, method: Method, path: &str, body: Option<&Value>) -> Result<String> {
        let (status, text) = self.send(method, path, body).await?;
        if !status.is_success() {
            return Err(http_error(status, &text));
        }
        Ok(text)
    }

    async fn point_exists(&self, id: &str) -> Result<bool> {
        let path = self.collection_path(&self.config.collection_name, &format!("/points/{}", id));
        let (status, text) = self.send(Method::GET, &path, None).await?;
        match status {
            s if s.is_success() => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            s => Err(http_error(s, &text)),
        }
    }

    async fn create_collection(&self, dimension: usize) -> Result<()> {
        tracing::info!(
            "QdrantAdapter: Creating collection '{}' with dimension {}",
            self.config.collection_name,
            dimension
        );
        let body = json!({ "vectors": { "size": dimension, "distance": "Cosine" } });
        let path = self.collection_path(&self.config.collection_name, "");
        self.send_ok(Method::PUT, &path, Some(&body)).await?;
        Ok(())
    }

    /// Payload stored with each point; the inverse of `from_qdrant_rest_result`
    fn to_payload(metadata: &VectorMetadata) -> Map<String, Value> {
        let mut payload = Map::new();
        payload.insert(
            "document_id".into(),
            json!(metadata.document_id.to_string()),
        );
        payload.insert("chunk_index".into(), json!(metadata.chunk_index));
        payload.insert("content".into(), json!(metadata.content));
        payload.insert("title".into(), json!(metadata.title));
        payload.insert("heading_path".into(), json!(metadata.heading_path));
        if let Some(url) = &metadata.url {
            payload.insert("url".into(), json!(url));
        }
        if let Some(collection) = &metadata.collection {
            payload.insert("collection".into(), json!(collection));
        }
        for (key, value) in &metadata.custom {
            payload.insert(format!("custom_{}", key), json!(value));
        }
        payload
    }

    /// Convert a Qdrant REST API search result to VectorDocument
    fn from_qdrant_rest_result(result: &QdrantSearchResult) -> Result<VectorDocument> {
        let document_id = result
            .payload
            .get("document_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ZeroLatencyError::database("Missing document_id in payload"))?;

        let chunk_index = result
            .payload
            .get("chunk_index")
            .and_then(|v| v.as_u64().map(|i| i as usize))
            .unwrap_or(0);

        let content = result
            .payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let title = result
            .payload
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Stored as an array; older points used a '/'-joined string
        let heading_path = match result.payload.get("heading_path") {
            Some(Value::Array(items)) => items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            Some(Value::String(s)) if !s.is_empty() => s.split('/').map(str::to_string).collect(),
            _ => Vec::new(),
        };

        let url = result
            .payload
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let collection = result
            .payload
            .get("collection")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut custom = HashMap::new();
        for (key, value) in &result.payload {
            if let (Some(name), Some(s)) = (key.strip_prefix("custom_"), value.as_str()) {
                custom.insert(name.to_string(), s.to_string());
            }
        }

        let metadata = VectorMetadata {
            document_id: Uuid::from_str(document_id).unwrap_or_else(|_| Uuid::new_v4()),
            chunk_index,
            content,
            title,
            heading_path,
            url,
            collection,
            custom,
        };

        // Convert id to string
        let id_str = match &result.id {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => Uuid::new_v4().to_string(),
        };

        Ok(VectorDocument {
            id: Uuid::from_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            embedding: result.vector.clone().unwrap_or_default(),
            metadata,
        })
    }

    /// Search the configured Qdrant collection, optionally restricted to one
    /// logical collection (the `collection` payload field), as the other
    /// adapters do. Points without a collection belong to "default".
    async fn search_points(
        &self,
        logical_collection: Option<&str>,
        query_vector: Vec<f32>,
        k: usize,
    ) -> Result<Vec<SimilarityResult>> {
        tracing::debug!(
            "QdrantAdapter: Searching '{}' (logical collection {:?}) with vector size {} for {} results",
            self.config.collection_name,
            logical_collection,
            query_vector.len(),
            k
        );

        let mut body = json!({
            "vector": query_vector,
            "limit": k,
            "with_payload": true,
            "with_vector": true,
            "score_threshold": 0.0
        });
        if let Some(name) = logical_collection {
            let matches = json!({ "key": "collection", "match": { "value": name } });
            body["filter"] = if name == "default" {
                json!({ "should": [matches, { "is_empty": { "key": "collection" } }] })
            } else {
                json!({ "must": [matches] })
            };
        }

        let path = self.collection_path(&self.config.collection_name, "/points/search");
        let (status, text) = self.send(Method::POST, &path, Some(&body)).await?;
        if status == StatusCode::NOT_FOUND {
            // Nothing has been indexed yet, so the collection doesn't exist
            return Ok(Vec::new());
        }
        if !status.is_success() {
            return Err(http_error(status, &text));
        }

        let search_response: QdrantSearchResponse = serde_json::from_str(&text).map_err(|e| {
            ZeroLatencyError::external_service(
                "qdrant",
                format!("Failed to parse search response: {}", e),
            )
        })?;

        let mut similarity_results = Vec::new();
        for result in &search_response.result {
            match Self::from_qdrant_rest_result(result) {
                Ok(document) => similarity_results.push(SimilarityResult {
                    document_id: document.metadata.document_id,
                    similarity: Score::new(result.score).unwrap_or_default(),
                    metadata: document.metadata,
                }),
                Err(e) => tracing::warn!("QdrantAdapter: Skipping unreadable result: {}", e),
            }
        }

        Ok(similarity_results)
    }
}

fn http_error(status: StatusCode, body: &str) -> ZeroLatencyError {
    ZeroLatencyError::external_service("qdrant", format!("HTTP {}: {}", status, body))
}

#[derive(Debug, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantSearchResult>,
}

#[derive(Debug, Deserialize)]
struct QdrantSearchResult {
    id: Value, // Can be string or number
    score: f32,
    payload: HashMap<String, Value>,
    vector: Option<Vec<f32>>,
}

#[async_trait]
impl VectorRepository for QdrantAdapter {
    async fn insert(&self, vectors: Vec<VectorDocument>) -> Result<()> {
        let path = self.collection_path(&self.config.collection_name, "/points?wait=true");

        for batch in vectors.chunks(UPSERT_BATCH_SIZE) {
            let points: Vec<Value> = batch
                .iter()
                .map(|doc| {
                    json!({
                        "id": doc.id.to_string(),
                        "vector": doc.embedding,
                        "payload": Self::to_payload(&doc.metadata),
                    })
                })
                .collect();
            let body = json!({ "points": points });

            let (status, text) = self.send(Method::PUT, &path, Some(&body)).await?;
            if status == StatusCode::NOT_FOUND {
                // Collection doesn't exist yet: size it from the vectors and retry once
                self.create_collection(batch[0].embedding.len()).await?;
                self.send_ok(Method::PUT, &path, Some(&body)).await?;
            } else if !status.is_success() {
                return Err(http_error(status, &text));
            }
        }

        Ok(())
    }

    async fn search(&self, query_vector: Vec<f32>, k: usize) -> Result<Vec<SimilarityResult>> {
        self.search_points(None, query_vector, k).await
    }

    async fn search_in_collection(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        k: usize,
    ) -> Result<Vec<SimilarityResult>> {
        self.search_points(Some(collection_name), query_vector, k)
            .await
    }

    async fn delete(&self, document_id: &str) -> Result<bool> {
        if !self.point_exists(document_id).await? {
            return Ok(false);
        }
        let path = self.collection_path(&self.config.collection_name, "/points/delete?wait=true");
        let body = json!({ "points": [document_id] });
        self.send_ok(Method::POST, &path, Some(&body)).await?;
        Ok(true)
    }

    async fn delete_collection(&self, collection_name: &str) -> Result<usize> {
        let filter =
            json!({ "must": [{ "key": "collection", "match": { "value": collection_name } }] });

        let path = self.collection_path(&self.config.collection_name, "/points/count");
        let body = json!({ "filter": filter, "exact": true });
        let (status, text) = self.send(Method::POST, &path, Some(&body)).await?;
        if status == StatusCode::NOT_FOUND {
            // Nothing has been indexed yet, so there is nothing to delete
            return Ok(0);
        }
        if !status.is_success() {
            return Err(http_error(status, &text));
        }
        let count = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["result"]["count"].as_u64())
            .ok_or_else(|| {
                ZeroLatencyError::external_service(
                    "qdrant",
                    format!("Unexpected count response: {}", text),
                )
            })? as usize;
        if count == 0 {
            return Ok(0);
        }

        let path = self.collection_path(&self.config.collection_name, "/points/delete?wait=true");
        self.send_ok(Method::POST, &path, Some(&json!({ "filter": filter })))
            .await?;
        Ok(count)
    }

    async fn update(&self, document_id: &str, vector: Vec<f32>) -> Result<bool> {
        if !self.point_exists(document_id).await? {
            return Ok(false);
        }
        let path = self.collection_path(&self.config.collection_name, "/points/vectors?wait=true");
        let body = json!({ "points": [{ "id": document_id, "vector": vector }] });
        self.send_ok(Method::PUT, &path, Some(&body)).await?;
        Ok(true)
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        match self.send(Method::GET, "/healthz", None).await {
            Ok((status, _)) if status.is_success() => Ok(HealthStatus::Healthy),
            Ok((status, text)) => Ok(HealthStatus::Unhealthy {
                message: format!("Qdrant health check returned HTTP {}: {}", status, text),
            }),
            Err(e) => Ok(HealthStatus::Unhealthy {
                message: e.to_string(),
            }),
        }
    }

    async fn count(&self) -> Result<usize> {
        let path = self.collection_path(&self.config.collection_name, "/points/count");
        let body = json!({ "exact": true });
        let (status, text) = self.send(Method::POST, &path, Some(&body)).await?;
        if status == StatusCode::NOT_FOUND {
            return Ok(0);
        }
        if !status.is_success() {
            return Err(http_error(status, &text));
        }

        let parsed: Value = serde_json::from_str(&text).map_err(|e| {
            ZeroLatencyError::external_service(
                "qdrant",
                format!("Failed to parse count response: {}", e),
            )
        })?;
        parsed["result"]["count"]
            .as_u64()
            .map(|n| n as usize)
            .ok_or_else(|| {
                ZeroLatencyError::external_service("qdrant", "Count response missing result.count")
            })
    }
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            collection_name: "zero_latency_docs".to_string(),
            api_key: None,
            timeout_seconds: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::mock_http::{MockServer, RecordedRequest};
    use axum::http::{Method, StatusCode};

    const POINTS: &str = "/collections/docs/points";

    async fn adapter_for(server: &MockServer, api_key: Option<&str>) -> QdrantAdapter {
        QdrantAdapter::new(QdrantConfig {
            url: server.base_url.clone(),
            collection_name: "docs".to_string(),
            api_key: api_key.map(str::to_string),
            timeout_seconds: 5,
        })
        .await
        .unwrap()
    }

    fn ok() -> (StatusCode, Value) {
        (StatusCode::OK, json!({ "result": true, "status": "ok" }))
    }

    fn not_found() -> (StatusCode, Value) {
        (
            StatusCode::NOT_FOUND,
            json!({ "status": { "error": "Not found" } }),
        )
    }

    fn sample_document() -> VectorDocument {
        let mut custom = HashMap::new();
        custom.insert("file_type".to_string(), "md".to_string());
        VectorDocument {
            id: Uuid::new_v4(),
            embedding: vec![0.1, 0.2, 0.3],
            metadata: VectorMetadata {
                document_id: Uuid::new_v4(),
                chunk_index: 2,
                content: "Some content".to_string(),
                title: "A Title".to_string(),
                heading_path: vec!["Intro".to_string(), "Setup".to_string()],
                url: Some("/docs/a.md".to_string()),
                collection: Some("guides".to_string()),
                custom,
            },
        }
    }

    #[tokio::test]
    async fn test_insert_then_search_round_trips_metadata() {
        // Echo the upserted points back from search, as Qdrant would
        let stored = std::sync::Arc::new(std::sync::Mutex::new(Vec::<Value>::new()));
        let store = stored.clone();
        let server = MockServer::start(move |req: &RecordedRequest, _| match req.route() {
            POINTS => {
                let points = req.body["points"].as_array().unwrap().clone();
                store.lock().unwrap().extend(points);
                ok()
            }
            "/collections/docs/points/search" => {
                let result: Vec<Value> = store
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|p| json!({ "id": p["id"], "score": 0.9, "payload": p["payload"], "vector": p["vector"] }))
                    .collect();
                (StatusCode::OK, json!({ "result": result }))
            }
            _ => not_found(),
        })
        .await;
        let adapter = adapter_for(&server, None).await;
        let document = sample_document();

        adapter.insert(vec![document.clone()]).await.unwrap();
        let results = adapter.search(vec![0.1, 0.2, 0.3], 5).await.unwrap();

        assert_eq!(results.len(), 1);
        let m = &results[0].metadata;
        let expected = &document.metadata;
        assert_eq!(results[0].document_id, expected.document_id);
        assert_eq!(m.document_id, expected.document_id);
        assert_eq!(m.chunk_index, expected.chunk_index);
        assert_eq!(m.content, expected.content);
        assert_eq!(m.title, expected.title);
        assert_eq!(m.heading_path, expected.heading_path);
        assert_eq!(m.url, expected.url);
        assert_eq!(m.collection, expected.collection);
        assert_eq!(m.custom, expected.custom);

        let upserts = server.requests_to(Method::PUT, POINTS);
        assert_eq!(upserts[0].path, "/collections/docs/points?wait=true");
        assert_eq!(upserts[0].body["points"][0]["id"], document.id.to_string());
    }

    #[tokio::test]
    async fn test_insert_creates_missing_collection() {
        let server = MockServer::start(|req: &RecordedRequest, seen| {
            match (req.method.clone(), req.route()) {
                (Method::PUT, POINTS) if seen == 0 => not_found(),
                (Method::PUT, POINTS) => ok(),
                (Method::PUT, "/collections/docs") => ok(),
                _ => not_found(),
            }
        })
        .await;
        let adapter = adapter_for(&server, None).await;

        adapter.insert(vec![sample_document()]).await.unwrap();

        let creates = server.requests_to(Method::PUT, "/collections/docs");
        assert_eq!(creates.len(), 1);
        assert_eq!(
            creates[0].body,
            json!({ "vectors": { "size": 3, "distance": "Cosine" } })
        );
        assert_eq!(server.requests_to(Method::PUT, POINTS).len(), 2);
    }

    #[tokio::test]
    async fn test_insert_batches_points() {
        let server = MockServer::start(|_, _| ok()).await;
        let adapter = adapter_for(&server, None).await;

        let documents: Vec<VectorDocument> = (0..600).map(|_| sample_document()).collect();
        adapter.insert(documents).await.unwrap();

        let sizes: Vec<usize> = server
            .requests_to(Method::PUT, POINTS)
            .iter()
            .map(|r| r.body["points"].as_array().unwrap().len())
            .collect();
        assert_eq!(sizes, vec![256, 256, 88]);
    }

    #[tokio::test]
    async fn test_rejected_write_is_error() {
        let server = MockServer::start(|_, _| {
            (
                StatusCode::BAD_REQUEST,
                json!({ "status": { "error": "Wrong input: Vector dimension error" } }),
            )
        })
        .await;
        let adapter = adapter_for(&server, None).await;

        let error = adapter.insert(vec![sample_document()]).await.unwrap_err();
        let message = error.to_string();
        assert!(message.contains("400"), "{}", message);
        assert!(message.contains("Vector dimension error"), "{}", message);
    }

    #[tokio::test]
    async fn test_delete_existing_point() {
        let id = Uuid::new_v4().to_string();
        let point_path = format!("{}/{}", POINTS, id);
        let server = MockServer::start(move |req: &RecordedRequest, _| {
            if req.route() == point_path || req.route() == "/collections/docs/points/delete" {
                ok()
            } else {
                not_found()
            }
        })
        .await;
        let adapter = adapter_for(&server, None).await;

        assert!(adapter.delete(&id).await.unwrap());

        let deletes = server.requests_to(Method::POST, "/collections/docs/points/delete");
        assert_eq!(deletes.len(), 1);
        assert_eq!(deletes[0].body, json!({ "points": [id] }));
    }

    #[tokio::test]
    async fn test_delete_and_update_unknown_point() {
        let server = MockServer::start(|_, _| not_found()).await;
        let adapter = adapter_for(&server, None).await;
        let id = Uuid::new_v4().to_string();

        assert!(!adapter.delete(&id).await.unwrap());
        assert!(!adapter.update(&id, vec![1.0]).await.unwrap());

        // Only the existence checks were sent, no writes
        assert!(server.requests().iter().all(|r| r.method == Method::GET));
    }

    #[tokio::test]
    async fn test_update_existing_point() {
        let server = MockServer::start(|_, _| ok()).await;
        let adapter = adapter_for(&server, None).await;
        let id = Uuid::new_v4().to_string();

        assert!(adapter.update(&id, vec![0.5, 0.5]).await.unwrap());

        let updates = server.requests_to(Method::PUT, "/collections/docs/points/vectors");
        assert_eq!(updates.len(), 1);
        assert_eq!(
            updates[0].body,
            json!({ "points": [{ "id": id, "vector": [0.5, 0.5] }] })
        );
    }

    #[tokio::test]
    async fn test_count() {
        let server = MockServer::start(|req: &RecordedRequest, _| {
            assert_eq!(req.body, json!({ "exact": true }));
            (StatusCode::OK, json!({ "result": { "count": 3 } }))
        })
        .await;
        let adapter = adapter_for(&server, None).await;

        assert_eq!(adapter.count().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_count_missing_collection_is_zero() {
        let server = MockServer::start(|_, _| not_found()).await;
        let adapter = adapter_for(&server, None).await;

        assert_eq!(adapter.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let server = MockServer::start(|_, _| ok()).await;
        let adapter = adapter_for(&server, None).await;
        assert!(adapter.health_check().await.unwrap().is_healthy());
        assert_eq!(server.requests()[0].route(), "/healthz");

        let down = MockServer::start(|_, _| (StatusCode::SERVICE_UNAVAILABLE, json!({}))).await;
        let adapter = adapter_for(&down, None).await;
        assert!(!adapter.health_check().await.unwrap().is_healthy());
    }

    #[tokio::test]
    async fn test_health_check_unreachable() {
        let adapter = QdrantAdapter::new(QdrantConfig {
            url: "http://127.0.0.1:1".to_string(),
            timeout_seconds: 2,
            ..Default::default()
        })
        .await
        .unwrap();

        assert!(!adapter.health_check().await.unwrap().is_healthy());
    }

    #[tokio::test]
    async fn test_search_in_collection_filters_by_payload() {
        let server = MockServer::start(|_, _| (StatusCode::OK, json!({ "result": [] }))).await;
        let adapter = adapter_for(&server, None).await;

        adapter
            .search_in_collection("guides", vec![0.1], 3)
            .await
            .unwrap();
        adapter
            .search_in_collection("default", vec![0.1], 3)
            .await
            .unwrap();
        adapter.search(vec![0.1], 3).await.unwrap();

        // All three search the configured Qdrant collection
        let searches = server.requests_to(Method::POST, "/collections/docs/points/search");
        assert_eq!(searches.len(), 3);
        assert_eq!(
            searches[0].body["filter"],
            json!({ "must": [{ "key": "collection", "match": { "value": "guides" } }] })
        );
        assert_eq!(
            searches[1].body["filter"],
            json!({ "should": [
                { "key": "collection", "match": { "value": "default" } },
                { "is_empty": { "key": "collection" } }
            ] })
        );
        assert!(searches[2].body.get("filter").is_none());
    }

    #[tokio::test]
    async fn test_search_before_first_insert_is_empty() {
        let server = MockServer::start(|_, _| not_found()).await;
        let adapter = adapter_for(&server, None).await;

        assert!(adapter.search(vec![0.1], 3).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_api_key_sent_on_every_request() {
        let server = MockServer::start(|req: &RecordedRequest, _| match req.route() {
            "/collections/docs/points/search" => (StatusCode::OK, json!({ "result": [] })),
            "/collections/docs/points/count" => {
                (StatusCode::OK, json!({ "result": { "count": 0 } }))
            }
            _ => ok(),
        })
        .await;
        let adapter = adapter_for(&server, Some("secret")).await;
        let id = Uuid::new_v4().to_string();

        adapter.insert(vec![sample_document()]).await.unwrap();
        adapter.search(vec![0.1], 1).await.unwrap();
        adapter
            .search_in_collection("other", vec![0.1], 1)
            .await
            .unwrap();
        adapter.delete(&id).await.unwrap();
        adapter.update(&id, vec![0.1]).await.unwrap();
        adapter.count().await.unwrap();
        adapter.health_check().await.unwrap();

        let requests = server.requests();
        assert!(requests.len() >= 9);
        for request in requests {
            assert_eq!(
                request.header("api-key"),
                Some("secret"),
                "{}",
                request.path
            );
        }
    }

    #[tokio::test]
    async fn test_delete_collection_filters_on_collection_payload() {
        let server = MockServer::start(|req: &RecordedRequest, _| match req.route() {
            "/collections/docs/points/count" => (
                StatusCode::OK,
                json!({ "result": { "count": 3 }, "status": "ok" }),
            ),
            "/collections/docs/points/delete" => ok(),
            _ => not_found(),
        })
        .await;
        let adapter = adapter_for(&server, None).await;

        assert_eq!(adapter.delete_collection("guides").await.unwrap(), 3);

        let filter = json!({ "must": [{ "key": "collection", "match": { "value": "guides" } }] });
        let deletes = server.requests_to(Method::POST, "/collections/docs/points/delete");
        assert_eq!(deletes.len(), 1);
        assert_eq!(deletes[0].body, json!({ "filter": filter }));
    }

    #[tokio::test]
    async fn test_delete_collection_before_anything_is_indexed() {
        let server = MockServer::start(|_, _| not_found()).await;
        let adapter = adapter_for(&server, None).await;

        assert_eq!(adapter.delete_collection("guides").await.unwrap(), 0);
        assert!(server
            .requests_to(Method::POST, "/collections/docs/points/delete")
            .is_empty());
    }
}
