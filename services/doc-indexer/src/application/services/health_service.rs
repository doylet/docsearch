use std::time::Instant;
use zero_latency_core::{
    Result,
    models::{ComponentHealth, HealthStatus}
};
use crate::infrastructure::jsonrpc::types::{
    HealthCheckResult, ReadinessResult, LivenessResult, HealthCheckItem
};
use std::collections::HashMap;

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
        let mut checks = HashMap::new();
        
        // Perform individual component checks
        let vector_check = self.check_vector_store().await;
        let embedding_check = self.check_embedding_generator().await;
        let memory_check = self.check_memory_usage().await;
        
        // Convert to JSON-RPC format
        checks.insert("vector_store".to_string(), self.convert_to_check_item(&vector_check));
        checks.insert("embedding_generator".to_string(), self.convert_to_check_item(&embedding_check));
        checks.insert("memory".to_string(), self.convert_to_check_item(&memory_check));
        
        let all_checks = [&vector_check, &embedding_check, &memory_check];
        let overall_status = if all_checks.iter().all(|c| c.status.is_healthy()) {
            "healthy".to_string()
        } else if all_checks.iter().any(|c| c.status.is_unhealthy()) {
            "unhealthy".to_string()
        } else {
            "degraded".to_string()
        };
        
        Ok(HealthCheckResult {
            status: overall_status,
            timestamp: chrono::Utc::now().to_rfc3339(),
            checks,
        })
    }

    pub async fn readiness_check(&self) -> Result<ReadinessResult> {
        let health = self.health_check().await?;
        let is_ready = health.status == "healthy" || health.status == "degraded";
        
        Ok(ReadinessResult {
            ready: is_ready,
            checks: health.checks,
        })
    }

    pub async fn liveness_check(&self) -> Result<LivenessResult> {
        Ok(LivenessResult {
            alive: true,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        })
    }

    fn convert_to_check_item(&self, component: &ComponentHealth) -> HealthCheckItem {
        let status = match &component.status {
            HealthStatus::Healthy => "healthy".to_string(),
            HealthStatus::Degraded { message: _ } => "degraded".to_string(),
            HealthStatus::Unhealthy { message: _ } => "unhealthy".to_string(),
        };
        
        let message = match &component.status {
            HealthStatus::Healthy => None,
            HealthStatus::Degraded { message } => Some(message.clone()),
            HealthStatus::Unhealthy { message } => Some(message.clone()),
        };
        
        HealthCheckItem { status, message }
    }

    async fn check_vector_store(&self) -> ComponentHealth {
        ComponentHealth {
            component: "vector_store".to_string(),
            status: HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            details: None,
        }
    }

    async fn check_embedding_generator(&self) -> ComponentHealth {
        ComponentHealth {
            component: "embedding_generator".to_string(),
            status: HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            details: None,
        }
    }

    async fn check_memory_usage(&self) -> ComponentHealth {
        ComponentHealth {
            component: "memory".to_string(),
            status: HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            details: None,
        }
    }
}

impl Default for HealthService {
    fn default() -> Self {
        Self::new()
    }
}
