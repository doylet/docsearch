# Agent Implementation Guide

**Purpose**: Implementation details and code examples for the agent system.

**Status**: Design Phase
**Last Updated**: December 2025

---

## Implementation Structure

```
agents/
├── Cargo.toml                 # Agent system crate
├── src/
│   ├── main.rs                # CLI entry point
│   ├── lib.rs                 # Public API
│   ├── core/                  # Core framework
│   │   ├── mod.rs
│   │   ├── agent.rs           # Agent trait
│   │   ├── context.rs         # Agent context
│   │   ├── events.rs          # Event definitions
│   │   ├── orchestrator.rs    # Workflow orchestration
│   │   ├── state.rs           # State management
│   │   └── metrics.rs         # Metrics collection
│   ├── agents/                # Agent implementations
│   │   ├── mod.rs
│   │   ├── code_quality/
│   │   ├── testing/
│   │   ├── build/
│   │   ├── schema/
│   │   ├── security/
│   │   ├── documentation/
│   │   ├── deployment/
│   │   ├── performance/
│   │   ├── architecture/
│   │   └── dependency/
│   ├── workflows/             # Workflow definitions
│   │   ├── mod.rs
│   │   ├── pre_commit.rs
│   │   ├── ci_validation.rs
│   │   └── deployment.rs
│   └── cli/                   # CLI interface
│       ├── mod.rs
│       └── commands.rs
└── workflows/                 # YAML workflow definitions
    ├── pre_commit.yaml
    ├── ci_validation.yaml
    └── deployment.yaml
```

---

## Core Framework Implementation

### Agent Trait

```rust
// agents/src/core/agent.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub type AgentResult = Result<AgentExecutionResult, AgentError>;

#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent identifier
    fn id(&self) -> &'static str;

    /// Agent responsibility description
    fn responsibility(&self) -> &'static str;

    /// Execute agent task
    async fn execute(&self, context: AgentContext) -> AgentResult;

    /// Check if agent can handle a given event
    fn can_handle(&self, event: &AgentEvent) -> bool {
        false
    }

    /// Get agent health status
    fn health(&self) -> AgentHealth {
        AgentHealth::healthy()
    }

    /// Get agent metrics
    fn metrics(&self) -> AgentMetrics {
        AgentMetrics::default()
    }

    /// Get agent configuration schema
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    pub agent_id: String,
    pub status: AgentStatus,
    pub duration: Duration,
    pub artifacts: Vec<Artifact>,
    pub metrics: AgentMetrics,
    pub errors: Vec<AgentError>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Success,
    Failure { reason: String },
    Skipped { reason: String },
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub path: std::path::PathBuf,
    pub artifact_type: ArtifactType,
    pub size: Option<u64>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Report,
    Log,
    Binary,
    Documentation,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub execution_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub total_duration: Duration,
    pub custom_metrics: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub severity: RecommendationSeverity,
    pub message: String,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout: {0}")]
    Timeout(Duration),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Dependency error: {0}")]
    DependencyError(String),
}
```

### Agent Context

```rust
// agents/src/core/context.rs

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AgentContext {
    pub workspace: PathBuf,
    pub config: AgentConfig,
    pub previous_results: Arc<RwLock<std::collections::HashMap<String, AgentExecutionResult>>>,
    pub event_bus: Arc<EventBus>,
    pub state: Arc<StateStore>,
    pub logger: Arc<Logger>,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub agent_id: String,
    pub timeout: Option<std::time::Duration>,
    pub retries: usize,
    pub fail_fast: bool,
    pub custom: serde_json::Value,
}

impl AgentContext {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            workspace,
            config: AgentConfig::default(),
            previous_results: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_bus: Arc::new(EventBus::new()),
            state: Arc::new(StateStore::new()),
            logger: Arc::new(Logger::new()),
        }
    }

    pub async fn get_previous_result(&self, agent_id: &str) -> Option<AgentExecutionResult> {
        self.previous_results.read().await.get(agent_id).cloned()
    }

    pub async fn publish_event(&self, event: AgentEvent) -> Result<()> {
        self.event_bus.publish(event).await
    }
}
```

---

## Example Agent Implementation

### Code Quality Agent

```rust
// agents/src/agents/code_quality/mod.rs

use crate::core::{Agent, AgentContext, AgentResult, AgentExecutionResult, AgentStatus};
use std::process::Command;
use std::time::Instant;

pub struct CodeQualityAgent {
    config: CodeQualityConfig,
}

pub struct CodeQualityConfig {
    pub workspace: std::path::PathBuf,
    pub strict: bool,
    pub auto_format: bool,
    pub check_typescript: bool,
}

impl CodeQualityAgent {
    pub fn new(config: CodeQualityConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Agent for CodeQualityAgent {
    fn id(&self) -> &'static str {
        "code-quality"
    }

    fn responsibility(&self) -> &'static str {
        "Ensure code formatting, linting, and static analysis compliance"
    }

    async fn execute(&self, context: AgentContext) -> AgentResult {
        let start = Instant::now();

        // 1. Format Rust code
        let format_result = self.format_rust(&context).await?;

        // 2. Lint Rust code
        let rust_lint_result = self.lint_rust(&context).await?;

        // 3. Lint TypeScript code (if enabled)
        let ts_lint_result = if self.config.check_typescript {
            self.lint_typescript(&context).await?
        } else {
            LintResult::skipped()
        };

        // 4. Generate report
        let report = QualityReport {
            formatting: format_result,
            rust_linting: rust_lint_result,
            typescript_linting: ts_lint_result,
            quality_score: self.calculate_quality_score(&format_result, &rust_lint_result, &ts_lint_result),
        };

        let duration = start.elapsed();

        Ok(AgentExecutionResult {
            agent_id: self.id().to_string(),
            status: if report.quality_score >= 80.0 {
                AgentStatus::Success
            } else {
                AgentStatus::Failure {
                    reason: format!("Quality score {} below threshold", report.quality_score),
                }
            },
            duration,
            artifacts: vec![
                Artifact {
                    name: "quality_report.json".to_string(),
                    path: context.workspace.join("target/agents/quality_report.json"),
                    artifact_type: ArtifactType::Report,
                    size: None,
                    metadata: serde_json::to_value(&report)?,
                }
            ],
            metrics: AgentMetrics {
                execution_count: 1,
                success_count: if report.quality_score >= 80.0 { 1 } else { 0 },
                failure_count: if report.quality_score < 80.0 { 1 } else { 0 },
                total_duration: duration,
                custom_metrics: std::collections::HashMap::from([
                    ("quality_score".to_string(), report.quality_score),
                    ("format_violations".to_string(), format_result.violations as f64),
                    ("lint_errors".to_string(), rust_lint_result.errors as f64),
                ]),
            },
            errors: vec![],
            warnings: vec![],
            recommendations: vec![],
        })
    }
}

impl CodeQualityAgent {
    async fn format_rust(&self, context: &AgentContext) -> Result<FormatResult> {
        let output = Command::new("cargo")
            .arg("fmt")
            .arg("--all")
            .arg("--")
            .arg("--check")
            .current_dir(&context.workspace)
            .output()
            .await?;

        if output.status.success() {
            Ok(FormatResult::success())
        } else {
            let violations = self.parse_format_violations(&String::from_utf8_lossy(&output.stderr));
            Ok(FormatResult::with_violations(violations))
        }
    }

    async fn lint_rust(&self, context: &AgentContext) -> Result<LintResult> {
        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--all-targets")
            .arg("--all-features")
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .current_dir(&context.workspace)
            .output()
            .await?;

        if output.status.success() {
            Ok(LintResult::success())
        } else {
            let (errors, warnings) = self.parse_lint_output(&String::from_utf8_lossy(&output.stderr));
            Ok(LintResult::with_issues(errors, warnings))
        }
    }

    async fn lint_typescript(&self, context: &AgentContext) -> Result<LintResult> {
        let output = Command::new("npm")
            .arg("run")
            .arg("lint")
            .current_dir(&context.workspace)
            .output()
            .await?;

        if output.status.success() {
            Ok(LintResult::success())
        } else {
            let (errors, warnings) = self.parse_eslint_output(&String::from_utf8_lossy(&output.stderr));
            Ok(LintResult::with_issues(errors, warnings))
        }
    }

    fn calculate_quality_score(
        &self,
        format: &FormatResult,
        rust_lint: &LintResult,
        ts_lint: &LintResult,
    ) -> f64 {
        let mut score = 100.0;

        // Deduct points for violations
        score -= format.violations as f64 * 2.0;
        score -= rust_lint.errors as f64 * 5.0;
        score -= rust_lint.warnings as f64 * 1.0;
        score -= ts_lint.errors as f64 * 5.0;
        score -= ts_lint.warnings as f64 * 1.0;

        score.max(0.0).min(100.0)
    }
}
```

---

## Workflow Orchestrator

```rust
// agents/src/core/orchestrator.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct WorkflowOrchestrator {
    agents: Arc<RwLock<HashMap<String, Arc<dyn Agent>>>>,
    workflows: Arc<RwLock<HashMap<String, WorkflowDefinition>>>,
    event_bus: Arc<EventBus>,
    state_store: Arc<StateStore>,
}

impl WorkflowOrchestrator {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            workflows: Arc::new(RwLock::new(HashMap::new())),
            event_bus: Arc::new(EventBus::new()),
            state_store: Arc::new(StateStore::new()),
        }
    }

    pub async fn register_agent(&self, agent: Arc<dyn Agent>) {
        let agent_id = agent.id().to_string();
        self.agents.write().await.insert(agent_id, agent);
    }

    pub async fn register_workflow(&self, workflow: WorkflowDefinition) {
        let workflow_id = workflow.id.clone();
        self.workflows.write().await.insert(workflow_id, workflow);
    }

    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        context: AgentContext,
    ) -> Result<WorkflowResult> {
        let workflow = self.workflows.read().await
            .get(workflow_id)
            .ok_or_else(|| AgentError::ConfigurationError(format!("Workflow not found: {}", workflow_id)))?
            .clone();

        // Build execution graph
        let graph = self.build_execution_graph(&workflow)?;

        // Execute agents in dependency order
        let mut results = HashMap::new();
        for level in graph.levels() {
            // Execute agents in parallel at this level
            let level_results = self.execute_level(level, &context).await?;

            // Store results
            for (agent_id, result) in level_results {
                results.insert(agent_id, result.clone());

                // Update state store
                self.state_store.set_agent_state(&agent_id, result).await;

                // Publish event
                self.event_bus.publish(AgentEvent::AgentCompleted {
                    agent_id: agent_id.clone(),
                    result: result.clone(),
                }).await?;
            }

            // Check for failures
            if workflow.fail_fast {
                if let Some(failure) = results.values().find(|r| matches!(r.status, AgentStatus::Failure { .. })) {
                    return Err(AgentError::ExecutionFailed(format!(
                        "Workflow failed at agent: {}",
                        failure.agent_id
                    )));
                }
            }
        }

        Ok(WorkflowResult {
            workflow_id: workflow_id.to_string(),
            agents_executed: results.len(),
            agents_succeeded: results.values().filter(|r| matches!(r.status, AgentStatus::Success)).count(),
            agents_failed: results.values().filter(|r| matches!(r.status, AgentStatus::Failure { .. })).count(),
            results,
        })
    }

    async fn execute_level(
        &self,
        agent_ids: Vec<String>,
        context: &AgentContext,
    ) -> Result<HashMap<String, AgentExecutionResult>> {
        let agents = self.agents.read().await;

        let tasks: Vec<_> = agent_ids.into_iter()
            .filter_map(|agent_id| {
                agents.get(&agent_id).map(|agent| {
                    let agent = agent.clone();
                    let context = context.clone();
                    let agent_id = agent_id.clone();
                    tokio::spawn(async move {
                        (agent_id, agent.execute(context).await)
                    })
                })
            })
            .collect();

        let mut results = HashMap::new();
        for task in tasks {
            let (agent_id, result) = task.await?;
            match result {
                Ok(result) => {
                    results.insert(agent_id, result);
                },
                Err(e) => {
                    results.insert(agent_id.clone(), AgentExecutionResult {
                        agent_id,
                        status: AgentStatus::Failure { reason: e.to_string() },
                        duration: std::time::Duration::ZERO,
                        artifacts: vec![],
                        metrics: AgentMetrics::default(),
                        errors: vec![e],
                        warnings: vec![],
                        recommendations: vec![],
                    });
                },
            }
        }

        Ok(results)
    }

    fn build_execution_graph(&self, workflow: &WorkflowDefinition) -> Result<ExecutionGraph> {
        // Build dependency graph from workflow definition
        // ...
        Ok(ExecutionGraph::new())
    }
}
```

---

## CLI Interface

```rust
// agents/src/cli/commands.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent")]
#[command(about = "Zero-Latency Agent System")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a single agent
    Run {
        /// Agent ID
        agent_id: String,

        /// Workspace path
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Agent-specific options
        #[arg(long)]
        options: Vec<String>,
    },

    /// Execute a workflow
    Workflow {
        /// Workflow ID
        workflow_id: String,

        /// Workspace path
        #[arg(short, long, default_value = ".")]
        workspace: std::path::PathBuf,

        /// Environment
        #[arg(short, long)]
        env: Option<String>,
    },

    /// List available agents
    ListAgents,

    /// List available workflows
    ListWorkflows,

    /// Get agent status
    Status {
        /// Agent ID
        agent_id: String,
    },
}
```

---

## Next Steps

1. **Implement Core Framework**: Start with `Agent` trait and `AgentContext`
2. **Implement First Agent**: Start with Code Quality Agent as reference
3. **Implement Orchestrator**: Build workflow execution engine
4. **Add CLI**: Create command-line interface
5. **Add Tests**: Comprehensive test coverage
6. **Add Documentation**: Complete API documentation

---

**Status**: Implementation guide complete. Ready for development phase.
