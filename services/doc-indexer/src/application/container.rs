/// Dependency injection container for doc-indexer service
/// 
/// This module implements the Dependency Inversion Principle by providing
/// a centralized container that manages the creation and lifecycle of all
/// service dependencies.

use std::sync::Arc;
use zero_latency_core::{Result, ZeroLatencyError};
use zero_latency_search::{SearchOrchestrator, SimpleSearchOrchestrator, SearchPipeline, QueryEnhancementStep, ResultRankingStep};
use zero_latency_vector::{VectorRepository, EmbeddingGenerator};

use crate::config::Config;
use crate::infrastructure::search_enhancement::{SimpleQueryEnhancer, MultiFactorResultRanker};

/// Central dependency injection container for the doc-indexer service
pub struct ServiceContainer {
    // Core services
    search_orchestrator: Arc<dyn SearchOrchestrator>,
    
    // Infrastructure services  
    vector_repository: Arc<dyn VectorRepository>,
    embedding_generator: Arc<dyn EmbeddingGenerator>,
    
    // Configuration
    config: Arc<Config>,
}

impl ServiceContainer {
    /// Create a new service container with all dependencies initialized
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        
        // Create infrastructure services based on configuration
        let vector_repository = Self::create_vector_repository(&config).await?;
        let embedding_generator = Self::create_embedding_generator(&config).await?;
        
        // Create search pipeline and orchestrator
        let search_pipeline = Self::create_search_pipeline(
            vector_repository.clone(),
            embedding_generator.clone(),
        ).await?;
        
        let search_orchestrator = Arc::new(SimpleSearchOrchestrator::new(search_pipeline));
        
        Ok(Self {
            search_orchestrator,
            vector_repository,
            embedding_generator,
            config,
        })
    }
    
    /// Get the search orchestrator
    pub fn search_orchestrator(&self) -> Arc<dyn SearchOrchestrator> {
        self.search_orchestrator.clone()
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
    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }
    
    /// Create vector repository based on configuration
    async fn create_vector_repository(config: &Config) -> Result<Arc<dyn VectorRepository>> {
        use crate::infrastructure::InMemoryVectorStore;
        use crate::config::VectorBackend;
        
        #[cfg(feature = "cloud")]
        use crate::infrastructure::QdrantAdapter;
        
        #[cfg(feature = "embedded")]
        use crate::infrastructure::EmbeddedVectorStore;
        
        match config.vector.backend {
            VectorBackend::Memory => {
                Ok(Arc::new(InMemoryVectorStore::new()))
            }
            #[cfg(feature = "cloud")]
            VectorBackend::Qdrant => {
                let adapter = QdrantAdapter::new(config.vector.qdrant.clone()).await?;
                Ok(Arc::new(adapter))
            }
            #[cfg(not(feature = "cloud"))]
            VectorBackend::Qdrant => {
                Err(ZeroLatencyError::Configuration {
                    message: "Qdrant backend requires 'cloud' feature to be enabled".to_string()
                })
            }
            #[cfg(feature = "embedded")]
            VectorBackend::Embedded => {
                let adapter = EmbeddedVectorStore::new(config.vector.embedded.clone()).await?;
                Ok(Arc::new(adapter))
            }
            #[cfg(not(feature = "embedded"))]
            VectorBackend::Embedded => {
                Err(ZeroLatencyError::Configuration {
                    message: "Embedded backend requires 'embedded' feature to be enabled".to_string()
                })
            }
        }
    }
    
    /// Create embedding generator based on configuration
    async fn create_embedding_generator(config: &Config) -> Result<Arc<dyn EmbeddingGenerator>> {
        use crate::config::EmbeddingProvider;
        
        #[cfg(feature = "embedded")]
        use crate::infrastructure::LocalEmbeddingAdapter;
        
        #[cfg(feature = "cloud")]
        use crate::infrastructure::OpenAIAdapter;
        
        match config.embedding.provider {
            #[cfg(feature = "embedded")]
            EmbeddingProvider::Local => {
                let adapter = LocalEmbeddingAdapter::new(config.embedding.local.clone())?;
                Ok(Arc::new(adapter))
            }
            #[cfg(not(feature = "embedded"))]
            EmbeddingProvider::Local => {
                Err(ZeroLatencyError::Configuration {
                    message: "Local embedding provider requires 'embedded' feature to be enabled".to_string()
                })
            }
            #[cfg(feature = "cloud")]
            EmbeddingProvider::OpenAI => {
                let adapter = OpenAIAdapter::new(config.embedding.openai.clone()).await?;
                Ok(Arc::new(adapter))
            }
            #[cfg(not(feature = "cloud"))]
            EmbeddingProvider::OpenAI => {
                Err(ZeroLatencyError::Configuration {
                    message: "OpenAI embedding provider requires 'cloud' feature to be enabled".to_string()
                })
            }
        }
    }
    
    /// Create search pipeline with all steps
    async fn create_search_pipeline(
        vector_repository: Arc<dyn VectorRepository>,
        embedding_generator: Arc<dyn EmbeddingGenerator>,
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
        let query_enhancement_step = Box::new(QueryEnhancementStep::new(query_enhancer));
        
        let vector_search_step = Box::new(zero_latency_search::VectorSearchStep::new(
            vector_repository,
            embedding_service,
        ));
        
        let result_ranking_step = Box::new(ResultRankingStep::new(result_ranker));
        
        // Build the enhanced pipeline: Query Enhancement → Vector Search → Result Ranking
        let pipeline = SearchPipeline::builder()
            .add_step(query_enhancement_step)
            .add_step(vector_search_step)
            .add_step(result_ranking_step)
            .build();
            
        Ok(pipeline)
    }
}

/// Builder for service container to support testing and configuration flexibility
pub struct ServiceContainerBuilder {
    config: Option<Config>,
    vector_repository: Option<Arc<dyn VectorRepository>>,
    embedding_generator: Option<Arc<dyn EmbeddingGenerator>>,
}

impl ServiceContainerBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            vector_repository: None,
            embedding_generator: None,
        }
    }
    
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn with_vector_repository(mut self, repo: Arc<dyn VectorRepository>) -> Self {
        self.vector_repository = Some(repo);
        self
    }
    
    pub fn with_embedding_generator(mut self, generator: Arc<dyn EmbeddingGenerator>) -> Self {
        self.embedding_generator = Some(generator);
        self
    }
    
    pub async fn build(self) -> Result<ServiceContainer> {
        let config = self.config.ok_or_else(|| {
            ZeroLatencyError::configuration("Config is required")
        })?;
        
        // Use provided services or create defaults
        if let (Some(vector_repo), Some(embedding_gen)) = (self.vector_repository, self.embedding_generator) {
            let config = Arc::new(config);
            let search_pipeline = ServiceContainer::create_search_pipeline(
                vector_repo.clone(),
                embedding_gen.clone(),
            ).await?;
            
            Ok(ServiceContainer {
                search_orchestrator: Arc::new(SimpleSearchOrchestrator::new(search_pipeline)),
                vector_repository: vector_repo,
                embedding_generator: embedding_gen,
                config,
            })
        } else {
            ServiceContainer::new(config).await
        }
    }
}

impl Default for ServiceContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
