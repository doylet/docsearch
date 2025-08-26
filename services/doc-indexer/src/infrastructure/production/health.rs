/// Health Monitoring System
/// 
/// Comprehensive health checking for production deployment including
/// dependency validation, resource monitoring, and service availability checks.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use sysinfo::System;
use crate::application::interfaces::VectorStorage;
use crate::application::interfaces::EmbeddingService;
use zero_latency_vector::VectorRepository;
use crate::infrastructure::load_testing::scenario::EmbeddingInput;

/// Overall health status of the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub duration: Duration,
    pub timestamp: u64,
    pub details: HashMap<String, String>,
}

/// Individual health check trait
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthCheckResult;
    fn timeout(&self) -> Duration;
}

/// Memory usage health check
pub struct MemoryHealthCheck {
    name: String,
    max_memory_mb: f64,
    warning_threshold: f64,
}

impl MemoryHealthCheck {
    pub fn new(max_memory_mb: f64) -> Self {
        Self {
            name: "memory".to_string(),
            max_memory_mb,
            warning_threshold: 0.8, // 80% threshold for degraded status
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str { &self.name }
    
    fn timeout(&self) -> Duration { Duration::from_secs(1) }
    
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
    // Get current memory usage using sysinfo
    let mut system = System::new_all();
    system.refresh_memory();
    let used_memory_bytes = system.used_memory();
    let total_memory_bytes = system.total_memory();
    let used_memory_mb = used_memory_bytes as f64 / 1024.0 / 1024.0;
    let memory_usage_ratio = used_memory_bytes as f64 / total_memory_bytes as f64;
        
        let duration = start.elapsed();
        let mut details = HashMap::new();
        details.insert("used_memory_mb".to_string(), format!("{:.2}", used_memory_mb));
        details.insert("total_memory_mb".to_string(), format!("{:.2}", total_memory_bytes as f64 / 1024.0 / 1024.0));
        details.insert("usage_ratio".to_string(), format!("{:.2}", memory_usage_ratio));
        details.insert("max_allowed_mb".to_string(), format!("{:.2}", self.max_memory_mb));
        
        let (status, message) = if used_memory_mb > self.max_memory_mb {
            (HealthStatus::Unhealthy, format!("Memory usage {:.2}MB exceeds limit {:.2}MB", used_memory_mb, self.max_memory_mb))
        } else if memory_usage_ratio > self.warning_threshold {
            (HealthStatus::Degraded, format!("Memory usage {:.1}% above warning threshold", memory_usage_ratio * 100.0))
        } else {
            (HealthStatus::Healthy, format!("Memory usage {:.2}MB ({:.1}%) is healthy", used_memory_mb, memory_usage_ratio * 100.0))
        };
        
        HealthCheckResult {
            name: self.name.clone(),
            status,
            message,
            duration,
            timestamp,
            details,
        }
    }
}

/// Embedding service health check
pub struct EmbeddingServiceHealthCheck {
    name: String,
    embedding_service: Arc<dyn EmbeddingService>,
}

impl EmbeddingServiceHealthCheck {
    pub fn new(embedding_service: Arc<dyn EmbeddingService>) -> Self {
        Self {
            name: "embedding_service".to_string(),
            embedding_service,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for EmbeddingServiceHealthCheck {
    fn name(&self) -> &str { &self.name }
    
    fn timeout(&self) -> Duration { Duration::from_secs(5) }
    
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        let timestamp = chrono::Utc::now().timestamp() as u64;
        
    let test_text = "health check test".to_string();
    let mut details = HashMap::new();
    match tokio::time::timeout(self.timeout(), self.embedding_service.generate_embeddings(&test_text)).await {
            Ok(Ok(embedding)) => {
                let duration = start.elapsed();
                details.insert("embedding_dimension".to_string(), embedding.len().to_string());
                details.insert("response_time_ms".to_string(), duration.as_millis().to_string());
                
                let (status, message) = if duration > Duration::from_millis(2000) {
                    (HealthStatus::Degraded, format!("Embedding service slow: {}ms", duration.as_millis()))
                } else {
                    (HealthStatus::Healthy, format!("Embedding service healthy: {}ms", duration.as_millis()))
                };
                
                HealthCheckResult {
                    name: self.name.clone(),
                    status,
                    message,
                    duration,
                    timestamp,
                    details,
                }
            },
            Ok(Err(error)) => {
                let duration = start.elapsed();
                details.insert("error".to_string(), error.to_string());
                
                HealthCheckResult {
                    name: self.name.clone(),
                    status: HealthStatus::Unhealthy,
                    message: format!("Embedding service error: {}", error),
                    duration,
                    timestamp,
                    details,
                }
            },
            Err(_timeout) => {
                let duration = start.elapsed();
                details.insert("timeout_ms".to_string(), self.timeout().as_millis().to_string());
                
                HealthCheckResult {
                    name: self.name.clone(),
                    status: HealthStatus::Unhealthy,
                    message: "Embedding service timeout".to_string(),
                    duration,
                    timestamp,
                    details,
                }
            }
        }
    }
}

/// Vector store health check
pub struct VectorStoreHealthCheck {
    name: String,
    vector_repo: Arc<dyn VectorRepository>,
}

impl VectorStoreHealthCheck {
    pub fn new(vector_repo: Arc<dyn VectorRepository>) -> Self {
        Self {
            name: "vector_store".to_string(),
            vector_repo,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for VectorStoreHealthCheck {
    fn name(&self) -> &str { &self.name }
    
    fn timeout(&self) -> Duration { Duration::from_secs(3) }
    
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let mut details = HashMap::new();
        
    // Test vector store with a simple query
    let test_vector = vec![0.1; 384]; // Standard embedding dimension
    let k = 3;
    match tokio::time::timeout(self.timeout(), self.vector_repo.search(test_vector, k)).await {
            Ok(Ok(results)) => {
                let duration = start.elapsed();
                details.insert("results_count".to_string(), results.len().to_string());
                details.insert("response_time_ms".to_string(), duration.as_millis().to_string());
                
                let (status, message) = if duration > Duration::from_millis(1000) {
                    (HealthStatus::Degraded, format!("Vector store slow: {}ms", duration.as_millis()))
                } else {
                    (HealthStatus::Healthy, format!("Vector store healthy: {}ms", duration.as_millis()))
                };
                
                HealthCheckResult {
                    name: self.name.clone(),
                    status,
                    message,
                    duration,
                    timestamp,
                    details,
                }
            },
            Ok(Err(error)) => {
                let duration = start.elapsed();
                details.insert("error".to_string(), error.to_string());
                
                HealthCheckResult {
                    name: self.name.clone(),
                    status: HealthStatus::Unhealthy,
                    message: format!("Vector store error: {}", error),
                    duration,
                    timestamp,
                    details,
                }
            },
            Err(_timeout) => {
                let duration = start.elapsed();
                details.insert("timeout_ms".to_string(), self.timeout().as_millis().to_string());
                
                HealthCheckResult {
                    name: self.name.clone(),
                    status: HealthStatus::Unhealthy,
                    message: "Vector store timeout".to_string(),
                    duration,
                    timestamp,
                    details,
                }
            }
        }
    }
}

/// Aggregated health status for all checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedHealthStatus {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub summary: String,
    pub timestamp: u64,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
}

/// Main health checker orchestrator
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
    check_interval: Duration,
    check_timeout: Duration,
    latest_status: Arc<RwLock<Option<AggregatedHealthStatus>>>,
    running: Arc<RwLock<bool>>,
}

impl HealthChecker {
    pub fn new(check_interval: Duration, check_timeout: Duration) -> Self {
        Self {
            checks: Vec::new(),
            check_interval,
            check_timeout,
            latest_status: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Add a health check to the checker
    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }
    
    /// Add standard health checks for the service
    pub fn add_standard_checks(
        &mut self,
        embedding_service: Arc<dyn EmbeddingService>,
        vector_repo: Arc<dyn VectorRepository>,
    ) {
        self.add_check(Box::new(MemoryHealthCheck::new(1024.0))); // 1GB limit
        self.add_check(Box::new(EmbeddingServiceHealthCheck::new(embedding_service)));
        self.add_check(Box::new(VectorStoreHealthCheck::new(vector_repo)));
    }
    
    /// Start periodic health checking
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.running.write().await = true;
        
        let checks = std::mem::take(&mut self.checks);
        let check_interval = self.check_interval;
        let check_timeout = self.check_timeout;
        let latest_status = self.latest_status.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            while *running.read().await {
                let health_status = Self::run_all_checks(&checks, check_timeout).await;
                *latest_status.write().await = Some(health_status);
                
                tokio::time::sleep(check_interval).await;
            }
        });
        
        Ok(())
    }
    
    /// Stop health checking
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.running.write().await = false;
        Ok(())
    }
    
    /// Get the latest health status
    pub async fn get_latest_status(&self) -> HealthStatus {
        if let Some(status) = self.latest_status.read().await.as_ref() {
            status.overall_status.clone()
        } else {
            HealthStatus::Unhealthy // No status available yet
        }
    }
    
    /// Get detailed health report
    pub async fn get_detailed_status(&self) -> Option<AggregatedHealthStatus> {
        self.latest_status.read().await.clone()
    }
    
    /// Run a single health check cycle
    pub async fn run_check_cycle(&self) -> AggregatedHealthStatus {
        Self::run_all_checks(&self.checks, self.check_timeout).await
    }
    
    async fn run_all_checks(
        checks: &[Box<dyn HealthCheck>],
        timeout: Duration,
    ) -> AggregatedHealthStatus {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let mut check_results = Vec::new();
        
        for check in checks {
            let result = tokio::time::timeout(timeout, check.check())
                .await
                .unwrap_or_else(|_| HealthCheckResult {
                    name: check.name().to_string(),
                    status: HealthStatus::Unhealthy,
                    message: "Health check timeout".to_string(),
                    duration: timeout,
                    timestamp,
                    details: HashMap::new(),
                });
            
            check_results.push(result);
        }
        
        // Aggregate results
        let healthy_count = check_results.iter().filter(|r| r.status == HealthStatus::Healthy).count();
        let degraded_count = check_results.iter().filter(|r| r.status == HealthStatus::Degraded).count();
        let unhealthy_count = check_results.iter().filter(|r| r.status == HealthStatus::Unhealthy).count();
        
        let overall_status = if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        let summary = format!(
            "Health: {} checks ({}✅ {}⚠️ {}❌)",
            check_results.len(),
            healthy_count,
            degraded_count,
            unhealthy_count
        );
        
        AggregatedHealthStatus {
            overall_status,
            checks: check_results,
            summary,
            timestamp,
            healthy_count,
            degraded_count,
            unhealthy_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::test_utils::{MockEmbeddingService, MockVectorStore};
    

    

    

}
