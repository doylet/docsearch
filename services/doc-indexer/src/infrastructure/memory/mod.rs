/// Memory optimization utilities
/// 
/// This module provides memory pooling, string interning, and other
/// optimization utilities to reduce memory allocations and improve
/// performance.

pub mod pool;
pub mod intern;
pub mod cache;
pub mod benchmark;

pub use pool::{VectorPool, VectorPoolConfig, PooledVector};
pub use intern::{StringInterner, InternedString};
pub use cache::{MemoryEfficientCache, CacheConfig};
pub use benchmark::{Benchmark, BenchmarkResult};
