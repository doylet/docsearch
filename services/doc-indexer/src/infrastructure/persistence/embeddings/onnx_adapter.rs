/// Local semantic embeddings: bge-small-en-v1.5 on ONNX Runtime
///
/// Tokenize (truncate at 512, pad to the longest in the batch), run the model,
/// take the CLS vector of `last_hidden_state`, L2-normalise. Queries get bge's
/// retrieval instruction prefix; documents don't. Inference runs on the blocking
/// pool, one batch at a time through a single shared session.
///
/// All `ort` calls live in this file: its API changes between release candidates.
use async_trait::async_trait;
use ort::session::{builder::GraphOptimizationLevel, Session, SessionInputValue};
use ort::value::Tensor;
use std::borrow::Cow;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};
use zero_latency_core::{Result, ZeroLatencyError};
use zero_latency_vector::EmbeddingGenerator;

use super::model_files::ModelStore;

/// bge-small-en-v1.5 output size
pub const BGE_DIMENSION: usize = 384;

/// Instruction bge models are trained to see in front of retrieval queries
pub const BGE_QUERY_PREFIX: &str = "Represent this sentence for searching relevant passages: ";

const MAX_TOKENS: usize = 512;
const MAX_BATCH: usize = 32;

struct Model {
    tokenizer: Tokenizer,
    session: Mutex<Session>,
    /// Whether the graph takes `token_type_ids` (BERT exports usually do)
    wants_token_type_ids: bool,
    output_name: String,
}

pub struct OnnxEmbeddingAdapter {
    model: Arc<Model>,
    model_id: String,
}

impl OnnxEmbeddingAdapter {
    /// Make sure the model files are present and verified, then load them
    pub async fn from_store(store: &ModelStore) -> Result<Self> {
        let started = std::time::Instant::now();
        let dir = store.ensure().await?;
        tracing::info!(
            "Model files in {} verified in {} ms",
            dir.display(),
            started.elapsed().as_millis()
        );
        let model_id = format!("{}@{}", store.spec().name, store.spec().revision);
        let started = std::time::Instant::now();
        let adapter = tokio::task::spawn_blocking(move || Self::load(&dir, model_id))
            .await
            .map_err(|e| ZeroLatencyError::internal(format!("Model load task failed: {}", e)))??;
        tracing::info!(
            "Loaded local embedding model {} in {} ms",
            adapter.model_id,
            started.elapsed().as_millis()
        );
        Ok(adapter)
    }

    /// Load `model.onnx` and `tokenizer.json` from `dir`. Callers must have
    /// verified the files; `from_store` does.
    pub fn load(dir: &Path, model_id: String) -> Result<Self> {
        let mut tokenizer = Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| {
            ZeroLatencyError::configuration(format!(
                "Cannot load {}: {}",
                dir.join("tokenizer.json").display(),
                e
            ))
        })?;
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: MAX_TOKENS,
                ..Default::default()
            }))
            .map_err(|e| ZeroLatencyError::configuration(format!("Tokenizer truncation: {}", e)))?;
        tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            ..Default::default()
        }));

        let session = Session::builder()
            .and_then(|b| b.with_optimization_level(GraphOptimizationLevel::Level3))
            .and_then(|b| b.commit_from_file(dir.join("model.onnx")))
            .map_err(|e| {
                ZeroLatencyError::configuration(format!(
                    "Cannot load {}: {}",
                    dir.join("model.onnx").display(),
                    e
                ))
            })?;

        let wants_token_type_ids = session.inputs.iter().any(|i| i.name == "token_type_ids");
        let output_name = session
            .outputs
            .iter()
            .find(|o| o.name == "last_hidden_state")
            .or_else(|| session.outputs.first())
            .map(|o| o.name.clone())
            .ok_or_else(|| ZeroLatencyError::configuration("ONNX model has no outputs"))?;

        Ok(Self {
            model: Arc::new(Model {
                tokenizer,
                session: Mutex::new(session),
                wants_token_type_ids,
                output_name,
            }),
            model_id,
        })
    }

    /// Embed texts in groups of `MAX_BATCH`, each on the blocking pool
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.iter().any(|t| t.is_empty()) {
            return Err(ZeroLatencyError::validation("text", "Text cannot be empty"));
        }
        let mut embeddings = Vec::with_capacity(texts.len());
        for group in texts.chunks(MAX_BATCH) {
            let model = self.model.clone();
            let group = group.to_vec();
            let vectors = tokio::task::spawn_blocking(move || model.embed_blocking(group))
                .await
                .map_err(|e| {
                    ZeroLatencyError::internal(format!("Embedding task failed: {}", e))
                })??;
            embeddings.extend(vectors);
        }
        Ok(embeddings)
    }
}

impl Model {
    fn embed_blocking(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let encodings = self
            .tokenizer
            .encode_batch(texts, true)
            .map_err(|e| ZeroLatencyError::internal(format!("Tokenization failed: {}", e)))?;
        let batch = encodings.len();
        let seq_len = encodings.first().map(|e| e.get_ids().len()).unwrap_or(0);

        let flatten = |field: fn(&tokenizers::Encoding) -> &[u32]| -> Vec<i64> {
            encodings
                .iter()
                .flat_map(|e| field(e).iter().map(|&v| v as i64))
                .collect()
        };
        let shape = vec![batch as i64, seq_len as i64];
        let tensor = |data: Vec<i64>| -> Result<SessionInputValue<'static>> {
            Tensor::from_array((shape.clone(), data))
                .map(Into::into)
                .map_err(|e| ZeroLatencyError::internal(format!("Tensor creation failed: {}", e)))
        };

        let mut inputs: Vec<(Cow<'static, str>, SessionInputValue<'static>)> = vec![
            ("input_ids".into(), tensor(flatten(|e| e.get_ids()))?),
            (
                "attention_mask".into(),
                tensor(flatten(|e| e.get_attention_mask()))?,
            ),
        ];
        if self.wants_token_type_ids {
            inputs.push(("token_type_ids".into(), tensor(vec![0; batch * seq_len])?));
        }

        let mut session = self
            .session
            .lock()
            .map_err(|_| ZeroLatencyError::internal("Embedding session lock poisoned"))?;
        let outputs = session
            .run(inputs)
            .map_err(|e| ZeroLatencyError::internal(format!("Model inference failed: {}", e)))?;
        let (out_shape, data) = outputs[self.output_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| ZeroLatencyError::internal(format!("Unexpected model output: {}", e)))?;

        // last_hidden_state: [batch, seq_len, hidden]
        if out_shape.len() != 3
            || out_shape[0] as usize != batch
            || out_shape[2] as usize != BGE_DIMENSION
        {
            return Err(ZeroLatencyError::internal(format!(
                "Unexpected model output shape {:?}",
                &out_shape[..]
            )));
        }
        let row = out_shape[1] as usize * BGE_DIMENSION;

        Ok((0..batch)
            .map(|b| {
                let mut cls = data[b * row..b * row + BGE_DIMENSION].to_vec();
                l2_normalise(&mut cls);
                cls
            })
            .collect())
    }
}

fn l2_normalise(vector: &mut [f32]) {
    let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        vector.iter_mut().for_each(|x| *x /= norm);
    }
}

#[async_trait]
impl EmbeddingGenerator for OnnxEmbeddingAdapter {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let mut vectors = self.embed(vec![text.to_string()]).await?;
        Ok(vectors.remove(0))
    }

    async fn generate_batch_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        self.embed(texts.into_iter().map(str::to_string).collect())
            .await
    }

    async fn generate_query_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if text.is_empty() {
            return Err(ZeroLatencyError::validation("text", "Text cannot be empty"));
        }
        self.generate_embedding(&format!("{}{}", BGE_QUERY_PREFIX, text))
            .await
    }

    fn dimension(&self) -> usize {
        BGE_DIMENSION
    }

    fn model_name(&self) -> &str {
        "bge-small-en-v1.5"
    }

    fn model_id(&self) -> String {
        self.model_id.clone()
    }
}

/// Real-model tests. Run with the model on disk:
/// `ZL_EMBEDDING_LOCAL_MODEL_PATH=<dir> cargo test -p doc-indexer -- --ignored onnx`
#[cfg(test)]
mod onnx_tests {
    use super::*;

    fn load() -> Option<OnnxEmbeddingAdapter> {
        let Some(dir) = std::env::var_os("ZL_EMBEDDING_LOCAL_MODEL_PATH") else {
            eprintln!("skipping: ZL_EMBEDDING_LOCAL_MODEL_PATH is not set (run `doc-indexer --fetch-model`)");
            return None;
        };
        Some(
            OnnxEmbeddingAdapter::load(Path::new(&dir), "bge-small-en-v1.5@test".to_string())
                .unwrap(),
        )
    }

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    const KUBERNETES: &str = "Kubernetes Deployments support rolling updates: new pods are started and old pods terminated gradually, so the application stays available while a new version is rolled out.";
    const SOURDOUGH: &str = "To bake sourdough bread, feed your starter the night before, mix flour, water and salt, then let the dough ferment slowly before shaping and baking in a hot Dutch oven.";

    #[tokio::test]
    #[ignore = "needs the bge-small-en-v1.5 model; set ZL_EMBEDDING_LOCAL_MODEL_PATH"]
    async fn onnx_related_text_ranks_higher() {
        let Some(adapter) = load() else { return };
        let query = adapter
            .generate_query_embedding("How do I deploy with rolling updates?")
            .await
            .unwrap();
        let docs = adapter
            .generate_batch_embeddings(vec![KUBERNETES, SOURDOUGH])
            .await
            .unwrap();
        let (k8s, bread) = (cosine(&query, &docs[0]), cosine(&query, &docs[1]));
        assert!(k8s > bread, "kubernetes {} vs sourdough {}", k8s, bread);
    }

    #[tokio::test]
    #[ignore = "needs the bge-small-en-v1.5 model; set ZL_EMBEDDING_LOCAL_MODEL_PATH"]
    async fn onnx_shape_and_normalisation() {
        let Some(adapter) = load() else { return };
        for text in ["a", KUBERNETES, &"long text ".repeat(400)] {
            let v = adapter.generate_embedding(text).await.unwrap();
            assert_eq!(v.len(), 384);
            let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.001, "norm {}", norm);
        }
    }

    #[test]
    #[ignore = "needs the bge-small-en-v1.5 model; set ZL_EMBEDDING_LOCAL_MODEL_PATH"]
    fn onnx_graph_signature() {
        let Some(adapter) = load() else { return };
        let session = adapter.model.session.lock().unwrap();
        let names = |values: Vec<String>| values.join(", ");
        let inputs = names(
            session
                .inputs
                .iter()
                .map(|i| format!("{}: {:?}", i.name, i.input_type))
                .collect(),
        );
        let outputs = names(
            session
                .outputs
                .iter()
                .map(|o| format!("{}: {:?}", o.name, o.output_type))
                .collect(),
        );
        eprintln!("inputs: {}\noutputs: {}", inputs, outputs);
        assert!(adapter.model.wants_token_type_ids, "{}", inputs);
        assert_eq!(
            adapter.model.output_name, "last_hidden_state",
            "{}",
            outputs
        );
    }

    #[tokio::test]
    #[ignore = "needs the bge-small-en-v1.5 model; set ZL_EMBEDDING_LOCAL_MODEL_PATH"]
    async fn onnx_deterministic() {
        let Some(adapter) = load() else { return };
        let a = adapter.generate_embedding(KUBERNETES).await.unwrap();
        let b = adapter.generate_embedding(KUBERNETES).await.unwrap();
        assert_eq!(a, b);
    }

    #[tokio::test]
    #[ignore = "needs the bge-small-en-v1.5 model; set ZL_EMBEDDING_LOCAL_MODEL_PATH"]
    async fn onnx_batch_equals_single() {
        let Some(adapter) = load() else { return };
        let texts = ["short", KUBERNETES, SOURDOUGH];
        let batch = adapter
            .generate_batch_embeddings(texts.to_vec())
            .await
            .unwrap();
        for (text, from_batch) in texts.iter().zip(&batch) {
            let single = adapter.generate_embedding(text).await.unwrap();
            for (x, y) in single.iter().zip(from_batch) {
                assert!((x - y).abs() < 1e-5, "{}: {} vs {}", text, x, y);
            }
        }
    }

    #[tokio::test]
    #[ignore = "needs the bge-small-en-v1.5 model; set ZL_EMBEDDING_LOCAL_MODEL_PATH"]
    async fn onnx_query_prefix_changes_vector() {
        let Some(adapter) = load() else { return };
        let text = "How do I deploy with rolling updates?";
        let as_query = adapter.generate_query_embedding(text).await.unwrap();
        let as_doc = adapter.generate_embedding(text).await.unwrap();
        assert_ne!(as_query, as_doc);
    }
}
