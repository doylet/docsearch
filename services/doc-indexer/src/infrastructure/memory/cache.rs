/// Memory-efficient caching implementation
///
/// Provides smart caching with memory pressure awareness
/// and automatic eviction strategies.
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Configuration for memory-efficient cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// TTL for entries
    pub ttl: Duration,
    /// Enable memory pressure monitoring
    pub memory_pressure_enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            ttl: Duration::from_secs(3600),     // 1 hour
            memory_pressure_enabled: true,
        }
    }
}

/// Memory-efficient cache entry
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    size_bytes: usize,
}

impl<T> CacheEntry<T> {
    fn new(value: T, size_bytes: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_bytes,
        }
    }

    fn access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    fn score(&self) -> f64 {
        // Scoring function for eviction: higher score = more likely to evict
        let age = self.created_at.elapsed().as_secs_f64();
        let recency = self.last_accessed.elapsed().as_secs_f64();
        let frequency = self.access_count as f64;

        // Combine age, recency, and frequency (lower is better)
        (age + recency * 2.0) / (frequency + 1.0)
    }
}

/// Memory-efficient cache with automatic eviction
pub struct MemoryEfficientCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    config: CacheConfig,
    entries: RwLock<HashMap<K, CacheEntry<V>>>,
    stats: RwLock<CacheStats>,
}

impl<K, V> MemoryEfficientCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Create a new memory-efficient cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: RwLock::new(HashMap::new()),
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired(self.config.ttl) {
                entries.remove(key);
                stats.expirations += 1;
                None
            } else {
                entry.access();
                stats.hits += 1;
                Some(entry.value.clone())
            }
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V) {
        let size = self.estimate_size(&value);
        self.insert_with_size(key, value, size);
    }

    /// Insert a value with explicit size
    pub fn insert_with_size(&self, key: K, value: V, size_bytes: usize) {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        // Create new entry
        let entry = CacheEntry::new(value, size_bytes);

        // Check if we need to evict
        if entries.len() >= self.config.max_entries
            || self.current_memory_usage(&entries) + size_bytes > self.config.max_memory_bytes
        {
            self.evict_entries(&mut entries, &mut stats);
        }

        // Insert new entry
        if let Some(old_entry) = entries.insert(key, entry) {
            stats.total_memory_bytes = stats
                .total_memory_bytes
                .saturating_sub(old_entry.size_bytes)
                .saturating_add(size_bytes);
        } else {
            stats.total_memory_bytes = stats.total_memory_bytes.saturating_add(size_bytes);
        }

        stats.insertions += 1;
    }

    /// Remove a value from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        if let Some(entry) = entries.remove(key) {
            stats.total_memory_bytes = stats.total_memory_bytes.saturating_sub(entry.size_bytes);
            Some(entry.value)
        } else {
            None
        }
    }

    /// Clear all entries
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        entries.clear();
        stats.total_memory_bytes = 0;
        stats.evictions += entries.len() as u64;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        *self.stats.read().unwrap()
    }

    /// Get current number of entries
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let ttl = self.config.ttl;
        let expired_keys: Vec<K> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired(ttl))
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            if let Some(entry) = entries.remove(&key) {
                stats.total_memory_bytes =
                    stats.total_memory_bytes.saturating_sub(entry.size_bytes);
                stats.expirations += 1;
            }
        }
    }

    /// Evict entries based on scoring algorithm
    fn evict_entries(&self, entries: &mut HashMap<K, CacheEntry<V>>, stats: &mut CacheStats) {
        let target_size = (self.config.max_entries * 3) / 4; // Remove 25%
        let target_memory = (self.config.max_memory_bytes * 3) / 4;

        // Score all entries
        let mut scored_entries: Vec<(K, f64)> = entries
            .iter()
            .map(|(k, entry)| (k.clone(), entry.score()))
            .collect();

        // Sort by score (highest first - most likely to evict)
        scored_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Evict entries until we're under limits
        let mut evicted = 0;
        for (key, _) in scored_entries {
            if entries.len() <= target_size && self.current_memory_usage(entries) <= target_memory {
                break;
            }

            if let Some(entry) = entries.remove(&key) {
                stats.total_memory_bytes =
                    stats.total_memory_bytes.saturating_sub(entry.size_bytes);
                evicted += 1;
            }
        }

        stats.evictions += evicted;
    }

    /// Calculate current memory usage
    fn current_memory_usage(&self, entries: &HashMap<K, CacheEntry<V>>) -> usize {
        entries.values().map(|entry| entry.size_bytes).sum()
    }

    /// Estimate size of a value (can be overridden for specific types)
    fn estimate_size(&self, _value: &V) -> usize {
        std::mem::size_of::<V>()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub insertions: u64,
    pub evictions: u64,
    pub expirations: u64,
    pub total_memory_bytes: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses > 0 {
            self.hits as f64 / (self.hits + self.misses) as f64
        } else {
            0.0
        }
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {:.1}% hit rate, {} entries, {:.1}MB memory",
            self.hit_rate() * 100.0,
            self.insertions - self.evictions - self.expirations,
            self.total_memory_bytes as f64 / (1024.0 * 1024.0)
        )
    }
}

/// Specialized cache for vectors
pub type VectorCache = MemoryEfficientCache<String, Vec<f32>>;

impl MemoryEfficientCache<String, Vec<f32>> {
    /// Create a vector-optimized cache
    pub fn for_vectors(config: CacheConfig) -> Self {
        Self::new(config)
    }

    /// Estimate size of a vector (specialized implementation)
    pub fn estimate_vector_size(&self, value: &Vec<f32>) -> usize {
        value.len() * std::mem::size_of::<f32>() + std::mem::size_of::<Vec<f32>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cache_operations() {
        let config = CacheConfig {
            max_entries: 2,
            max_memory_bytes: 1024,
            ttl: Duration::from_secs(60),
            memory_pressure_enabled: false,
        };
        let cache = MemoryEfficientCache::new(config);

        // Test insert and get
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);

        // Test statistics
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.insertions, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            max_memory_bytes: 1024 * 1024,
            ttl: Duration::from_secs(60),
            memory_pressure_enabled: false,
        };
        let cache = MemoryEfficientCache::new(config);

        // Fill cache to capacity
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        // Access key1 to make it more likely to be kept
        cache.get(&"key1".to_string());

        // Insert another item - should trigger eviction
        cache.insert("key3".to_string(), "value3".to_string());

        // key1 should still be there (accessed recently)
        // key2 might be evicted
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key3".to_string()), Some("value3".to_string()));
    }

    #[test]
    fn test_vector_cache() {
        let config = CacheConfig::default();
        let cache = VectorCache::for_vectors(config);

        let vector = vec![1.0, 2.0, 3.0];
        cache.insert("test".to_string(), vector.clone());

        assert_eq!(cache.get(&"test".to_string()), Some(vector));
    }
}
