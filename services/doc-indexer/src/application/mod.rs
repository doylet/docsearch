/// Application layer modules
///
/// This layer contains the use case implementations and application services.
/// It coordinates between the domain layer and infrastructure layer.
pub mod container;
pub mod concurrent_container;
pub mod content_processing;
pub mod content_processor;
pub mod services;

// SOLID-compliant modules (Phase 2)
pub mod adapters;
pub mod indexing_strategies;
pub mod interfaces;

// Re-export the old interface for backward compatibility during transition

// Export new SOLID-compliant interfaces
pub use content_processing::ContentProcessor;

// Re-export all services and container for easy access
pub use container::ServiceContainer;
pub use concurrent_container::ConcurrentServiceContainer;
pub use services::{
    collection_service::CollectionService, document_service::DocumentIndexingService,
    health_service::HealthService,
};

// Export SOLID-compliant services
