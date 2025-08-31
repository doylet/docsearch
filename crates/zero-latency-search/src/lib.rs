/// Search domain logic for Zero-Latency
///
/// This crate provides reusable search capabilities including:
/// - Query processing and enhancement
/// - Result ranking and scoring
/// - Search orchestration patterns
/// - Search analytics and metrics
/// - Performance optimization and caching
pub mod bm25;
pub mod cache;
pub mod evaluation;
pub mod fusion;
pub mod hybrid;
pub mod models;
pub mod pipeline;
pub mod query_expansion;
// pub mod examples;
pub mod services;
pub mod traits;
pub mod vector_search;

pub use bm25::*;
pub use evaluation::*;
pub use fusion::*;
pub use hybrid::*;
pub use models::*;
pub use pipeline::*;
pub use query_expansion::*;
pub use services::*;
pub use traits::*;
pub use vector_search::*;
