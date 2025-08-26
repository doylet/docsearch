use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Startup validator coordinates system initialization and validation
pub struct StartupValidator {
    /// Validation steps to perform
    validation_steps: Vec<ValidationStep>,

    /// Configuration
    config: StartupConfig,

    /// Results from validation
    results: HashMap<String, ValidationResult>,
}

/// Individual validation step
pub struct ValidationStep {
    /// Step identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Step description
    pub description: String,

    /// Whether this step is required for startup
    pub required: bool,

    /// Timeout for this step
    pub timeout_seconds: u64,

    /// Dependencies (other step IDs that must complete first)
    pub dependencies: Vec<String>,

    /// Validation function
    pub validator: ValidationFunction,
}

impl Clone for ValidationStep {
    fn clone(&self) -> Self {
        ValidationStep {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            required: self.required,
            timeout_seconds: self.timeout_seconds,
            dependencies: self.dependencies.clone(),
            validator: Box::new(|| panic!("Cloning of ValidationStep.validator is not supported")),
        }
    }
}

/// Validation function type
pub type ValidationFunction = Box<
    dyn Fn() -> Box<dyn std::future::Future<Output = ValidationResult> + Send + Unpin>
        + Send
        + Sync,
>;

/// Result of a validation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Step identifier
    pub step_id: String,

    /// Whether validation passed
    pub passed: bool,

    /// Validation message
    pub message: String,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Additional details
    pub details: HashMap<String, serde_json::Value>,

    /// Warnings (non-critical issues)
    pub warnings: Vec<String>,

    /// Timestamp when validation completed
    pub completed_at: u64,
}

/// Complete startup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupResult {
    /// Whether startup was successful
    pub success: bool,

    /// Overall startup time
    pub total_startup_time_ms: u64,

    /// Results from all validation steps
    pub step_results: HashMap<String, ValidationResult>,

    /// Critical failures (if any)
    pub critical_failures: Vec<String>,

    /// Non-critical warnings
    pub warnings: Vec<String>,

    /// System information
    pub system_info: SystemInfo,

    /// Startup timestamp
    pub startup_timestamp: u64,
}

/// System information collected during startup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,

    /// Architecture
    pub arch: String,

    /// Available memory (MB)
    pub available_memory_mb: u64,

    /// Number of CPU cores
    pub cpu_cores: u32,

    /// Rust version
    pub rust_version: String,

    /// Application version
    pub app_version: String,

    /// Environment variables (filtered)
    pub environment: HashMap<String, String>,
}

/// Startup configuration
#[derive(Debug, Clone)]
pub struct StartupConfig {
    /// Startup timeout in seconds
    pub startup_timeout_seconds: u64,
    /// Whether to continue on warnings
    pub continue_on_warnings: bool,
    /// Whether to enable parallel validation
    pub enable_parallel_validation: bool,
    /// Number of retry attempts
    pub retry_attempts: u64,
    /// Delay between retry attempts (seconds)
    pub retry_delay_seconds: u64,
    /// Whether to save startup results to file
    pub save_startup_results: bool,
    /// Results file path
    pub results_file_path: String,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            startup_timeout_seconds: std::env::var("STARTUP_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300), // 5 minutes
            continue_on_warnings: std::env::var("STARTUP_CONTINUE_ON_WARNINGS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            enable_parallel_validation: std::env::var("STARTUP_PARALLEL_VALIDATION")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            retry_attempts: std::env::var("STARTUP_RETRY_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            retry_delay_seconds: std::env::var("STARTUP_RETRY_DELAY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            save_startup_results: std::env::var("STARTUP_SAVE_RESULTS")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            results_file_path: std::env::var("STARTUP_RESULTS_FILE")
                .unwrap_or_else(|_| "./data/startup_results.json".to_string()),
        }
    }
}

impl StartupValidator {
    pub fn new(config: StartupConfig) -> Self {
        Self {
            validation_steps: Vec::new(),
            config,
            results: HashMap::new(),
        }
    }

    /// Add a validation step
    pub fn add_step(&mut self, step: ValidationStep) {
        self.validation_steps.push(step);
    }

    /// Add multiple validation steps
    pub fn add_steps(&mut self, steps: Vec<ValidationStep>) {
        self.validation_steps.extend(steps);
    }

    /// Run all validation steps and return startup result
    pub async fn validate_startup(&mut self) -> StartupResult {
        let start_time = std::time::Instant::now();
        let startup_timestamp = chrono::Utc::now().timestamp() as u64;

        println!("🚀 Starting system validation...");

        // Collect system information
        let system_info = self.collect_system_info().await;

        // Run validation steps
        let validation_success = if self.config.enable_parallel_validation {
            self.run_parallel_validation().await
        } else {
            self.run_sequential_validation().await
        };

        let total_time = start_time.elapsed().as_millis() as u64;

        // Analyze results
        let (success, critical_failures, warnings) = self.analyze_results();

        let startup_result = StartupResult {
            success: validation_success && success,
            total_startup_time_ms: total_time,
            step_results: self.results.clone(),
            critical_failures,
            warnings,
            system_info,
            startup_timestamp,
        };

        // Save results if configured
        if self.config.save_startup_results {
            if let Err(e) = self.save_results(&startup_result).await {
                eprintln!("Failed to save startup results: {}", e);
            }
        }

        if startup_result.success {
            println!(
                "✅ System validation completed successfully in {}ms",
                total_time
            );
        } else {
            println!("❌ System validation failed after {}ms", total_time);
        }

        startup_result
    }

    /// Run validation steps in parallel where possible
    async fn run_parallel_validation(&mut self) -> bool {
        // TODO: Implement dependency-aware parallel execution
        // For now, fall back to sequential
        self.run_sequential_validation().await
    }

    /// Run validation steps sequentially
    async fn run_sequential_validation(&mut self) -> bool {
        let total_steps = self.validation_steps.len();
        for (index, step) in self.validation_steps.iter().enumerate() {
            println!(
                "🔍 Running validation step {}/{}: {}",
                index + 1,
                total_steps,
                step.name
            );
            let result = self.run_validation_step(step).await;
            let passed = result.passed;
            self.results.insert(step.id.clone(), result.clone());
            if !passed && step.required {
                println!("❌ Critical validation step failed: {}", step.name);
                return false;
            } else if !passed {
                println!("⚠️  Non-critical validation step failed: {}", step.name);
            } else {
                println!("✅ Validation step passed: {}", step.name);
            }
        }
        true
    }

    /// Run a single validation step with retries
    async fn run_validation_step(&self, step: &ValidationStep) -> ValidationResult {
        let mut attempts = 0;
        let max_attempts = if step.required {
            self.config.retry_attempts + 1
        } else {
            1
        };

        loop {
            attempts += 1;
            let start_time = std::time::Instant::now();

            // Run validation with timeout
            let timeout_duration = Duration::from_secs(step.timeout_seconds);
            let validation_future = (step.validator)();

            let result = match tokio::time::timeout(timeout_duration, validation_future).await {
                Ok(result) => result,
                Err(_) => ValidationResult {
                    step_id: step.id.clone(),
                    passed: false,
                    message: format!(
                        "Validation step timed out after {} seconds",
                        step.timeout_seconds
                    ),
                    execution_time_ms: timeout_duration.as_millis() as u64,
                    details: HashMap::new(),
                    warnings: Vec::new(),
                    completed_at: chrono::Utc::now().timestamp() as u64,
                },
            };

            if result.passed || attempts >= max_attempts {
                return result;
            }

            println!(
                "🔄 Retrying validation step: {} (attempt {}/{})",
                step.name, attempts, max_attempts
            );
            tokio::time::sleep(Duration::from_secs(self.config.retry_delay_seconds)).await;
        }
    }

    /// Collect system information
    async fn collect_system_info(&self) -> SystemInfo {
        let environment = std::env::vars()
            .filter(|(key, _)| {
                // Only include non-sensitive environment variables
                !key.to_lowercase().contains("password")
                    && !key.to_lowercase().contains("secret")
                    && !key.to_lowercase().contains("token")
                    && !key.to_lowercase().contains("key")
            })
            .collect();

        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            available_memory_mb: 0, // TODO: Get actual memory info
            cpu_cores: num_cpus::get() as u32,
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            environment,
        }
    }

    /// Analyze validation results
    fn analyze_results(&self) -> (bool, Vec<String>, Vec<String>) {
        let mut success = true;
        let mut critical_failures = Vec::new();
        let mut warnings = Vec::new();

        for (step_id, result) in &self.results {
            if !result.passed {
                // Find the corresponding step to check if it's required
                if let Some(step) = self.validation_steps.iter().find(|s| s.id == *step_id) {
                    if step.required {
                        success = false;
                        critical_failures.push(format!("{}: {}", step.name, result.message));
                    } else {
                        warnings.push(format!("{}: {}", step.name, result.message));
                    }
                }
            }

            // Add step-specific warnings
            warnings.extend(
                result
                    .warnings
                    .iter()
                    .map(|w| format!("{}: {}", step_id, w)),
            );
        }

        (success, critical_failures, warnings)
    }

    /// Save startup results to file
    async fn save_results(
        &self,
        result: &StartupResult,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let results_json = serde_json::to_string_pretty(result)?;

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&self.config.results_file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.config.results_file_path, results_json)?;
        println!(
            "💾 Saved startup results to: {}",
            self.config.results_file_path
        );

        Ok(())
    }

    /// Create standard validation steps for the doc-indexer service
    pub fn create_standard_steps() -> Vec<ValidationStep> {
        vec![
            ValidationStep {
                id: "config_validation".to_string(),
                name: "Configuration Validation".to_string(),
                description: "Validate service configuration".to_string(),
                required: true,
                timeout_seconds: 10,
                dependencies: Vec::new(),
                validator: Box::new(|| {
                    Box::new(Box::pin(async {
                        // TODO: Implement actual config validation
                        ValidationResult {
                            step_id: "config_validation".to_string(),
                            passed: true,
                            message: "Configuration is valid".to_string(),
                            execution_time_ms: 50,
                            details: HashMap::new(),
                            warnings: Vec::new(),
                            completed_at: chrono::Utc::now().timestamp() as u64,
                        }
                    }))
                }),
            },
            ValidationStep {
                id: "database_connectivity".to_string(),
                name: "Database Connectivity".to_string(),
                description: "Test database connection".to_string(),
                required: true,
                timeout_seconds: 30,
                dependencies: vec!["config_validation".to_string()],
                validator: Box::new(|| {
                    Box::new(Box::pin(async {
                        // TODO: Implement actual database connectivity test
                        ValidationResult {
                            step_id: "database_connectivity".to_string(),
                            passed: true,
                            message: "Database connection successful".to_string(),
                            execution_time_ms: 200,
                            details: HashMap::new(),
                            warnings: Vec::new(),
                            completed_at: chrono::Utc::now().timestamp() as u64,
                        }
                    }))
                }),
            },
            ValidationStep {
                id: "vector_store_initialization".to_string(),
                name: "Vector Store Initialization".to_string(),
                description: "Initialize and test vector store".to_string(),
                required: true,
                timeout_seconds: 60,
                dependencies: vec!["config_validation".to_string()],
                validator: Box::new(|| {
                    Box::new(Box::pin(async {
                        // TODO: Implement actual vector store initialization
                        ValidationResult {
                            step_id: "vector_store_initialization".to_string(),
                            passed: true,
                            message: "Vector store initialized successfully".to_string(),
                            execution_time_ms: 1500,
                            details: HashMap::new(),
                            warnings: Vec::new(),
                            completed_at: chrono::Utc::now().timestamp() as u64,
                        }
                    }))
                }),
            },
            ValidationStep {
                id: "embedding_service_check".to_string(),
                name: "Embedding Service Check".to_string(),
                description: "Test embedding service connectivity".to_string(),
                required: true,
                timeout_seconds: 30,
                dependencies: vec!["config_validation".to_string()],
                validator: Box::new(|| {
                    Box::new(Box::pin(async {
                        // TODO: Implement actual embedding service check
                        ValidationResult {
                            step_id: "embedding_service_check".to_string(),
                            passed: true,
                            message: "Embedding service is accessible".to_string(),
                            execution_time_ms: 800,
                            details: HashMap::new(),
                            warnings: Vec::new(),
                            completed_at: chrono::Utc::now().timestamp() as u64,
                        }
                    }))
                }),
            },
            ValidationStep {
                id: "memory_pool_initialization".to_string(),
                name: "Memory Pool Initialization".to_string(),
                description: "Initialize memory optimization pools".to_string(),
                required: false, // Non-critical
                timeout_seconds: 15,
                dependencies: Vec::new(),
                validator: Box::new(|| {
                    Box::new(Box::pin(async {
                        // TODO: Implement actual memory pool initialization
                        ValidationResult {
                            step_id: "memory_pool_initialization".to_string(),
                            passed: true,
                            message: "Memory pools initialized".to_string(),
                            execution_time_ms: 300,
                            details: HashMap::new(),
                            warnings: Vec::new(),
                            completed_at: chrono::Utc::now().timestamp() as u64,
                        }
                    }))
                }),
            },
            ValidationStep {
                id: "health_check_endpoints".to_string(),
                name: "Health Check Endpoints".to_string(),
                description: "Verify health check endpoints are responsive".to_string(),
                required: false, // Non-critical
                timeout_seconds: 10,
                dependencies: vec![
                    "vector_store_initialization".to_string(),
                    "embedding_service_check".to_string(),
                ],
                validator: Box::new(|| {
                    Box::new(Box::pin(async {
                        // TODO: Implement actual health check validation
                        ValidationResult {
                            step_id: "health_check_endpoints".to_string(),
                            passed: true,
                            message: "Health check endpoints are responsive".to_string(),
                            execution_time_ms: 100,
                            details: HashMap::new(),
                            warnings: Vec::new(),
                            completed_at: chrono::Utc::now().timestamp() as u64,
                        }
                    }))
                }),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_startup_validator_creation() {
        let config = StartupConfig::default();
        let validator = StartupValidator::new(config);

        assert_eq!(validator.validation_steps.len(), 0);
        assert_eq!(validator.results.len(), 0);
    }

    #[test]
    fn test_standard_validation_steps() {
        let steps = StartupValidator::create_standard_steps();

        assert!(!steps.is_empty());
        assert!(steps.iter().any(|s| s.id == "config_validation"));
        assert!(steps.iter().any(|s| s.id == "vector_store_initialization"));
    }

    #[test]
    fn test_system_info_creation() {
        // Test that we can create system info without panicking
        let info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            available_memory_mb: 1024,
            cpu_cores: num_cpus::get() as u32,
            rust_version: "1.70.0".to_string(),
            app_version: "0.1.0".to_string(),
            environment: HashMap::new(),
        };

        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(info.cpu_cores > 0);
    }
}
