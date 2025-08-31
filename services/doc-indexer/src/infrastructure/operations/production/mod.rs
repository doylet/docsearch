/// Production Deployment Infrastructure
///
/// Comprehensive production deployment support including health monitoring,
/// graceful shutdown, startup validation, and deployment readiness checks.
pub mod health;
pub mod monitoring;
pub mod shutdown;
pub mod startup;

pub use health::{HealthChecker, HealthStatus};
pub use monitoring::{ProductionMonitor, SystemMetrics};
pub use shutdown::{GracefulShutdown, ShutdownSignal};
pub use startup::StartupValidator;

use std::time::Duration;

/// Configuration for production deployment features
#[derive(Debug, Clone)]
pub struct ProductionConfig {
    /// Health check configuration
    pub health_check_interval: Duration,
    pub health_check_timeout: Duration,
    pub health_check_enabled: bool,

    /// Monitoring configuration
    pub monitoring_enabled: bool,
    pub metrics_collection_interval: Duration,
    pub performance_alerts_enabled: bool,

    /// Shutdown configuration
    pub graceful_shutdown_timeout: Duration,
    pub shutdown_signal_handlers: bool,

    /// Startup validation configuration
    pub startup_validation_enabled: bool,
    pub startup_timeout: Duration,
    pub dependency_check_enabled: bool,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(
                std::env::var("HEALTH_CHECK_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
            ),
            health_check_timeout: Duration::from_secs(
                std::env::var("HEALTH_CHECK_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
            ),
            health_check_enabled: std::env::var("HEALTH_CHECK_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            monitoring_enabled: std::env::var("MONITORING_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            metrics_collection_interval: Duration::from_secs(
                std::env::var("METRICS_COLLECTION_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
            ),
            performance_alerts_enabled: std::env::var("PERFORMANCE_ALERTS_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            graceful_shutdown_timeout: Duration::from_secs(
                std::env::var("GRACEFUL_SHUTDOWN_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
            ),
            shutdown_signal_handlers: std::env::var("SHUTDOWN_SIGNAL_HANDLERS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            startup_validation_enabled: std::env::var("STARTUP_VALIDATION_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            startup_timeout: Duration::from_secs(
                std::env::var("STARTUP_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
            ),
            dependency_check_enabled: std::env::var("DEPENDENCY_CHECK_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        }
    }
}

/// Overall production deployment status
#[derive(Debug, Clone, PartialEq)]
pub enum DeploymentStatus {
    Starting,
    Healthy,
    Degraded,
    Unhealthy,
    ShuttingDown,
    Stopped,
}

/// Production deployment orchestrator
pub struct ProductionDeployment {
    config: ProductionConfig,
    health_checker: Option<HealthChecker>,
    monitor: Option<ProductionMonitor>,
    shutdown_handler: Option<GracefulShutdown>,
    startup_validator: Option<StartupValidator>,
    status: DeploymentStatus,
}

impl ProductionDeployment {
    pub fn new(config: ProductionConfig) -> Self {
        Self {
            config,
            health_checker: None,
            monitor: None,
            shutdown_handler: None,
            startup_validator: None,
            status: DeploymentStatus::Starting,
        }
    }

    /// Initialize all production deployment components
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Initializing production deployment...");

        // Initialize startup validator
        if self.config.startup_validation_enabled {
            let startup_config = crate::infrastructure::operations::production::startup::StartupConfig {
                startup_timeout_seconds: self.config.startup_timeout.as_secs(),
                continue_on_warnings: true, // or derive from ProductionConfig if available
                enable_parallel_validation: true, // or derive from ProductionConfig if available
                retry_attempts: 3,          // or derive from ProductionConfig if available
                retry_delay_seconds: 5,     // or derive from ProductionConfig if available
                save_startup_results: false, // or derive from ProductionConfig if available
                results_file_path: "startup_results.json".to_string(), // or derive from ProductionConfig if available
            };
            self.startup_validator = Some(StartupValidator::new(startup_config));
        }

        // Initialize health checker
        if self.config.health_check_enabled {
            self.health_checker = Some(HealthChecker::new(
                self.config.health_check_interval,
                self.config.health_check_timeout,
            ));
        }

        // Initialize monitoring
        if self.config.monitoring_enabled {
            let monitoring_config =
                crate::infrastructure::operations::production::monitoring::MonitoringConfig {
                    collection_interval_seconds: self.config.metrics_collection_interval.as_secs(),
                    enable_detailed_logging: false, // or derive from ProductionConfig if available
                    enable_profiling: false,        // or derive from ProductionConfig if available
                    export_endpoints: vec![],       // or derive from ProductionConfig if available
                    alert_thresholds:
                        crate::infrastructure::operations::production::monitoring::AlertThresholds {
                            cpu_threshold_percent: 80.0,
                            memory_threshold_percent: 85.0,
                            disk_threshold_percent: 90.0,
                            error_rate_threshold: 0.05,
                            response_time_threshold_ms: 1000.0,
                        },
                };
            self.monitor = Some(ProductionMonitor::new(monitoring_config));
        }

        // Initialize graceful shutdown
        if self.config.shutdown_signal_handlers {
            let shutdown_config = crate::infrastructure::operations::production::shutdown::ShutdownConfig {
                grace_period_seconds: self.config.graceful_shutdown_timeout.as_secs(),
                service_timeout_seconds: 30, // or derive from ProductionConfig if available
                save_state_on_shutdown: false, // or derive from ProductionConfig if available
                cleanup_timeout_seconds: 10, // or derive from ProductionConfig if available
                enable_signal_handling: true, // or derive from ProductionConfig if available
                shutdown_order: vec![],      // or derive from ProductionConfig if available
            };
            self.shutdown_handler = Some(GracefulShutdown::new(shutdown_config));
        }

        println!("✅ Production deployment components initialized");
        Ok(())
    }

    /// Start all production services
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting production deployment...");

        // Run startup validation
        if let Some(validator) = &mut self.startup_validator {
            let validation_result = validator.validate_startup().await;
            if !validation_result.success {
                return Err(format!(
                    "Startup validation failed: {:?}",
                    validation_result.critical_failures
                )
                .into());
            }
            println!("✅ Startup validation passed");
        }

        // Start health checking
        if let Some(health_checker) = &mut self.health_checker {
            health_checker.start().await?;
            println!("✅ Health checking started");
        }

        // Start monitoring (if needed)
        if let Some(monitor) = &self.monitor {
            monitor.start_monitoring().await;
            println!("✅ Monitoring started");
        }

        // Setup shutdown handlers
        if let Some(shutdown_handler) = &mut self.shutdown_handler {
            shutdown_handler.setup_signal_handlers().await?;
            println!("✅ Shutdown handlers configured");
        }

        self.status = DeploymentStatus::Healthy;
        println!("🎉 Production deployment started successfully");
        Ok(())
    }

    /// Check current deployment status
    pub async fn check_status(&mut self) -> DeploymentStatus {
        if let Some(health_checker) = &self.health_checker {
            match health_checker.get_latest_status().await {
                HealthStatus::Healthy => {
                    if self.status != DeploymentStatus::ShuttingDown {
                        self.status = DeploymentStatus::Healthy;
                    }
                }
                HealthStatus::Degraded => {
                    self.status = DeploymentStatus::Degraded;
                }
                HealthStatus::Unhealthy => {
                    self.status = DeploymentStatus::Unhealthy;
                }
            }
        }

        self.status.clone()
    }

    /// Get current system metrics
    pub async fn get_metrics(&self) -> Option<SystemMetrics> {
        if let Some(monitor) = &self.monitor {
            Some(monitor.get_system_metrics())
        } else {
            None
        }
    }

    /// Initiate graceful shutdown
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🛑 Initiating graceful shutdown...");
        self.status = DeploymentStatus::ShuttingDown;

        if let Some(shutdown_handler) = &mut self.shutdown_handler {
            use crate::infrastructure::operations::production::ShutdownSignal;
            shutdown_handler
                .initiate_shutdown(ShutdownSignal::Graceful)
                .await?;
        }

        // Stop health checking
        if let Some(health_checker) = &mut self.health_checker {
            health_checker.stop().await?;
            println!("✅ Health checking stopped");
        }

        self.status = DeploymentStatus::Stopped;
        println!("✅ Graceful shutdown completed");
        Ok(())
    }

    pub fn status(&self) -> &DeploymentStatus {
        &self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_production_deployment_creation() {
        let config = ProductionConfig::default();
        let deployment = ProductionDeployment::new(config);
        assert!(matches!(deployment.status(), DeploymentStatus::Starting));
    }

    #[tokio::test]
    async fn test_production_deployment_initialization() {
        let config = ProductionConfig {
            health_check_enabled: false,
            monitoring_enabled: false,
            shutdown_signal_handlers: false,
            startup_validation_enabled: false,
            ..Default::default()
        };

        let mut deployment = ProductionDeployment::new(config);
        let result = deployment.initialize().await;
        assert!(result.is_ok());
    }
}
