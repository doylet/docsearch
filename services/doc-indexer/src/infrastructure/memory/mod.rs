pub mod benchmark;
pub mod cache;
pub mod intern;
/// Memory optimization utilities
///
/// This module provides memory pooling, string interning, and other
/// optimization utilities to reduce memory allocations and improve
/// performance.
pub mod pool;

pub use cache::{CacheConfig, MemoryEfficientCache};
pub use intern::StringInterner;
pub use pool::{PooledVector, VectorPool, VectorPoolConfig};
