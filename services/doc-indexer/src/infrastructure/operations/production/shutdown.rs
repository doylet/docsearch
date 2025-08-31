use serde::{Deserialize, Serialize};
/// Graceful Shutdown System
///
/// Handles graceful shutdown of the production service with proper cleanup
/// and signal handling.
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

/// Graceful shutdown coordinator
pub struct GracefulShutdown {
    /// Shutdown state
    state: Arc<RwLock<ShutdownState>>,

    /// Shutdown signal sender
    shutdown_sender: broadcast::Sender<ShutdownSignal>,

    /// Configuration
    config: ShutdownConfig,
}

/// Shutdown state tracking
#[derive(Debug, Clone)]
pub struct ShutdownState {
    /// Whether shutdown has been initiated
    pub shutdown_initiated: bool,

    /// Current shutdown phase
    pub current_phase: ShutdownPhase,

    /// Services that have completed shutdown
    pub completed_services: Vec<String>,

    /// Services still shutting down
    pub pending_services: Vec<String>,

    /// Shutdown start time
    pub shutdown_started_at: Option<u64>,

    /// Whether forced shutdown is active
    pub forced_shutdown: bool,
}

/// Shutdown phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownPhase {
    /// Normal operation
    Running,

    /// Graceful shutdown initiated
    Graceful,

    /// Draining connections
    Draining,

    /// Stopping services
    Stopping,

    /// Cleanup phase
    Cleanup,

    /// Shutdown complete
    Complete,

    /// Forced shutdown
    Forced,
}

/// Shutdown signal types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownSignal {
    /// Graceful shutdown requested
    Graceful,

    /// Immediate shutdown requested
    Immediate,

    /// Force shutdown (emergency)
    Force,

    /// Restart requested
    Restart,
}

/// Shutdown configuration
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Grace period for graceful shutdown (seconds)
    pub grace_period_seconds: u64,

    /// Timeout for service shutdown (seconds)
    pub service_timeout_seconds: u64,

    /// Whether to save state before shutdown
    pub save_state_on_shutdown: bool,

    /// Cleanup timeout (seconds)
    pub cleanup_timeout_seconds: u64,

    /// Signal handling enabled
    pub enable_signal_handling: bool,

    /// Services to shutdown in order
    pub shutdown_order: Vec<String>,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            grace_period_seconds: std::env::var("SHUTDOWN_GRACE_PERIOD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            service_timeout_seconds: std::env::var("SHUTDOWN_SERVICE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            save_state_on_shutdown: std::env::var("SHUTDOWN_SAVE_STATE")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            cleanup_timeout_seconds: std::env::var("SHUTDOWN_CLEANUP_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15),
            enable_signal_handling: std::env::var("SHUTDOWN_ENABLE_SIGNAL_HANDLING")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            shutdown_order: std::env::var("SHUTDOWN_ORDER")
                .ok()
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|| {
                    vec![
                        "http_server".to_string(),
                        "search_service".to_string(),
                        "vector_store".to_string(),
                        "embedding_service".to_string(),
                    ]
                }),
        }
    }
}

impl GracefulShutdown {
    pub fn new(config: ShutdownConfig) -> Self {
        let (shutdown_sender, _) = broadcast::channel(16);

        Self {
            state: Arc::new(RwLock::new(ShutdownState {
                shutdown_initiated: false,
                current_phase: ShutdownPhase::Running,
                completed_services: Vec::new(),
                pending_services: config.shutdown_order.clone(),
                shutdown_started_at: None,
                forced_shutdown: false,
            })),
            shutdown_sender,
            config,
        }
    }

    /// Get a shutdown signal receiver
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownSignal> {
        self.shutdown_sender.subscribe()
    }

    /// Initiate graceful shutdown
    pub async fn initiate_shutdown(
        &self,
        signal: ShutdownSignal,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.write().await;

        if state.shutdown_initiated {
            return Ok(()); // Already shutting down
        }

        state.shutdown_initiated = true;
        state.current_phase = match signal {
            ShutdownSignal::Force => ShutdownPhase::Forced,
            _ => ShutdownPhase::Graceful,
        };
        state.shutdown_started_at = Some(chrono::Utc::now().timestamp() as u64);

        drop(state);

        // Send shutdown signal to all subscribers
        let _ = self.shutdown_sender.send(signal.clone());

        println!("🛑 Initiating shutdown: {:?}", signal);

        // Start shutdown process
        self.execute_shutdown(signal).await
    }

    /// Execute the shutdown process
    async fn execute_shutdown(
        &self,
        signal: ShutdownSignal,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match signal {
            ShutdownSignal::Force => self.force_shutdown().await,
            ShutdownSignal::Immediate => self.immediate_shutdown().await,
            _ => self.graceful_shutdown().await,
        }
    }

    /// Perform graceful shutdown
    async fn graceful_shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔄 Starting graceful shutdown...");

        // Phase 1: Drain connections
        self.update_phase(ShutdownPhase::Draining).await;
        self.drain_connections().await?;

        // Phase 2: Stop services in order
        self.update_phase(ShutdownPhase::Stopping).await;
        self.stop_services().await?;

        // Phase 3: Cleanup
        self.update_phase(ShutdownPhase::Cleanup).await;
        self.cleanup_resources().await?;

        // Phase 4: Complete
        self.update_phase(ShutdownPhase::Complete).await;

        println!("✅ Graceful shutdown completed");
        Ok(())
    }

    /// Perform immediate shutdown
    async fn immediate_shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("⚡ Starting immediate shutdown...");

        // Skip draining, go straight to stopping services
        self.update_phase(ShutdownPhase::Stopping).await;
        self.stop_services_immediate().await?;

        self.update_phase(ShutdownPhase::Complete).await;

        println!("✅ Immediate shutdown completed");
        Ok(())
    }

    /// Perform forced shutdown
    async fn force_shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("💥 Starting forced shutdown...");

        {
            let mut state = self.state.write().await;
            state.forced_shutdown = true;
            state.current_phase = ShutdownPhase::Forced;
        }

        // Force stop everything immediately
        self.force_stop_services().await?;

        self.update_phase(ShutdownPhase::Complete).await;

        println!("✅ Forced shutdown completed");
        Ok(())
    }

    /// Drain active connections
    async fn drain_connections(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚰 Draining active connections...");

        // Wait for connections to drain naturally or timeout
        let timeout = Duration::from_secs(self.config.grace_period_seconds);
        tokio::time::timeout(timeout, async {
            // TODO: Implement actual connection draining
            tokio::time::sleep(Duration::from_secs(2)).await;
        })
        .await
        .map_err(|_| "Timeout while draining connections")?;

        Ok(())
    }

    /// Stop services in configured order
    async fn stop_services(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let shutdown_order = self.config.shutdown_order.clone();

        for service_name in shutdown_order {
            println!("🛑 Stopping service: {}", service_name);

            // Stop the service with timeout
            let timeout = Duration::from_secs(self.config.service_timeout_seconds);
            let result = tokio::time::timeout(timeout, self.stop_service(&service_name)).await;

            match result {
                Ok(Ok(_)) => {
                    self.mark_service_completed(&service_name).await;
                    println!("✅ Service stopped: {}", service_name);
                }
                Ok(Err(e)) => {
                    eprintln!("❌ Error stopping service {}: {}", service_name, e);
                }
                Err(_) => {
                    eprintln!("⏰ Timeout stopping service: {}", service_name);
                }
            }
        }

        Ok(())
    }

    /// Stop services immediately without graceful handling
    async fn stop_services_immediate(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let shutdown_order = self.config.shutdown_order.clone();

        for service_name in shutdown_order {
            println!("⚡ Force stopping service: {}", service_name);
            self.force_stop_service(&service_name).await?;
            self.mark_service_completed(&service_name).await;
        }

        Ok(())
    }

    /// Force stop all services
    async fn force_stop_services(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let shutdown_order = self.config.shutdown_order.clone();

        // Stop all services in parallel for force shutdown
        let futures: Vec<_> = shutdown_order
            .iter()
            .map(|service_name| {
                let name = service_name.clone();
                async move {
                    println!("💥 Force stopping service: {}", name);
                    // TODO: Implement actual force stop
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
            })
            .collect();

        futures::future::try_join_all(futures).await?;

        Ok(())
    }

    /// Stop a specific service
    async fn stop_service(
        &self,
        service_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement actual service stopping based on service type
        match service_name {
            "http_server" => {
                // Stop HTTP server
            }
            "search_service" => {
                // Stop search service
            }
            "vector_store" => {
                // Stop vector store
            }
            "embedding_service" => {
                // Stop embedding service
            }
            _ => {
                println!("Unknown service: {}", service_name);
            }
        }

        Ok(())
    }

    /// Force stop a specific service
    async fn force_stop_service(
        &self,
        service_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement actual force stopping
        Ok(())
    }

    /// Cleanup resources
    async fn cleanup_resources(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🧹 Cleaning up resources...");

        let timeout = Duration::from_secs(self.config.cleanup_timeout_seconds);
        tokio::time::timeout(timeout, async {
            // Save state if configured
            if self.config.save_state_on_shutdown {
                self.save_state().await?;
            }

            // Cleanup temporary files, flush caches, etc.
            // TODO: Implement actual cleanup

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        })
        .await
        .map_err(|_| "Cleanup timeout")??;

        Ok(())
    }

    /// Save application state
    async fn save_state(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("💾 Saving application state...");

        // TODO: Implement state saving
        // - Save vector store indices
        // - Save configuration
        // - Save metrics
        // - Save active sessions

        Ok(())
    }

    /// Update the current shutdown phase
    async fn update_phase(&self, phase: ShutdownPhase) {
        let mut state = self.state.write().await;
        state.current_phase = phase;
    }

    /// Mark a service as completed shutdown
    async fn mark_service_completed(&self, service_name: &str) {
        let mut state = self.state.write().await;
        state.completed_services.push(service_name.to_string());
        state.pending_services.retain(|s| s != service_name);
    }

    /// Get current shutdown state
    pub async fn get_state(&self) -> ShutdownState {
        self.state.read().await.clone()
    }

    /// Check if shutdown is complete
    pub async fn is_shutdown_complete(&self) -> bool {
        let state = self.state.read().await;
        matches!(state.current_phase, ShutdownPhase::Complete)
    }

    /// Setup signal handlers for Unix signals
    pub async fn setup_signal_handlers(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enable_signal_handling {
            return Ok(());
        }

        #[cfg(unix)]
        {
            use tokio::signal;

            let shutdown = self.clone();
            tokio::spawn(async move {
                let mut sigterm =
                    signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
                let mut sigint =
                    signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();

                tokio::select! {
                    _ = sigterm.recv() => {
                        println!("📡 Received SIGTERM, initiating graceful shutdown");
                        let _ = shutdown.initiate_shutdown(ShutdownSignal::Graceful).await;
                    },
                    _ = sigint.recv() => {
                        println!("📡 Received SIGINT, initiating graceful shutdown");
                        let _ = shutdown.initiate_shutdown(ShutdownSignal::Graceful).await;
                    }
                }
            });
        }

        Ok(())
    }
}

// Clone implementation for tokio::spawn
impl Clone for GracefulShutdown {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            shutdown_sender: self.shutdown_sender.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_initialization() {
        let config = ShutdownConfig::default();
        let shutdown = GracefulShutdown::new(config);

        let state = shutdown.get_state().await;
        assert!(!state.shutdown_initiated);
        assert!(matches!(state.current_phase, ShutdownPhase::Running));
    }

    #[tokio::test]
    async fn test_shutdown_signal_subscription() {
        let config = ShutdownConfig::default();
        let shutdown = GracefulShutdown::new(config);

        let mut receiver = shutdown.subscribe();

        // Test that we can subscribe without panic
        assert!(receiver.try_recv().is_err()); // Should be empty initially
    }
}
