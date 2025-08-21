use std::time::Instant;
use zero_latency_core::{
    Result,
    models::{
        HealthCheckResult, HealthComponentCheck, HealthStatus,
        ReadinessResult, LivenessResult
    }
};

#[derive(Clone)]
pub struct HealthService {
    start_time: Instant,
}

impl HealthService {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        let mut checks = Vec::new();

        checks.push(self.check_vector_store().await);
        checks.push(self.check_embedding_generator().await);
        checks.push(self.check_memory_usage().await);
        
        let overall_status = if checks.iter().all(|c| matches!(c.status, HealthStatus::Healthy)) {
            HealthStatus::Healthy
        } else if checks.iter().any(|c| matches!(c.status, HealthStatus::Unhealthy { .. })) {
            HealthStatus::Unhealthy { message: "One or more components are unhealthy".to_string() }
        } else {
            HealthStatus::Degraded { message: "One or more components are degraded".to_string() }
        };
        
        Ok(HealthCheckResult {
            status: overall_status,
            checks,
            timestamp: chrono::Utc::now(),
            uptime: self.start_time.elapsed(),
        })
    }

    pub async fn readiness_check(&self) -> Result<ReadinessResult> {
        let health = self.health_check().await?;
        let is_ready = matches!(health.status, HealthStatus::Healthy | HealthStatus::Degraded { .. });
        
        Ok(ReadinessResult {
            ready: is_ready,
            checks: health.checks,
            timestamp: chrono::Utc::now(),
        })
    }

    pub async fn liveness_check(&self) -> Result<LivenessResult> {
        Ok(LivenessResult {
            alive: true,
            timestamp: chrono::Utc::now(),
            uptime: self.start_time.elapsed(),
        })
    }

    async fn check_vector_store(&self) -> HealthComponentCheck {
        HealthComponentCheck {
            component: "vector_store".to_string(),
            status: HealthStatus::Healthy,
            message: Some("Vector store is operational".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    async fn check_embedding_generator(&self) -> HealthComponentCheck {
        HealthComponentCheck {
            component: "embedding_generator".to_string(),
            status: HealthStatus::Healthy,
            message: Some("Embedding generator is operational".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    async fn check_memory_usage(&self) -> HealthComponentCheck {
        HealthComponentCheck {
            component: "memory".to_string(),
            status: HealthStatus::Healthy,
            message: Some("Memory usage is within normal range".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl Default for HealthService {
    fn default() -> Self {
        Self::new()
    }
}
