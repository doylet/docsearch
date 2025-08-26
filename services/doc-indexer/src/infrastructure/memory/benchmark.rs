/// Performance benchmarking utilities
///
/// This module provides simple benchmarking tools to measure
/// the impact of memory optimizations.
use std::time::{Duration, Instant};

/// Simple benchmark result
#[derive(Debug)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub duration: Duration,
    pub memory_usage: Option<usize>,
    pub operations: usize,
    pub ops_per_sec: f64,
}

impl BenchmarkResult {
    pub fn new(test_name: String, duration: Duration, operations: usize) -> Self {
        let ops_per_sec = operations as f64 / duration.as_secs_f64();
        Self {
            test_name,
            duration,
            memory_usage: None,
            operations,
            ops_per_sec,
        }
    }

    pub fn with_memory(mut self, memory_usage: usize) -> Self {
        self.memory_usage = Some(memory_usage);
        self
    }
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:.2}ms ({} ops, {:.0} ops/sec)",
            self.test_name,
            self.duration.as_millis(),
            self.operations,
            self.ops_per_sec
        )?;

        if let Some(memory) = self.memory_usage {
            write!(f, ", {}KB memory", memory / 1024)?;
        }

        Ok(())
    }
}

/// Simple benchmark runner
pub struct Benchmark {
    results: Vec<BenchmarkResult>,
}

impl Benchmark {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run a benchmark test
    pub fn run<F>(&mut self, name: &str, operations: usize, test_fn: F)
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        test_fn();
        let duration = start.elapsed();

        let result = BenchmarkResult::new(name.to_string(), duration, operations);
        println!("{}", result);
        self.results.push(result);
    }

    /// Run a benchmark test with memory measurement
    pub fn run_with_memory<F>(&mut self, name: &str, operations: usize, test_fn: F)
    where
        F: FnOnce() -> usize,
    {
        let start = Instant::now();
        let memory_usage = test_fn();
        let duration = start.elapsed();

        let result =
            BenchmarkResult::new(name.to_string(), duration, operations).with_memory(memory_usage);
        println!("{}", result);
        self.results.push(result);
    }

    /// Get benchmark results
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Print summary comparison
    pub fn print_summary(&self) {
        if self.results.len() < 2 {
            return;
        }

        println!("\n=== Benchmark Summary ===");
        for result in &self.results {
            println!("{}", result);
        }

        // Find fastest and slowest
        let fastest = self
            .results
            .iter()
            .max_by(|a, b| a.ops_per_sec.partial_cmp(&b.ops_per_sec).unwrap())
            .unwrap();
        let slowest = self
            .results
            .iter()
            .min_by(|a, b| a.ops_per_sec.partial_cmp(&b.ops_per_sec).unwrap())
            .unwrap();

        if fastest.test_name != slowest.test_name {
            let speedup = fastest.ops_per_sec / slowest.ops_per_sec;
            println!(
                "\nSpeedup: {:.2}x ({} vs {})",
                speedup, fastest.test_name, slowest.test_name
            );
        }
    }
}

/// Memory usage estimation utilities
pub mod memory {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    /// Simple allocator wrapper for memory tracking
    pub struct TrackingAllocator;

    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
            }
            ptr
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout);
            ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
        }
    }

    /// Get current allocated memory (approximate)
    pub fn current_usage() -> usize {
        ALLOCATED.load(Ordering::Relaxed)
    }

    /// Reset memory tracking
    pub fn reset() {
        ALLOCATED.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::memory::{VectorPool, VectorPoolConfig};
    use std::sync::Arc;

    #[test]
    fn test_vector_allocation_benchmark() {
        let mut bench = Benchmark::new();

        // Test traditional allocation
        bench.run("Traditional Vector Allocation", 10000, || {
            for _ in 0..10000 {
                let _vec: Vec<f32> = vec![0.0; 384];
            }
        });

        // Test pooled allocation
        let pool = Arc::new(VectorPool::new(VectorPoolConfig::default()));
        bench.run("Pooled Vector Allocation", 10000, || {
            for _ in 0..10000 {
                let vec = pool.get_vector(384);
                pool.return_vector(vec);
            }
        });

        bench.print_summary();
    }

    #[test]
    fn test_string_operations_benchmark() {
        let mut bench = Benchmark::new();

        // Test string creation
        bench.run("String Creation", 10000, || {
            for i in 0..10000 {
                let _s = format!("test_string_{}", i);
            }
        });

        // Test string interning
        use crate::infrastructure::memory::StringInterner;
        let interner = StringInterner::new();
        bench.run("String Interning", 10000, || {
            for i in 0..1000 {
                for _ in 0..10 {
                    let _s = interner.intern(&format!("test_string_{}", i));
                }
            }
        });

        bench.print_summary();
    }
}
