use serde::{Deserialize, Serialize};

/// Engine that provided this result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchEngine {
    Vector,
    BM25,
    Hybrid,
}

/// Tracking which search engines and query variants contributed to a result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromSignals {
    /// Result found by BM25 search
    pub bm25: bool,
    /// Result found by vector search
    pub vector: bool,
    /// Query variant indices that found this result (for multi-query expansion)
    pub variants: Vec<usize>,
    /// Result enhanced by query expansion
    pub query_expansion: bool,
}

impl FromSignals {
    /// Create new signals for vector-only result
    pub fn vector_only() -> Self {
        Self {
            bm25: false,
            vector: true,
            variants: vec![0], // Original query
            query_expansion: false,
        }
    }

    /// Create new signals for BM25-only result
    pub fn bm25_only() -> Self {
        Self {
            bm25: true,
            vector: false,
            variants: vec![0], // Original query
            query_expansion: false,
        }
    }

    /// Create new signals for hybrid result (both engines)
    pub fn hybrid() -> Self {
        Self {
            bm25: true,
            vector: true,
            variants: vec![0], // Original query
            query_expansion: false,
        }
    }    /// Create signals for specific query variant
    pub fn from_variant(variant_index: usize, engine: SearchEngine) -> Self {
        match engine {
            SearchEngine::Vector => Self {
                bm25: false,
                vector: true,
                variants: vec![variant_index],
                query_expansion: false,
            },
            SearchEngine::BM25 => Self {
                bm25: true,
                vector: false,
                variants: vec![variant_index],
                query_expansion: false,
            },
            SearchEngine::Hybrid => Self {
                bm25: true,
                vector: true,
                variants: vec![variant_index],
                query_expansion: false,
            },
        }
    }
    
    /// Merge signals from multiple sources
    pub fn merge(&mut self, other: &FromSignals) {
        self.bm25 |= other.bm25;
        self.vector |= other.vector;
        self.query_expansion |= other.query_expansion;
        
        // Merge variant indices, keeping unique values
        for &variant in &other.variants {
            if !self.variants.contains(&variant) {
                self.variants.push(variant);
            }
        }
        self.variants.sort_unstable();
    }
    
    /// Get primary search engine that contributed this result
    pub fn primary_engine(&self) -> SearchEngine {
        match (self.bm25, self.vector) {
            (true, true) => SearchEngine::Hybrid,
            (true, false) => SearchEngine::BM25,
            (false, true) => SearchEngine::Vector,
            (false, false) => SearchEngine::Vector, // Default fallback
        }
    }
}

impl Default for FromSignals {
    fn default() -> Self {
        Self::vector_only()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let vector_signals = FromSignals::vector_only();
        assert!(!vector_signals.bm25);
        assert!(vector_signals.vector);
        assert_eq!(vector_signals.variants, vec![0]);

        let bm25_signals = FromSignals::bm25_only();
        assert!(bm25_signals.bm25);
        assert!(!bm25_signals.vector);

        let hybrid_signals = FromSignals::hybrid();
        assert!(hybrid_signals.bm25);
        assert!(hybrid_signals.vector);
    }

    #[test]
    fn test_signal_merging() {
        let mut signals = FromSignals::vector_only();
        let bm25_signals = FromSignals::from_variant(1, SearchEngine::BM25);
        
        signals.merge(&bm25_signals);
        
        assert!(signals.bm25);
        assert!(signals.vector);
        assert_eq!(signals.variants, vec![0, 1]);
        assert!(matches!(signals.primary_engine(), SearchEngine::Hybrid));
    }

    #[test]
    fn test_variant_signals() {
        let signals = FromSignals::from_variant(2, SearchEngine::Vector);
        assert_eq!(signals.variants, vec![2]);
        assert!(signals.vector);
        assert!(!signals.bm25);
    }
}
