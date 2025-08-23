/// Search domain logic for Zero-Latency
/// 
/// This crate provides reusable search capabilities including:
/// - Query processing and enhancement
/// - Result ranking and scoring
/// - Search orchestration patterns
/// - Search analytics and metrics

pub mod models;
pub mod services;
pub mod traits;
pub mod vector_search;

pub use models::*;
pub use services::*;
pub use traits::*;
pub use vector_search::*;
