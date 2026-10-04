/// Embedding wiring shared by both service containers
///
/// Builds the configured embedding generator, adapts it for query embedding in
/// the search pipeline, and checks that an embedded store's vectors were written
/// by the active model.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zero_latency_core::Result;
#[cfg(any(not(feature = "embedded"), not(feature = "cloud")))]
use zero_latency_core::ZeroLatencyError;
use zero_latency_vector::EmbeddingGenerator;

use crate::config::{Config, EmbeddingProvider};

/// Index state shared between startup, indexing and `/api/status`
#[derive(Debug, Default)]
pub struct IndexState {
    reindex_required: AtomicBool,
}

impl IndexState {
    /// True after startup discarded vectors written by a different model,
    /// until something is indexed again
    pub fn reindex_required(&self) -> bool {
        self.reindex_required.load(Ordering::Relaxed)
    }

    pub fn set_reindex_required(&self, value: bool) {
        self.reindex_required.store(value, Ordering::Relaxed);
    }
}

/// Build the embedding generator selected by `embedding.provider`
pub(crate) async fn create_embedding_generator(
    config: &Config,
) -> Result<Arc<dyn EmbeddingGenerator>> {
    match config.embedding.provider {
        #[cfg(feature = "embedded")]
        EmbeddingProvider::Local => {
            use crate::infrastructure::{ModelSpec, ModelStore, OnnxEmbeddingAdapter};

            let local = &config.embedding.local;
            let store = ModelStore::new(
                ModelSpec::bge_small_en_v1_5(),
                local.model_path.clone(),
                local.model_url.clone(),
            )?;
            Ok(Arc::new(OnnxEmbeddingAdapter::from_store(&store).await?))
        }
        #[cfg(feature = "embedded")]
        EmbeddingProvider::Hash => {
            use crate::infrastructure::{HashEmbeddingAdapter, HashEmbeddingConfig};

            let local = &config.embedding.local;
            let adapter = HashEmbeddingAdapter::new(HashEmbeddingConfig {
                dimension: local.dimension,
                seed: local.seed,
                enable_vector_pooling: local.enable_vector_pooling,
            })?;
            Ok(Arc::new(adapter))
        }
        #[cfg(not(feature = "embedded"))]
        EmbeddingProvider::Local | EmbeddingProvider::Hash => Err(ZeroLatencyError::configuration(
            "Local and hash embedding providers require the 'embedded' feature",
        )),
        #[cfg(feature = "cloud")]
        EmbeddingProvider::OpenAI => {
            let adapter =
                crate::infrastructure::OpenAIAdapter::new(config.embedding.openai.clone()).await?;
            Ok(Arc::new(adapter))
        }
        #[cfg(not(feature = "cloud"))]
        EmbeddingProvider::OpenAI => Err(ZeroLatencyError::configuration(
            "OpenAI embedding provider requires the 'cloud' feature",
        )),
    }
}

/// Search pipeline adapter: embeds queries with `generate_query_embedding`
pub(crate) struct QueryEmbeddingService {
    generator: Arc<dyn EmbeddingGenerator>,
}

impl QueryEmbeddingService {
    pub(crate) fn new(generator: Arc<dyn EmbeddingGenerator>) -> Self {
        Self { generator }
    }
}

#[async_trait::async_trait]
impl zero_latency_search::EmbeddingService for QueryEmbeddingService {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.generator.generate_query_embedding(text).await
    }
}

/// Discard an embedded store's vectors if another model wrote them.
/// Returns true when vectors were removed and a re-index is needed.
#[cfg(feature = "embedded")]
pub(crate) async fn reconcile_embedded_store(
    store: &crate::infrastructure::EmbeddedVectorStore,
    generator: &dyn EmbeddingGenerator,
) -> Result<bool> {
    let model_id = generator.model_id();
    let check = store
        .reconcile_model(&model_id, generator.dimension())
        .await?;
    if check.removed > 0 {
        tracing::warn!(
            "Removed {} vectors written by {} because the active embedding model is {}. Search returns nothing until you re-index (`mdx reindex`, `mdx index <path>` or POST /api/index); source documents are untouched",
            check.removed,
            check.previous_model.as_deref().unwrap_or("an older version (no model recorded)"),
            model_id
        );
    }
    Ok(check.removed > 0)
}
