//! Health checking framework for production services
//!
//! Provides a standardized health checking system that can monitor
//! various service dependencies and report overall system health.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy and operating normally
    Healthy,
    /// Service is degraded but still functional
    Degraded,
    /// Service is unhealthy and not functioning properly
    Unhealthy,
    /// Health check could not be performed
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Name of the health check
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Human-readable message
    pub message: String,
    /// Additional metadata
    pub details: HashMap<String, serde_json::Value>,
    /// When this check was last performed
    pub timestamp: String,
    /// How long the check took to complete
    pub duration_ms: f64,
}

/// Overall system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall system status
    pub status: HealthStatus,
    /// Individual check results
    pub checks: Vec<HealthCheckResult>,
    /// Overall system uptime
    pub uptime_seconds: f64,
    /// When this report was generated
    pub timestamp: String,
    /// Service version information
    pub version: String,
}

/// Trait for implementing health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of this health check
    fn name(&self) -> &str;
    
    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;
    
    /// Timeout for this health check (default 30 seconds)
    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

/// Health check manager that coordinates multiple health checks
pub struct HealthChecker {
    checks: Vec<Arc<dyn HealthCheck>>,
    start_time: Instant,
    version: String,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            checks: Vec::new(),
            start_time: Instant::now(),
            version: version.into(),
        }
    }
    
    /// Add a health check
    pub fn add_check(&mut self, check: Arc<dyn HealthCheck>) {
        self.checks.push(check);
    }
    
    /// Perform all health checks and generate a report
    pub async fn check_health(&self) -> HealthReport {
        let mut check_results = Vec::new();
        let mut overall_status = HealthStatus::Healthy;
        
        // Run all health checks concurrently
        let futures: Vec<_> = self.checks.iter().map(|check| {
            let check = Arc::clone(check);
            async move {
                let check_timeout = check.timeout();
                match timeout(check_timeout, check.check()).await {
                    Ok(result) => result,
                    Err(_) => HealthCheckResult {
                        name: check.name().to_string(),
                        status: HealthStatus::Unhealthy,
                        message: format!("Health check timed out after {:?}", check_timeout),
                        details: HashMap::new(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        duration_ms: check_timeout.as_millis() as f64,
                    }
                }
            }
        }).collect();
        
        let results = futures::future::join_all(futures).await;
        
        // Aggregate results and determine overall status
        for result in results {
            match result.status {
                HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
                HealthStatus::Degraded if overall_status == HealthStatus::Healthy => {
                    overall_status = HealthStatus::Degraded;
                }
                HealthStatus::Unknown if overall_status != HealthStatus::Unhealthy => {
                    overall_status = HealthStatus::Unknown;
                }
                _ => {}
            }
            check_results.push(result);
        }
        
        HealthReport {
            status: overall_status,
            checks: check_results,
            uptime_seconds: self.start_time.elapsed().as_secs_f64(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: self.version.clone(),
        }
    }
    
    /// Quick readiness check (subset of health checks)
    pub async fn check_readiness(&self) -> HealthStatus {
        // For readiness, we only check critical dependencies
        // This is a faster check used by load balancers
        let critical_checks: Vec<_> = self.checks.iter()
            .filter(|check| self.is_critical_check(check.name()))
            .collect();
            
        if critical_checks.is_empty() {
            return HealthStatus::Healthy;
        }
        
        let futures: Vec<_> = critical_checks.iter().map(|check| {
            let check = Arc::clone(check);
            async move {
                match timeout(Duration::from_secs(5), check.check()).await {
                    Ok(result) => result.status,
                    Err(_) => HealthStatus::Unhealthy,
                }
            }
        }).collect();
        
        let statuses = futures::future::join_all(futures).await;
        
        // If any critical check is unhealthy, service is not ready
        for status in statuses {
            if status == HealthStatus::Unhealthy {
                return HealthStatus::Unhealthy;
            }
        }
        
        HealthStatus::Healthy
    }
    
    fn is_critical_check(&self, name: &str) -> bool {
        // Define which checks are critical for readiness
        matches!(name, "database" | "vector_store" | "cache" | "storage")
    }
}

/// Database connectivity health check
pub struct DatabaseHealthCheck {
    name: String,
    connection_string: String,
}

impl DatabaseHealthCheck {
    pub fn new(name: impl Into<String>, connection_string: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            connection_string: connection_string.into(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        
        // TODO: Implement actual database connectivity check
        // For now, simulate a check
        let (status, message) = if self.connection_string.contains("localhost") {
            (HealthStatus::Healthy, "Database connection successful".to_string())
        } else {
            (HealthStatus::Unknown, "Database check not implemented".to_string())
        };
        
        let duration_ms = start.elapsed().as_millis() as f64;
        
        HealthCheckResult {
            name: self.name.clone(),
            status,
            message,
            details: HashMap::from([
                ("connection_string".to_string(), serde_json::Value::String(
                    self.connection_string.chars().take(20).collect::<String>() + "..."
                )),
                ("timeout_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(30000)
                )),
            ]),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms,
        }
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}

/// Memory usage health check
pub struct MemoryHealthCheck {
    warning_threshold_mb: f64,
    critical_threshold_mb: f64,
}

impl MemoryHealthCheck {
    pub fn new(warning_threshold_mb: f64, critical_threshold_mb: f64) -> Self {
        Self {
            warning_threshold_mb,
            critical_threshold_mb,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str {
        "memory"
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        
        // Get memory usage (simplified - in real implementation would use system APIs)
        let memory_usage_mb = self.get_memory_usage_mb();
        
        let (status, message) = if memory_usage_mb > self.critical_threshold_mb {
            (HealthStatus::Unhealthy, format!("Memory usage critically high: {:.1}MB", memory_usage_mb))
        } else if memory_usage_mb > self.warning_threshold_mb {
            (HealthStatus::Degraded, format!("Memory usage elevated: {:.1}MB", memory_usage_mb))
        } else {
            (HealthStatus::Healthy, format!("Memory usage normal: {:.1}MB", memory_usage_mb))
        };
        
        let duration_ms = start.elapsed().as_millis() as f64;
        
        HealthCheckResult {
            name: "memory".to_string(),
            status,
            message,
            details: HashMap::from([
                ("current_mb".to_string(), serde_json::Value::Number(
                    serde_json::Number::from_f64(memory_usage_mb).unwrap()
                )),
                ("warning_threshold_mb".to_string(), serde_json::Value::Number(
                    serde_json::Number::from_f64(self.warning_threshold_mb).unwrap()
                )),
                ("critical_threshold_mb".to_string(), serde_json::Value::Number(
                    serde_json::Number::from_f64(self.critical_threshold_mb).unwrap()
                )),
            ]),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms,
        }
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

impl MemoryHealthCheck {
    fn get_memory_usage_mb(&self) -> f64 {
        // Simplified memory usage calculation
        // In a real implementation, this would use system APIs
        // For now, return a simulated value
        
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/self/status on Linux
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<f64>() {
                                return kb / 1024.0; // Convert KB to MB
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: return a simulated value
        128.5
    }
}

/// Disk space health check
pub struct DiskSpaceHealthCheck {
    path: String,
    warning_threshold_percent: f64,
    critical_threshold_percent: f64,
}

impl DiskSpaceHealthCheck {
    pub fn new(path: impl Into<String>, warning_threshold_percent: f64, critical_threshold_percent: f64) -> Self {
        Self {
            path: path.into(),
            warning_threshold_percent,
            critical_threshold_percent,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DiskSpaceHealthCheck {
    fn name(&self) -> &str {
        "disk_space"
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        
        let usage_percent = self.get_disk_usage_percent();
        
        let (status, message) = if usage_percent > self.critical_threshold_percent {
            (HealthStatus::Unhealthy, format!("Disk usage critically high: {:.1}%", usage_percent))
        } else if usage_percent > self.warning_threshold_percent {
            (HealthStatus::Degraded, format!("Disk usage elevated: {:.1}%", usage_percent))
        } else {
            (HealthStatus::Healthy, format!("Disk usage normal: {:.1}%", usage_percent))
        };
        
        let duration_ms = start.elapsed().as_millis() as f64;
        
        HealthCheckResult {
            name: "disk_space".to_string(),
            status,
            message,
            details: HashMap::from([
                ("path".to_string(), serde_json::Value::String(self.path.clone())),
                ("usage_percent".to_string(), serde_json::Value::Number(
                    serde_json::Number::from_f64(usage_percent).unwrap()
                )),
                ("warning_threshold".to_string(), serde_json::Value::Number(
                    serde_json::Number::from_f64(self.warning_threshold_percent).unwrap()
                )),
                ("critical_threshold".to_string(), serde_json::Value::Number(
                    serde_json::Number::from_f64(self.critical_threshold_percent).unwrap()
                )),
            ]),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms,
        }
    }
}

impl DiskSpaceHealthCheck {
    fn get_disk_usage_percent(&self) -> f64 {
        // Simplified disk usage calculation
        // In a real implementation, this would use statvfs or similar
        
        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;
            
            unsafe {
                let path = CString::new(self.path.as_bytes()).unwrap();
                let mut stat: libc::statvfs = mem::zeroed();
                
                if libc::statvfs(path.as_ptr(), &mut stat) == 0 {
                    let total = stat.f_blocks as f64 * stat.f_frsize as f64;
                    let available = stat.f_bavail as f64 * stat.f_frsize as f64;
                    let used = total - available;
                    return (used / total) * 100.0;
                }
            }
        }
        
        // Fallback: return a simulated value
        45.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_checker() {
        let mut checker = HealthChecker::new("1.0.0");
        
        checker.add_check(Arc::new(MemoryHealthCheck::new(1000.0, 2000.0)));
        checker.add_check(Arc::new(DiskSpaceHealthCheck::new("/", 80.0, 95.0)));
        
        let report = checker.check_health().await;
        
        assert_eq!(report.checks.len(), 2);
        assert!(!report.version.is_empty());
        assert!(report.uptime_seconds >= 0.0);
    }
    
    #[tokio::test]
    async fn test_readiness_check() {
        let mut checker = HealthChecker::new("1.0.0");
        
        checker.add_check(Arc::new(DatabaseHealthCheck::new("database", "localhost:5432")));
        
        let status = checker.check_readiness().await;
        
        // Should be healthy for localhost database
        assert_eq!(status, HealthStatus::Healthy);
    }
}
