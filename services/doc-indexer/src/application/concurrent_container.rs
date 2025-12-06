/// Enhanced dependency injection container with concurrent search capabilities
///
/// This module extends the base ServiceContainer to provide thread-safe, non-blocking
/// search operations that don't interfere with indexing operations.
use std::sync::Arc;
use zero_latency_core::{Result, ZeroLatencyError};
use zero_latency_search::{
    SearchPipeline, SimpleSearchOrchestrator,
};
use zero_latency_vector::{EmbeddingGenerator, VectorRepository};

use crate::config::{Config, VectorBackend, EmbeddingProvider};
use crate::infrastructure::search_enhancement::{MultiFactorResultRanker, SimpleQueryEnhancer};
use crate::infrastructure::concurrent_search::ConcurrentSearchService;

/// Enhanced service container with concurrent search capabilities
pub struct ConcurrentServiceContainer {
    // Core services
    concurrent_search_service: Arc<ConcurrentSearchService>,
    analytics: Arc<crate::infrastructure::operations::analytics::ProductionSearchAnalytics>,

    // Infrastructure services
    vector_repository: Arc<dyn VectorRepository>,
    embedding_generator: Arc<dyn EmbeddingGenerator>,

    // Configuration
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl ConcurrentServiceContainer {
    /// Create a new concurrent service container with all dependencies initialized
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);

        // Create infrastructure services based on configuration
        let vector_repository = Self::create_vector_repository(&config).await?;
        let embedding_generator = Self::create_embedding_generator(&config).await?;

        // Create analytics service first so it can be shared
        let analytics = Arc::new(
            crate::infrastructure::operations::analytics::ProductionSearchAnalytics::with_default_config(),
        );

        // Create search pipeline and orchestrator with shared analytics
        let search_pipeline = Self::create_search_pipeline(
            vector_repository.clone(),
            embedding_generator.clone(),
            analytics.clone(),
        )
        .await?;

        let search_orchestrator = Arc::new(SimpleSearchOrchestrator::new(search_pipeline));

        // Create concurrent search service wrapper
        let concurrent_search_service = Arc::new(
            ConcurrentSearchService::new(
                search_orchestrator,
                analytics.clone(),
            )
        );

        Ok(Self {
            concurrent_search_service,
            analytics,
            vector_repository,
            embedding_generator,
            config,
        })
    }

    /// Get the concurrent search service
    pub fn concurrent_search_service(&self) -> Arc<ConcurrentSearchService> {
        self.concurrent_search_service.clone()
    }

    /// Get the analytics service
    pub fn analytics(&self) -> Arc<crate::infrastructure::operations::analytics::ProductionSearchAnalytics> {
        self.analytics.clone()
    }

    /// Get the vector repository
    pub fn vector_repository(&self) -> Arc<dyn VectorRepository> {
        self.vector_repository.clone()
    }

    /// Get the embedding generator
    pub fn embedding_generator(&self) -> Arc<dyn EmbeddingGenerator> {
        self.embedding_generator.clone()
    }

    /// Get the configuration
    #[allow(dead_code)]
    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    /// Create vector repository based on configuration
    async fn create_vector_repository(config: &Config) -> Result<Arc<dyn VectorRepository>> {
        use crate::config::VectorBackend;
        use crate::infrastructure::InMemoryVectorStore;

        #[cfg(feature = "cloud")]
        use crate::infrastructure::QdrantAdapter;

        #[cfg(feature = "embedded")]
        use crate::infrastructure::EmbeddedVectorStore;

        match &config.vector.backend {
            VectorBackend::Memory => {
                let store = InMemoryVectorStore::new();
                Ok(Arc::new(store))
            }
            #[cfg(feature = "embedded")]
            VectorBackend::Embedded => {
                let embedded_config = crate::infrastructure::EmbeddedConfig {
                    db_path: config.vector.embedded.db_path.clone(),
                    dimension: config.vector.embedded.dimension,
                    cache_size: config.vector.embedded.cache_size,
                    enable_string_interning: true,
                    enable_smart_caching: true,
                };
                let store = EmbeddedVectorStore::new(embedded_config).await?;
                Ok(Arc::new(store))
            }
            #[cfg(feature = "cloud")]
            VectorBackend::Qdrant => {
                let qdrant_config = crate::infrastructure::QdrantConfig {
                    url: config.vector.qdrant.url.clone(),
                    api_key: config.vector.qdrant.api_key.clone(),
                    collection_name: config.vector.qdrant.collection_name.clone(),
                    timeout_seconds: config.vector.qdrant.timeout_seconds,
                    vector_size: 384, // Use default dimension
                };
                let adapter = QdrantAdapter::new(qdrant_config).await?;
                Ok(Arc::new(adapter))
            }
            #[cfg(not(feature = "embedded"))]
            VectorBackend::Embedded => {
                return Err(ZeroLatencyError::configuration(
                    "Embedded vector backend not available in this build",
                ));
            }
            #[cfg(not(feature = "cloud"))]
            VectorBackend::Qdrant => {
                return Err(ZeroLatencyError::configuration(
                    "Qdrant vector backend not available in this build",
                ));
            }
        }
    }

    /// Create embedding generator based on configuration
    async fn create_embedding_generator(config: &Config) -> Result<Arc<dyn EmbeddingGenerator>> {
        match &config.embedding.provider {
            EmbeddingProvider::Local => {
                // Use the existing local embedding adapter
                use crate::infrastructure::LocalEmbeddingAdapter;

                let local_config = crate::infrastructure::LocalEmbeddingConfig {
                    dimension: config.embedding.local.dimension,
                    seed: config.embedding.local.seed,
                    enable_vector_pooling: false,
                };
                let adapter = LocalEmbeddingAdapter::new(local_config)?;
                Ok(Arc::new(adapter))
            }
            EmbeddingProvider::OpenAI => {
                // Use OpenAI service when available
                #[cfg(feature = "cloud")]
                {
                    use crate::infrastructure::OpenAIEmbeddingService;

                    let service = OpenAIEmbeddingService::new(
                        config.embedding.openai.api_key.clone(),
                        config.embedding.openai.model.clone(),
                        config.embedding.openai.base_url.clone(),
                    );
                    Ok(Arc::new(service))
                }
                #[cfg(not(feature = "cloud"))]
                {
                    return Err(ZeroLatencyError::configuration(
                        "OpenAI embedding provider not available in this build",
                    ));
                }
            }
        }
    }

    /// Create search pipeline with all steps
    async fn create_search_pipeline(
        vector_repository: Arc<dyn VectorRepository>,
        embedding_generator: Arc<dyn EmbeddingGenerator>,
        analytics: Arc<crate::infrastructure::operations::analytics::ProductionSearchAnalytics>,
    ) -> Result<SearchPipeline> {
        // Create a simple embedding service adapter
        struct EmbeddingServiceAdapter {
            generator: Arc<dyn EmbeddingGenerator>,
        }

        #[async_trait::async_trait]
        impl zero_latency_search::EmbeddingService for EmbeddingServiceAdapter {
            async fn generate_embedding(&self, text: &str) -> zero_latency_core::Result<Vec<f32>> {
                self.generator.generate_embedding(text).await
            }
        }

        let embedding_service = Arc::new(EmbeddingServiceAdapter {
            generator: embedding_generator,
        });

        // Create enhanced search components
        let query_enhancer = Arc::new(SimpleQueryEnhancer::new());
        let result_ranker = Arc::new(MultiFactorResultRanker::new());

        // Create search steps
        let query_enhancement_step = Box::new(zero_latency_search::QueryEnhancementStep::new(query_enhancer));

        let vector_search_step = Box::new(zero_latency_search::VectorSearchStep::new(
            vector_repository,
            embedding_service,
        ));

        let result_ranking_step = Box::new(zero_latency_search::ResultRankingStep::new(result_ranker));

        // Build the enhanced pipeline: Query Enhancement → Vector Search → Result Ranking → Analytics
        let analytics_step = Box::new(zero_latency_search::services::AnalyticsStep::new(analytics));
        let pipeline = SearchPipeline::builder()
            .add_step(query_enhancement_step)
            .add_step(vector_search_step)
            .add_step(result_ranking_step)
            .add_step(analytics_step)
            .build();

        Ok(pipeline)
    }
}
