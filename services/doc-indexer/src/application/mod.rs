/// Application layer modules
/// 
/// This module contains the application services and dependency injection
/// container that coordinate business logic and use cases.

pub mod container;
pub mod services;

// Re-export commonly used types
pub use container::ServiceContainer;
pub use services::{
    document_service::DocumentIndexingService,
    health_service::HealthService,
};
