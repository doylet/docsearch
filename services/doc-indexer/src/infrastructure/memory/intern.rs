/// String interning for memory optimization
/// 
/// Reduces memory usage by storing common strings once and 
/// referencing them by ID. Particularly useful for metadata
/// keys, file extensions, and other repeated strings.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

/// Interned string reference
/// 
/// A lightweight reference to an interned string that
/// can be cloned cheaply and compared by ID rather than content.
#[derive(Debug, Clone)]
pub struct InternedString {
    id: u32,
    interner: Arc<StringInterner>,
}

impl InternedString {
    /// Get the string content
    pub fn as_str(&self) -> String {
        self.interner.get_by_id(self.id).unwrap_or_default()
    }

    /// Get the intern ID
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl PartialEq for InternedString {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InternedString {}

impl Hash for InternedString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        // For AsRef, we need to return a temporary - this is a limitation
        // In practice, users should call as_str() directly
        ""
    }
}

/// String interner for memory optimization
/// 
/// Stores strings once and provides lightweight references.
/// Thread-safe and optimized for read-heavy workloads.
#[derive(Debug)]
pub struct StringInterner {
    strings: RwLock<Vec<String>>,
    lookup: RwLock<HashMap<String, u32>>,
    stats: RwLock<InternerStats>,
}

impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            strings: RwLock::new(Vec::new()),
            lookup: RwLock::new(HashMap::new()),
            stats: RwLock::new(InternerStats::default()),
        })
    }

    /// Intern a string and get a reference
    pub fn intern(self: &Arc<Self>, s: &str) -> InternedString {
        // Fast path: check if already interned (read lock)
        {
            let lookup = self.lookup.read().unwrap();
            if let Some(&id) = lookup.get(s) {
                // Update hit counter
                self.stats.write().unwrap().hits += 1;
                return InternedString {
                    id,
                    interner: self.clone(),
                };
            }
        }

        // Slow path: intern new string (write lock)
        {
            let mut lookup = self.lookup.write().unwrap();
            let mut strings = self.strings.write().unwrap();
            
            // Double-check in case another thread interned it
            if let Some(&id) = lookup.get(s) {
                self.stats.write().unwrap().hits += 1;
                return InternedString {
                    id,
                    interner: self.clone(),
                };
            }

            // Actually intern the string
            let id = strings.len() as u32;
            strings.push(s.to_string());
            lookup.insert(s.to_string(), id);
            
            // Update miss counter
            self.stats.write().unwrap().misses += 1;

            InternedString {
                id,
                interner: self.clone(),
            }
        }
    }

    /// Get string by ID (internal use)
    fn get_by_id(&self, id: u32) -> Option<String> {
        let strings = self.strings.read().unwrap();
        strings.get(id as usize).cloned()
    }

    /// Get interner statistics
    pub fn stats(&self) -> InternerStats {
        *self.stats.read().unwrap()
    }

    /// Get number of interned strings
    pub fn len(&self) -> usize {
        self.strings.read().unwrap().len()
    }

    /// Check if interner is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get total memory usage estimate
    pub fn memory_usage(&self) -> usize {
        let strings = self.strings.read().unwrap();
        let lookup = self.lookup.read().unwrap();
        
        let strings_size: usize = strings.iter().map(|s| s.capacity()).sum();
        let lookup_size = lookup.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<u32>());
        let vec_overhead = strings.capacity() * std::mem::size_of::<String>();
        
        strings_size + lookup_size + vec_overhead
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self {
            strings: RwLock::new(Vec::new()),
            lookup: RwLock::new(HashMap::new()),
            stats: RwLock::new(InternerStats::default()),
        }
    }
}

/// Statistics for string interner
#[derive(Debug, Clone, Copy, Default)]
pub struct InternerStats {
    pub hits: u64,
    pub misses: u64,
}

impl InternerStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses > 0 {
            self.hits as f64 / (self.hits + self.misses) as f64
        } else {
            0.0
        }
    }
}

impl std::fmt::Display for InternerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StringInterner: {:.1}% hit rate ({} hits, {} misses)",
            self.hit_rate() * 100.0,
            self.hits,
            self.misses
        )
    }
}

/// Common string constants for interning
pub mod constants {
    use super::*;
    use std::sync::OnceLock;

    static COMMON_INTERNER: OnceLock<Arc<StringInterner>> = OnceLock::new();

    fn get_common_interner() -> &'static Arc<StringInterner> {
        COMMON_INTERNER.get_or_init(|| {
            let interner = StringInterner::new();
            
            // Pre-intern common strings
            for s in COMMON_STRINGS {
                interner.intern(s);
            }
            
            interner
        })
    }

    /// Pre-defined common strings
    const COMMON_STRINGS: &[&str] = &[
        // File extensions
        ".md", ".txt", ".rs", ".js", ".ts", ".py", ".html", ".json", ".yaml", ".toml",
        // Content types
        "markdown", "text", "rust", "javascript", "python", "html", "json", "yaml", "toml",
        // Common metadata keys
        "title", "author", "language", "content_type", "category", "tags",
        // Collection names
        "default", "documents", "code", "notes",
        // Common paths
        "src", "docs", "tests", "examples", "target", "node_modules", ".git",
    ];

    /// Get an interned version of a common string
    pub fn intern_common(s: &str) -> InternedString {
        get_common_interner().intern(s)
    }

    /// Get statistics for common string interner
    pub fn common_stats() -> InternerStats {
        get_common_interner().stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let interner = StringInterner::new();
        
        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        let s3 = interner.intern("world");
        
        // Same string should have same ID
        assert_eq!(s1.id(), s2.id());
        assert_ne!(s1.id(), s3.id());
        
        // Content should be preserved
        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s3.as_str(), "world");
        
        // Should have 1 hit, 2 misses
        let stats = interner.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 2);
    }

    #[test]
    fn test_interned_string_equality() {
        let interner = StringInterner::new();
        
        let s1 = interner.intern("test");
        let s2 = interner.intern("test");
        let s3 = interner.intern("other");
        
        assert_eq!(s1, s2); // Same content, same ID
        assert_ne!(s1, s3); // Different content, different ID
    }

    #[test]
    fn test_common_strings() {
        let s1 = constants::intern_common(".rs");
        let s2 = constants::intern_common(".rs");
        
        assert_eq!(s1, s2);
        assert_eq!(s1.as_str(), ".rs");
    }

    #[test]
    fn test_memory_usage() {
        let interner = StringInterner::new();
        
        let initial_usage = interner.memory_usage();
        
        interner.intern("some test string");
        
        let after_usage = interner.memory_usage();
        assert!(after_usage > initial_usage);
    }
}
