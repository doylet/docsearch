/// Application layer modules
/// 
/// This layer contains the use case implementations and application services.
/// It coordinates between the domain layer and infrastructure layer.

pub mod container;
pub mod content_processor;
pub mod content_processing;
pub mod services;

// SOLID-compliant modules (Phase 2)
pub mod interfaces;
pub mod adapters;
pub mod indexing_service;
pub mod indexing_strategies;

// Re-export the old interface for backward compatibility during transition
pub use content_processor::{ContentProcessor as LegacyContentProcessor, ContentType as LegacyContentType};

// Export new SOLID-compliant interfaces
pub use content_processing::{ContentProcessor, ContentType, ContentTypeDetector, ContentProcessorRegistry};

// Re-export all services and container for easy access
pub use container::ServiceContainer;
pub use services::{
    collection_service::CollectionService,
    document_service::DocumentIndexingService,
    health_service::HealthService,
};

// Export SOLID-compliant services
pub use services::indexing_service::{IndexingService, IndexingServiceBuilder};
pub use interfaces::*;
pub use adapters::*;
pub use indexing_strategies::*;
