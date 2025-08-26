/// Content processing module with SOLID compliance
///
/// This module implements content processing using SOLID principles:
/// - Single Responsibility: Each handler has one reason to change
/// - Open-Closed: Extensible for new content types without modification
/// - Interface Segregation: Focused traits for specific concerns
/// - Dependency Inversion: Depends on abstractions, not concretions
pub mod content_type;
pub mod detector;
pub mod handlers;
pub mod processor;
pub mod registry;

pub use content_type::ContentType;
pub use detector::ContentTypeDetector;
pub use handlers::ContentHandler;
pub use processor::ContentProcessor;
pub use registry::ContentProcessorRegistry;
