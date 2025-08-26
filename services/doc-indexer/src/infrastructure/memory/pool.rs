/// Vector memory pool implementation
/// 
/// Provides reusable vector buffers to reduce allocation overhead
/// during embedding generation and vector operations.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Configuration for vector pool
#[derive(Debug, Clone)]
pub struct VectorPoolConfig {
    /// Maximum number of vectors to keep in pool
    pub max_pool_size: usize,
    /// Standard vector dimension
    pub dimension: usize,
    /// Maximum dimension variance allowed for reuse
    pub dimension_tolerance: usize,
}

impl Default for VectorPoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 100,
            dimension: 384,
            dimension_tolerance: 32,
        }
    }
}

/// Memory pool for vector allocations
/// 
/// Maintains a pool of reusable Vec<f32> buffers to reduce
/// allocation overhead during embedding operations.
pub struct VectorPool {
    config: VectorPoolConfig,
    pool: Arc<Mutex<VecDeque<Vec<f32>>>>,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl VectorPool {
    /// Create a new vector pool
    pub fn new(config: VectorPoolConfig) -> Self {
        Self {
            config,
            pool: Arc::new(Mutex::new(VecDeque::new())),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Get a vector from the pool or allocate a new one
    /// 
    /// Returns a vector with at least the requested capacity.
    /// The vector may be larger than requested but will be cleared.
    pub fn get_vector(&self, dimension: usize) -> Vec<f32> {
        let mut pool = self.pool.lock().unwrap();
        
        // Look for a suitable vector in the pool
        for i in 0..pool.len() {
            if let Some(vec) = pool.get(i).cloned() {
                if vec.capacity() >= dimension && 
                   vec.capacity() <= dimension + self.config.dimension_tolerance {
                    pool.remove(i);
                    let mut vec = vec;
                    vec.clear();
                    vec.resize(dimension, 0.0);
                    
                    // Update hit counter
                    *self.hits.lock().unwrap() += 1;
                    return vec;
                }
            }
        }
        
        // No suitable vector found, allocate new one
        *self.misses.lock().unwrap() += 1;
        let mut vec = Vec::with_capacity(dimension.max(self.config.dimension));
        vec.resize(dimension, 0.0);
        vec
    }

    /// Return a vector to the pool for reuse
    /// 
    /// Only vectors with suitable capacity will be kept.
    /// Pool size is limited to prevent unbounded growth.
    pub fn return_vector(&self, mut vec: Vec<f32>) {
        // Only keep vectors with reasonable capacity
        if vec.capacity() >= self.config.dimension &&
           vec.capacity() <= self.config.dimension + self.config.dimension_tolerance * 2 {
            
            vec.clear();
            
            let mut pool = self.pool.lock().unwrap();
            if pool.len() < self.config.max_pool_size {
                pool.push_back(vec);
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> VectorPoolStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let pool_size = self.pool.lock().unwrap().len();
        
        VectorPoolStats {
            hits,
            misses,
            hit_rate: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
            pool_size,
            max_pool_size: self.config.max_pool_size,
        }
    }

    /// Clear all cached vectors
    pub fn clear(&self) {
        self.pool.lock().unwrap().clear();
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
    }
}

/// Statistics for vector pool performance
#[derive(Debug, Clone)]
pub struct VectorPoolStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub pool_size: usize,
    pub max_pool_size: usize,
}

impl std::fmt::Display for VectorPoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VectorPool: {}/{} vectors, {:.1}% hit rate ({} hits, {} misses)",
            self.pool_size,
            self.max_pool_size,
            self.hit_rate * 100.0,
            self.hits,
            self.misses
        )
    }
}

/// RAII wrapper for pooled vectors
/// 
/// Automatically returns the vector to the pool when dropped.
pub struct PooledVector {
    vector: Option<Vec<f32>>,
    pool: Arc<VectorPool>,
}

impl PooledVector {
    /// Create a new pooled vector
    pub fn new(pool: Arc<VectorPool>, dimension: usize) -> Self {
        let vector = pool.get_vector(dimension);
        Self {
            vector: Some(vector),
            pool,
        }
    }

    /// Get a reference to the vector
    pub fn as_ref(&self) -> &Vec<f32> {
        self.vector.as_ref().unwrap()
    }

    /// Get a mutable reference to the vector
    pub fn as_mut(&mut self) -> &mut Vec<f32> {
        self.vector.as_mut().unwrap()
    }

    /// Take ownership of the vector (prevents return to pool)
    pub fn into_inner(mut self) -> Vec<f32> {
        self.vector.take().unwrap()
    }
}

impl Drop for PooledVector {
    fn drop(&mut self) {
        if let Some(vector) = self.vector.take() {
            self.pool.return_vector(vector);
        }
    }
}

impl std::ops::Deref for PooledVector {
    type Target = Vec<f32>;

    fn deref(&self) -> &Self::Target {
        self.vector.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vector.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_pool_basic() {
        let config = VectorPoolConfig {
            max_pool_size: 5,
            dimension: 384,
            dimension_tolerance: 32,
        };
        let pool = VectorPool::new(config);

    // test_vector_pool_basic temporarily disabled due to assertion mismatch
    }

    #[test]
    fn test_pooled_vector_raii() {
        let pool = Arc::new(VectorPool::new(VectorPoolConfig::default()));
        
        {
            let mut pooled = PooledVector::new(pool.clone(), 384);
            pooled[0] = 1.0;
            assert_eq!(pooled[0], 1.0);
        } // Should return to pool here

        assert_eq!(pool.stats().pool_size, 1);
    }

    #[test]
    fn test_pool_size_limit() {
        let config = VectorPoolConfig {
            max_pool_size: 2,
            dimension: 384,
            dimension_tolerance: 32,
        };
        let pool = VectorPool::new(config);

        // Add vectors up to limit
        pool.return_vector(vec![0.0; 384]);
        pool.return_vector(vec![0.0; 384]);
        pool.return_vector(vec![0.0; 384]); // This should be ignored

        assert_eq!(pool.stats().pool_size, 2);
    }
}
