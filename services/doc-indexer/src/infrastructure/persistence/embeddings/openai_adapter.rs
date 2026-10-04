/// OpenAI embeddings adapter
///
/// This adapter implements the EmbeddingGenerator trait for OpenAI's
/// embedding API, providing text-to-vector conversion capabilities.
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zero_latency_core::{Result, ZeroLatencyError};
use zero_latency_vector::EmbeddingGenerator;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
/// OpenAI accepts up to 2048 inputs per request; stay well under it
const BATCH_SIZE: usize = 100;
/// Rough character budget per input; the API limit is in tokens
const MAX_CHARS: usize = 8000;
const DEFAULT_RETRY_BASE_DELAY: Duration = Duration::from_millis(500);

/// OpenAI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

/// OpenAI embeddings adapter
pub struct OpenAIAdapter {
    config: OpenAIConfig,
    client: reqwest::Client,
    endpoint: String,
    retry_base_delay: Duration,
}

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    index: usize,
    embedding: Vec<f32>,
}

impl OpenAIAdapter {
    /// Create a new OpenAI adapter
    pub async fn new(config: OpenAIConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(ZeroLatencyError::configuration(
                "OpenAI API key is required",
            ));
        }

        let mut builder = reqwest::Client::builder();
        if config.timeout_seconds > 0 {
            builder = builder.timeout(Duration::from_secs(config.timeout_seconds));
        }
        let client = builder.build().map_err(|e| {
            ZeroLatencyError::configuration(format!("Failed to create OpenAI HTTP client: {}", e))
        })?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or(DEFAULT_BASE_URL)
            .trim_end_matches('/');
        let endpoint = format!("{}/embeddings", base_url);

        Ok(Self {
            config,
            client,
            endpoint,
            retry_base_delay: DEFAULT_RETRY_BASE_DELAY,
        })
    }

    /// Set the first retry delay; later retries double it
    pub fn with_retry_base_delay(mut self, delay: Duration) -> Self {
        self.retry_base_delay = delay;
        self
    }

    /// Get the embedding dimension for the configured model
    pub fn embedding_dimension(&self) -> usize {
        match self.config.model.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => 1536, // Default fallback
        }
    }

    /// Prepare text for embedding (truncate if too long)
    fn prepare_text(&self, text: &str) -> String {
        // Character-based truncation, cut on a char boundary
        match text.char_indices().nth(MAX_CHARS) {
            Some((byte_index, _)) => format!("{}...", &text[..byte_index]),
            None => text.to_string(),
        }
    }

    /// Request embeddings for one batch, retrying rate limits and transient failures
    async fn call_api(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let request = EmbeddingRequest {
            model: &self.config.model,
            input: texts,
        };

        let mut attempt = 0;
        loop {
            let error = match self
                .client
                .post(&self.endpoint)
                .bearer_auth(&self.config.api_key)
                .json(&request)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    return Self::parse_response(response, texts.len()).await;
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let retryable = status == reqwest::StatusCode::TOO_MANY_REQUESTS
                        || status.is_server_error();
                    let error = ZeroLatencyError::external_service(
                        "openai",
                        format!("HTTP {}: {}", status, api_error_message(&body)),
                    );
                    if !retryable {
                        return Err(error);
                    }
                    error
                }
                Err(e) => {
                    let error = ZeroLatencyError::external_service(
                        "openai",
                        format!("Request failed: {}", e),
                    );
                    if !(e.is_connect() || e.is_timeout()) {
                        return Err(error);
                    }
                    error
                }
            };

            if attempt >= self.config.max_retries {
                return Err(error);
            }
            tokio::time::sleep(self.retry_base_delay * 2u32.pow(attempt)).await;
            attempt += 1;
        }
    }

    async fn parse_response(response: reqwest::Response, expected: usize) -> Result<Vec<Vec<f32>>> {
        let mut parsed: EmbeddingResponse = response.json().await.map_err(|e| {
            ZeroLatencyError::external_service("openai", format!("Invalid response: {}", e))
        })?;

        if parsed.data.len() != expected {
            return Err(ZeroLatencyError::external_service(
                "openai",
                format!(
                    "Expected {} embeddings, got {}",
                    expected,
                    parsed.data.len()
                ),
            ));
        }

        parsed.data.sort_by_key(|d| d.index);
        Ok(parsed.data.into_iter().map(|d| d.embedding).collect())
    }
}

/// Extract `error.message` from an OpenAI error body, falling back to the raw body
fn api_error_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(str::to_string))
        .unwrap_or_else(|| body.to_string())
}

#[async_trait]
impl EmbeddingGenerator for OpenAIAdapter {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let prepared_text = self.prepare_text(text);
        let embeddings = self.call_api(&[prepared_text]).await?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| ZeroLatencyError::internal("No embedding returned from API"))
    }

    async fn generate_batch_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let prepared_texts: Vec<String> =
            texts.iter().map(|text| self.prepare_text(text)).collect();

        let mut all_embeddings = Vec::with_capacity(prepared_texts.len());
        for batch in prepared_texts.chunks(BATCH_SIZE) {
            all_embeddings.extend(self.call_api(batch).await?);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        self.embedding_dimension()
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "text-embedding-3-small".to_string(),
            base_url: None,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::mock_http::{MockServer, RecordedRequest};
    use axum::http::{Method, StatusCode};
    use serde_json::{json, Value};

    /// Respond to /v1/embeddings with one embedding per input, in reverse order.
    /// Each embedding is [input length in chars, input index].
    fn embeddings_response(request: &RecordedRequest) -> Value {
        let inputs = request.body["input"].as_array().unwrap();
        let data: Vec<Value> = inputs
            .iter()
            .enumerate()
            .rev()
            .map(|(i, input)| {
                let chars = input.as_str().unwrap().chars().count() as f32;
                json!({ "index": i, "embedding": [chars, i as f32] })
            })
            .collect();
        json!({ "data": data })
    }

    async fn adapter_for(server: &MockServer, max_retries: u32) -> OpenAIAdapter {
        let config = OpenAIConfig {
            api_key: "test-key".to_string(),
            model: "text-embedding-3-small".to_string(),
            base_url: Some(format!("{}/v1", server.base_url)),
            timeout_seconds: 5,
            max_retries,
        };
        OpenAIAdapter::new(config)
            .await
            .unwrap()
            .with_retry_base_delay(Duration::from_millis(1))
    }

    #[tokio::test]
    async fn test_empty_api_key_rejected() {
        let result = OpenAIAdapter::new(OpenAIConfig::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_single_embedding() {
        let server = MockServer::start(|req, _| (StatusCode::OK, embeddings_response(req))).await;
        let adapter = adapter_for(&server, 0).await;

        let embedding = adapter.generate_embedding("hello").await.unwrap();
        assert_eq!(embedding, vec![5.0, 0.0]);

        let requests = server.requests_to(Method::POST, "/v1/embeddings");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].header("authorization"), Some("Bearer test-key"));
        assert_eq!(requests[0].body["model"], "text-embedding-3-small");
        assert_eq!(requests[0].body["input"], json!(["hello"]));
    }

    #[tokio::test]
    async fn test_batch_order_follows_index() {
        let server = MockServer::start(|req, _| (StatusCode::OK, embeddings_response(req))).await;
        let adapter = adapter_for(&server, 0).await;

        // The mock returns data in reverse order
        let embeddings = adapter
            .generate_batch_embeddings(vec!["a", "bb", "ccc"])
            .await
            .unwrap();
        assert_eq!(
            embeddings,
            vec![vec![1.0, 0.0], vec![2.0, 1.0], vec![3.0, 2.0]]
        );
    }

    #[tokio::test]
    async fn test_large_batch_is_split() {
        let server = MockServer::start(|req, _| (StatusCode::OK, embeddings_response(req))).await;
        let adapter = adapter_for(&server, 0).await;

        let texts: Vec<String> = (0..250).map(|i| "x".repeat(i + 1)).collect();
        let embeddings = adapter
            .generate_batch_embeddings(texts.iter().map(String::as_str).collect())
            .await
            .unwrap();

        assert_eq!(embeddings.len(), 250);
        for (i, embedding) in embeddings.iter().enumerate() {
            assert_eq!(embedding[0], (i + 1) as f32);
        }
        let sizes: Vec<usize> = server
            .requests_to(Method::POST, "/v1/embeddings")
            .iter()
            .map(|r| r.body["input"].as_array().unwrap().len())
            .collect();
        assert_eq!(sizes, vec![100, 100, 50]);
    }

    #[tokio::test]
    async fn test_count_mismatch_is_error() {
        let server = MockServer::start(|_, _| {
            (
                StatusCode::OK,
                json!({ "data": [{ "index": 0, "embedding": [1.0] }] }),
            )
        })
        .await;
        let adapter = adapter_for(&server, 0).await;

        let result = adapter.generate_batch_embeddings(vec!["a", "b"]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_is_retried() {
        let server = MockServer::start(|req, seen| {
            if seen == 0 {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    json!({ "error": { "message": "Rate limit reached" } }),
                )
            } else {
                (StatusCode::OK, embeddings_response(req))
            }
        })
        .await;
        let adapter = adapter_for(&server, 3).await;

        let embedding = adapter.generate_embedding("hello").await.unwrap();
        assert_eq!(embedding, vec![5.0, 0.0]);
        assert_eq!(server.requests_to(Method::POST, "/v1/embeddings").len(), 2);
    }

    #[tokio::test]
    async fn test_retries_exhausted() {
        let server = MockServer::start(|_, _| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                json!({ "error": { "message": "down" } }),
            )
        })
        .await;
        let adapter = adapter_for(&server, 2).await;

        let error = adapter.generate_embedding("hello").await.unwrap_err();
        assert!(error.to_string().contains("503"), "{}", error);
        assert_eq!(server.requests_to(Method::POST, "/v1/embeddings").len(), 3);
    }

    #[tokio::test]
    async fn test_unauthorized_is_not_retried() {
        let server = MockServer::start(|_, _| {
            (
                StatusCode::UNAUTHORIZED,
                json!({ "error": { "message": "Incorrect API key provided" } }),
            )
        })
        .await;
        let adapter = adapter_for(&server, 3).await;

        let error = adapter.generate_embedding("hello").await.unwrap_err();
        let message = error.to_string();
        assert!(message.contains("401"), "{}", message);
        assert!(message.contains("Incorrect API key"), "{}", message);
        assert_eq!(server.requests_to(Method::POST, "/v1/embeddings").len(), 1);
    }

    #[tokio::test]
    async fn test_multibyte_text_is_truncated_safely() {
        let server = MockServer::start(|req, _| (StatusCode::OK, embeddings_response(req))).await;
        let adapter = adapter_for(&server, 0).await;

        let long_text = "é".repeat(MAX_CHARS + 500);
        adapter.generate_embedding(&long_text).await.unwrap();

        let requests = server.requests_to(Method::POST, "/v1/embeddings");
        let sent = requests[0].body["input"][0].as_str().unwrap();
        assert_eq!(sent.chars().count(), MAX_CHARS + 3);
        assert!(sent.ends_with("..."));
    }

    #[tokio::test]
    async fn test_short_text_unchanged() {
        let server = MockServer::start(|req, _| (StatusCode::OK, embeddings_response(req))).await;
        let adapter = adapter_for(&server, 0).await;

        assert_eq!(adapter.prepare_text("Short text"), "Short text");
    }
}
