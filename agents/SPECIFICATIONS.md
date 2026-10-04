# Agent Specifications

**Purpose**: Detailed specifications for each discrete agent in the product engineering system.

**Status**: Design Phase
**Last Updated**: December 2025

---

## Agent Specification Template

Each agent follows this structure:

```yaml
agent:
  id: "agent-identifier"
  name: "Human Readable Name"
  responsibility: "Single, clear responsibility statement"
  scope: "What the agent controls"
  boundaries: "What the agent does NOT control"

  capabilities:
    - "Capability 1"
    - "Capability 2"

  inputs:
    - name: "input-name"
      type: "input-type"
      required: true
      description: "Input description"

  outputs:
    - name: "output-name"
      type: "output-type"
      description: "Output description"

  metrics:
    - name: "metric-name"
      type: "counter|gauge|histogram"
      description: "Metric description"

  dependencies:
    - "other-agent-id"

  triggers:
    - event: "trigger-event"
      condition: "trigger-condition"
```

---

## 1. Code Quality Agent

**ID**: `code-quality`
**Responsibility**: Ensure code formatting, linting, and static analysis compliance across the codebase.

### Scope
- Rust code formatting (`cargo fmt`)
- Rust linting (`cargo clippy`)
- TypeScript/JavaScript linting (ESLint)
- Static analysis (complexity, maintainability)
- Code style enforcement

### Boundaries
- Does NOT run tests (Testing Agent)
- Does NOT fix code automatically (only reports)
- Does NOT manage dependencies (Dependency Agent)

### Capabilities
- Format Rust code with `cargo fmt`
- Lint Rust code with `cargo clippy`
- Lint TypeScript/JavaScript with ESLint
- Calculate code complexity metrics
- Detect code smells
- Generate code quality reports

### Inputs
- `workspace`: Path to workspace root (required)
- `scope`: Files/directories to check (optional, defaults to all)
- `strict`: Fail on warnings (optional, default: false)
- `format`: Auto-format code (optional, default: false)

### Outputs
- `format_report`: Formatting violations report
- `lint_report`: Linting violations report
- `complexity_report`: Code complexity metrics
- `quality_score`: Overall quality score (0-100)

### Metrics
- `format_violations`: Counter of formatting violations
- `lint_errors`: Counter of linting errors
- `lint_warnings`: Counter of linting warnings
- `complexity_score`: Gauge of average complexity
- `execution_time`: Histogram of execution duration

### Dependencies
- None (can run independently)

### Triggers
- `pre-commit`: On git pre-commit hook
- `pull_request`: On pull request creation/update
- `manual`: On explicit invocation

### Implementation
```rust
pub struct CodeQualityAgent {
    config: CodeQualityConfig,
    rust_formatter: RustFormatter,
    rust_linter: RustLinter,
    ts_linter: TypeScriptLinter,
    analyzer: StaticAnalyzer,
}

impl Agent for CodeQualityAgent {
    async fn execute(&self, context: AgentContext) -> AgentResult {
        // 1. Format Rust code
        let format_result = self.rust_formatter.format(&context.workspace)?;

        // 2. Lint Rust code
        let rust_lint_result = self.rust_linter.lint(&context.workspace)?;

        // 3. Lint TypeScript code
        let ts_lint_result = self.ts_linter.lint(&context.workspace)?;

        // 4. Static analysis
        let analysis_result = self.analyzer.analyze(&context.workspace)?;

        // 5. Generate report
        let report = QualityReport::new()
            .with_formatting(format_result)
            .with_rust_linting(rust_lint_result)
            .with_ts_linting(ts_lint_result)
            .with_analysis(analysis_result);

        Ok(AgentResult::success(report))
    }
}
```

---

## 2. Testing Agent

**ID**: `testing`
**Responsibility**: Execute and validate all test suites, ensuring code correctness and coverage.

### Scope
- Unit tests (Rust `cargo test`, TypeScript `vitest`)
- Integration tests
- End-to-end tests (Playwright)
- Test coverage analysis
- Test performance metrics

### Boundaries
- Does NOT write tests (developer responsibility)
- Does NOT fix failing tests (only reports)
- Does NOT manage test infrastructure (Deployment Agent)

### Capabilities
- Run Rust unit tests
- Run TypeScript/JavaScript unit tests
- Run integration tests
- Run E2E tests
- Generate coverage reports
- Analyze test performance
- Detect flaky tests

### Inputs
- `workspace`: Path to workspace root (required)
- `test_type`: Type of tests to run (unit|integration|e2e|all)
- `coverage_threshold`: Minimum coverage percentage (optional)
- `parallel`: Run tests in parallel (optional, default: true)

### Outputs
- `test_results`: Test execution results
- `coverage_report`: Code coverage report
- `performance_report`: Test performance metrics
- `flaky_tests`: List of flaky tests detected

### Metrics
- `tests_total`: Counter of total tests
- `tests_passed`: Counter of passed tests
- `tests_failed`: Counter of failed tests
- `coverage_percentage`: Gauge of code coverage
- `test_duration`: Histogram of test execution time

### Dependencies
- `build`: Requires successful build before testing

### Triggers
- `pre-commit`: On git pre-commit hook
- `pull_request`: On pull request creation/update
- `push`: On push to main/develop branches
- `manual`: On explicit invocation

---

## 3. Build Agent

**ID**: `build`
**Responsibility**: Compile, bundle, and generate all build artifacts for the project.

### Scope
- Rust compilation (`cargo build`)
- TypeScript compilation (`npm run build`)
- Docker image builds
- Artifact generation
- Build optimization

### Boundaries
- Does NOT deploy artifacts (Deployment Agent)
- Does NOT run tests (Testing Agent)
- Does NOT manage dependencies (Dependency Agent)

### Capabilities
- Build Rust workspace
- Build TypeScript/Next.js frontend
- Build Docker images
- Generate release artifacts
- Optimize build performance
- Cache build artifacts

### Inputs
- `workspace`: Path to workspace root (required)
- `target`: Build target (debug|release|all)
- `features`: Feature flags to enable (optional)
- `optimize`: Enable optimizations (optional, default: true)

### Outputs
- `artifacts`: List of generated artifacts
- `build_log`: Build execution log
- `build_metrics`: Build performance metrics

### Metrics
- `build_success`: Counter of successful builds
- `build_failure`: Counter of failed builds
- `build_duration`: Histogram of build time
- `artifact_size`: Histogram of artifact sizes

### Dependencies
- None (can run independently)

### Triggers
- `pre-commit`: On git pre-commit hook
- `pull_request`: On pull request creation/update
- `push`: On push to branches
- `release`: On release creation
- `manual`: On explicit invocation

---

## 4. Schema Agent

**ID**: `schema`
**Responsibility**: Validate OpenAPI schemas, generate contracts, and ensure API contract compliance.

### Scope
- OpenAPI schema validation
- Contract generation (Rust, TypeScript, Python)
- Breaking change detection
- Schema linting
- API documentation generation

### Boundaries
- Does NOT implement API handlers (developer responsibility)
- Does NOT test API endpoints (Testing Agent)
- Does NOT deploy APIs (Deployment Agent)

### Capabilities
- Validate OpenAPI 3.1 schemas
- Generate Rust API types
- Generate TypeScript client SDKs
- Generate Python client SDKs
- Detect breaking changes
- Generate API documentation
- Lint schemas for best practices

### Inputs
- `schema_path`: Path to OpenAPI schema (required)
- `generate_clients`: Generate client SDKs (optional, default: true)
- `check_breaking`: Check for breaking changes (optional, default: true)
- `generate_docs`: Generate documentation (optional, default: true)

### Outputs
- `validation_report`: Schema validation results
- `generated_contracts`: Generated contract files
- `breaking_changes`: List of breaking changes detected
- `api_docs`: Generated API documentation

### Metrics
- `schema_validations`: Counter of schema validations
- `contracts_generated`: Counter of contracts generated
- `breaking_changes_detected`: Counter of breaking changes
- `generation_duration`: Histogram of generation time

### Dependencies
- None (can run independently)

### Triggers
- `schema_changed`: On OpenAPI schema file changes
- `pre-commit`: On git pre-commit hook
- `pull_request`: On pull request creation/update
- `manual`: On explicit invocation

---

## 5. Security Agent

**ID**: `security`
**Responsibility**: Scan for security vulnerabilities, audit dependencies, and enforce security policies.

### Scope
- Dependency vulnerability scanning
- Cargo audit (`cargo audit`)
- Cargo deny (`cargo-deny`)
- Secret scanning
- License compliance
- Security policy enforcement

### Boundaries
- Does NOT fix vulnerabilities (only reports)
- Does NOT manage dependencies (Dependency Agent)
- Does NOT deploy security patches (Deployment Agent)

### Capabilities
- Scan Rust dependencies for vulnerabilities
- Scan Node.js dependencies for vulnerabilities
- Audit licenses for compliance
- Detect secrets in code
- Enforce security policies
- Generate security reports

### Inputs
- `workspace`: Path to workspace root (required)
- `scan_type`: Type of scan (dependencies|secrets|licenses|all)
- `severity_threshold`: Minimum severity to report (optional)
- `fail_on_critical`: Fail on critical vulnerabilities (optional, default: true)

### Outputs
- `vulnerability_report`: List of vulnerabilities found
- `license_report`: License compliance report
- `secret_report`: Secrets detected in code
- `security_score`: Overall security score (0-100)

### Metrics
- `vulnerabilities_critical`: Counter of critical vulnerabilities
- `vulnerabilities_high`: Counter of high severity vulnerabilities
- `vulnerabilities_medium`: Counter of medium severity vulnerabilities
- `vulnerabilities_low`: Counter of low severity vulnerabilities
- `scan_duration`: Histogram of scan time

### Dependencies
- None (can run independently)

### Triggers
- `pre-commit`: On git pre-commit hook
- `pull_request`: On pull request creation/update
- `push`: On push to main/develop branches
- `scheduled`: Daily security scans
- `manual`: On explicit invocation

---

## 6. Documentation Agent

**ID**: `documentation`
**Responsibility**: Generate, validate, and maintain project documentation.

### Scope
- API documentation generation
- Code documentation (rustdoc, JSDoc)
- README validation
- Documentation link checking
- Documentation coverage analysis

### Boundaries
- Does NOT write documentation content (developer responsibility)
- Does NOT deploy documentation (Deployment Agent)
- Does NOT manage documentation infrastructure (Deployment Agent)

### Capabilities
- Generate Rust API docs (`cargo doc`)
- Generate TypeScript API docs
- Validate README files
- Check documentation links
- Analyze documentation coverage
- Generate documentation reports

### Inputs
- `workspace`: Path to workspace root (required)
- `doc_type`: Type of documentation (api|readme|all)
- `check_links`: Check for broken links (optional, default: true)
- `coverage_threshold`: Minimum documentation coverage (optional)

### Outputs
- `api_docs`: Generated API documentation
- `readme_report`: README validation report
- `link_report`: Broken link report
- `coverage_report`: Documentation coverage analysis

### Metrics
- `docs_generated`: Counter of documentation pages generated
- `broken_links`: Counter of broken links found
- `coverage_percentage`: Gauge of documentation coverage
- `generation_duration`: Histogram of generation time

### Dependencies
- `build`: Requires successful build for API docs
- `schema`: Requires schema validation for API docs

### Triggers
- `code_changed`: On code changes affecting documentation
- `pull_request`: On pull request creation/update
- `release`: On release creation
- `manual`: On explicit invocation

---

## 7. Deployment Agent

**ID**: `deployment`
**Responsibility**: Build, package, and deploy application artifacts to target environments.

### Scope
- Docker image building
- Kubernetes manifest validation
- Environment-specific deployments
- Deployment health checks
- Rollback management

### Boundaries
- Does NOT write deployment configs (developer responsibility)
- Does NOT manage infrastructure (infrastructure as code)
- Does NOT run tests (Testing Agent)

### Capabilities
- Build Docker images
- Validate Kubernetes manifests
- Deploy to staging environment
- Deploy to production environment
- Perform health checks
- Rollback deployments
- Generate deployment reports

### Inputs
- `environment`: Target environment (staging|production)
- `image_tag`: Docker image tag (required)
- `manifest_path`: Path to K8s manifests (required)
- `health_check`: Perform health checks (optional, default: true)

### Outputs
- `deployment_status`: Deployment success/failure status
- `deployment_url`: URL of deployed application
- `health_report`: Health check results
- `rollback_info`: Rollback information if needed

### Metrics
- `deployments_successful`: Counter of successful deployments
- `deployments_failed`: Counter of failed deployments
- `deployment_duration`: Histogram of deployment time
- `rollbacks`: Counter of rollbacks performed

### Dependencies
- `build`: Requires successful build
- `security`: Requires security scan pass
- `testing`: Requires test suite pass

### Triggers
- `push`: On push to staging/production branches
- `release`: On release creation
- `manual`: On explicit invocation

---

## 8. Performance Agent

**ID**: `performance`
**Responsibility**: Measure, analyze, and optimize application performance.

### Scope
- Benchmark execution
- Performance profiling
- Memory usage analysis
- Response time measurement
- Throughput analysis

### Boundaries
- Does NOT optimize code automatically (only reports)
- Does NOT deploy optimizations (Deployment Agent)
- Does NOT run functional tests (Testing Agent)

### Capabilities
- Run Rust benchmarks (`cargo bench`)
- Profile application performance
- Measure memory usage
- Analyze response times
- Calculate throughput metrics
- Generate performance reports

### Inputs
- `workspace`: Path to workspace root (required)
- `benchmark_type`: Type of benchmarks (unit|integration|e2e)
- `iterations`: Number of benchmark iterations (optional)
- `profile`: Enable profiling (optional, default: false)

### Outputs
- `benchmark_results`: Benchmark execution results
- `profile_report`: Performance profiling report
- `memory_report`: Memory usage analysis
- `performance_metrics`: Performance metrics summary

### Metrics
- `benchmark_duration`: Histogram of benchmark execution time
- `memory_usage`: Gauge of memory usage
- `response_time_p50`: Gauge of 50th percentile response time
- `response_time_p95`: Gauge of 95th percentile response time
- `throughput`: Gauge of requests per second

### Dependencies
- `build`: Requires successful build

### Triggers
- `code_changed`: On performance-critical code changes
- `pull_request`: On pull request creation/update
- `release`: On release creation
- `scheduled`: Weekly performance benchmarks
- `manual`: On explicit invocation

---

## 9. Architecture Agent

**ID**: `architecture`
**Responsibility**: Validate clean architecture compliance and analyze code structure.

### Scope
- Layer boundary validation
- Dependency direction analysis
- Circular dependency detection
- Coupling metrics
- Architecture rule enforcement

### Boundaries
- Does NOT refactor code (developer responsibility)
- Does NOT enforce coding standards (Code Quality Agent)
- Does NOT run tests (Testing Agent)

### Capabilities
- Validate clean architecture layers
- Analyze dependency directions
- Detect circular dependencies
- Calculate coupling metrics
- Enforce architecture rules
- Generate architecture reports

### Inputs
- `workspace`: Path to workspace root (required)
- `rules_file`: Path to architecture rules file (optional)
- `strict`: Fail on violations (optional, default: false)

### Outputs
- `violations_report`: Architecture violations found
- `dependency_graph`: Dependency graph visualization
- `coupling_metrics`: Code coupling metrics
- `architecture_score`: Overall architecture score (0-100)

### Metrics
- `layer_violations`: Counter of layer boundary violations
- `circular_dependencies`: Counter of circular dependencies
- `coupling_score`: Gauge of average coupling
- `analysis_duration`: Histogram of analysis time

### Dependencies
- `build`: Requires successful build for analysis

### Triggers
- `code_changed`: On code changes
- `pull_request`: On pull request creation/update
- `manual`: On explicit invocation

---

## 10. Dependency Agent

**ID**: `dependency`
**Responsibility**: Manage dependency versions, updates, and compatibility.

### Scope
- Dependency version checking
- Update recommendations
- Compatibility analysis
- Lock file validation
- Dependency audit

### Boundaries
- Does NOT update dependencies automatically (only reports)
- Does NOT fix dependency issues (developer responsibility)
- Does NOT scan for vulnerabilities (Security Agent)

### Capabilities
- Check for outdated dependencies
- Recommend dependency updates
- Analyze dependency compatibility
- Validate lock files
- Audit dependency licenses
- Generate dependency reports

### Inputs
- `workspace`: Path to workspace root (required)
- `check_type`: Type of check (outdated|compatibility|all)
- `update_strategy`: Update strategy (patch|minor|major|all)

### Outputs
- `outdated_dependencies`: List of outdated dependencies
- `update_recommendations`: Recommended updates
- `compatibility_report`: Compatibility analysis
- `dependency_report`: Comprehensive dependency report

### Metrics
- `outdated_deps`: Counter of outdated dependencies
- `update_opportunities`: Counter of update opportunities
- `compatibility_issues`: Counter of compatibility issues
- `check_duration`: Histogram of check time

### Dependencies
- None (can run independently)

### Triggers
- `lock_changed`: On lock file changes
- `pull_request`: On pull request creation/update
- `scheduled`: Weekly dependency checks
- `manual`: On explicit invocation

---

## Agent Interaction Matrix

| Agent | Code Quality | Testing | Build | Schema | Security | Docs | Deploy | Perf | Arch | Deps |
|-------|--------------|---------|-------|--------|----------|------|--------|------|------|------|
| **Code Quality** | - | ✓ | ✓ | - | - | - | - | - | - | - |
| **Testing** | ✓ | - | ✓ | - | - | - | - | - | - | - |
| **Build** | ✓ | - | - | - | - | ✓ | ✓ | ✓ | ✓ | - |
| **Schema** | - | - | - | - | - | ✓ | - | - | - | - |
| **Security** | - | - | - | - | - | - | ✓ | - | - | ✓ |
| **Documentation** | - | - | ✓ | ✓ | - | - | - | - | - | - |
| **Deployment** | - | ✓ | ✓ | - | ✓ | - | - | - | - | - |
| **Performance** | - | - | ✓ | - | - | - | - | - | - | - |
| **Architecture** | - | - | ✓ | - | - | - | - | - | - | - |
| **Dependency** | - | - | - | - | ✓ | - | - | - | - | - |

**Legend**: ✓ = Can depend on / receive input from

---

## Next Steps

See [IMPLEMENTATION.md](./IMPLEMENTATION.md) for implementation details and [ORCHESTRATION.md](./ORCHESTRATION.md) for workflow patterns.
