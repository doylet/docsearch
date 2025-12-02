<!--
Sync Impact Report:
- Version: 0.0.0 → 1.0.0 (Initial constitution)
- Modified Principles: None (initial creation)
- Added Sections: All (initial creation)
- Removed Sections: None
- Templates Status:
  ✅ plan-template.md - aligned with Clean Architecture and SOLID principles
  ✅ spec-template.md - aligned with user story priorities and independent testing
  ✅ tasks-template.md - aligned with MVP-first, user story organization
- Follow-up TODOs: None
-->

# Zero-Latency Documentation Search Constitution

## Core Principles

### I. Clean Architecture (NON-NEGOTIABLE)

All services and crates MUST follow Clean Architecture patterns with three-layer separation:

- **Application Layer**: Use cases, orchestration, no infrastructure details
- **Domain Layer**: Entities, interfaces, business rules - framework independent
- **Infrastructure Layer**: Adapters for external services, databases, APIs

**Rationale**: Enables testability, maintainability, and independent evolution of each layer. Infrastructure changes don't affect business logic.

**Enforcement**: Code reviews MUST verify layer boundaries. No domain/application imports from infrastructure. No infrastructure details in domain layer.

### II. SOLID Principles Compliance

Every component MUST demonstrate SOLID principles:

- **Single Responsibility**: One reason to change per module/struct
- **Open/Closed**: Extend through composition and traits, not modification
- **Liskov Substitution**: Implementations must be substitutable via traits
- **Interface Segregation**: Small, focused trait interfaces
- **Dependency Inversion**: Depend on abstractions (traits), not concrete types

**Rationale**: SOLID principles proven effective through Phase 4C/4D implementations. Reduces coupling, increases cohesion, enables testing.

**Enforcement**: Architecture reviews before implementation. >90% compliance required across all services.

### III. Shared Domain Crates

Reuse domain abstractions through shared crates:

- `zero-latency-core`: Foundation models, error handling, health monitoring
- `zero-latency-vector`: Vector storage and embedding abstractions
- `zero-latency-search`: Search orchestration and query processing
- `zero-latency-observability`: Metrics and monitoring frameworks
- `zero-latency-config`: Type-safe configuration management
- `zero-latency-contracts`: Shared API contracts and types
- `zero-latency-api`: API client implementations

**Rationale**: Prevents duplication, ensures consistency, accelerates development by providing battle-tested patterns.

**Enforcement**: MUST search for existing abstractions before creating new ones. Code reviews verify shared crate usage. No duplicate capabilities.

### IV. Dependency Injection via ServiceContainer

All services MUST use ServiceContainer pattern for dependency management:

- Concrete types preferred over trait objects (compile-time verification)
- Builder pattern for container construction
- All dependencies injected, never instantiated within business logic
- Makes testing trivial through dependency replacement

**Rationale**: Enables comprehensive testing, clear dependency graphs, production-ready configuration management.

**Enforcement**: No `new()` calls in application/domain layers. All external dependencies injected via ServiceContainer.

### V. Feature Flag Architecture

Optimize deployments with conditional compilation:

- **`embedded`**: Local SQLite + ONNX models (edge/single-binary deployment)
- **`cloud`**: External Qdrant + OpenAI integration (server deployment)
- **`full`**: All features enabled (development/testing)

**Rationale**: Tailored builds for specific environments. Reduced binary size for edge deployments. Clear runtime errors for missing features.

**Enforcement**: Features MUST be mutually exclusive or composable. Runtime checks for unavailable features with clear error messages.

### VI. Test Coverage Standards

Testing requirements by component type:

- **Unit Tests**: >80% coverage for domain/application logic
- **Integration Tests**: >80% coverage for service interactions
- **Contract Tests**: 100% coverage for API endpoints
- **End-to-End Tests**: Complete user journey validation

**Rationale**: Comprehensive testing prevents regressions, validates architecture, enables confident refactoring.

**Enforcement**: CI gates block merges below coverage thresholds. Contract tests generated from OpenAPI specs.

## Quality Standards

### Performance Requirements

- **Startup Time**: <1 second cold start
- **Search Latency**: <100ms p95 response time
- **Memory Usage**: <100MB baseline (before document loading)
- **Build Time**: <30 seconds incremental builds
- **Test Suite**: <2 minutes full test execution

**Rationale**: Fast iteration cycles improve developer productivity. Sub-100ms search ensures excellent user experience.

**Monitoring**: Prometheus metrics track all performance KPIs. Alerts on threshold violations.

### Code Quality Metrics

- **File Size**: <200 lines per file (excluding tests)
- **Cyclomatic Complexity**: <5 per function
- **Dead Code**: Zero `#[allow(dead_code)]` in production
- **Warnings**: Zero warnings in release builds
- **Documentation**: Public APIs 100% documented

**Rationale**: Maintainable codebase requires small, focused modules. Low complexity enables easy comprehension.

**Enforcement**: Clippy with strict lints. Code reviews verify complexity. CI fails on warnings.

### Maintainability Targets

- **Time to Add Feature**: <2 hours for new search step
- **Time to Change Vector Store**: <30 minutes
- **Time to Add Endpoint**: <1 hour
- **Onboarding Time**: <4 hours for new developer

**Rationale**: Measures architectural effectiveness. Fast feature addition indicates good separation of concerns.

**Tracking**: Measure actual times during Phase 5+. Adjust architecture if targets consistently missed.

## Development Workflow

### Branch Strategy

- **main**: Always releasable, protected branch
- **feature branches**: `###-feature-name` pattern for all work
- **No direct commits**: All changes via pull requests
- **Fast-forward merges**: Maintain linear history when possible

**Rationale**: Main branch stability critical for continuous deployment. Feature branches enable parallel work and code review.

**Enforcement**: Branch protection rules in Git. CI checks before merge. No force pushes to main.

### Code Review Requirements

Every pull request MUST include:

1. **Constitution Check**: Verify compliance with all principles
2. **Architecture Review**: Validate Clean Architecture layers
3. **Test Coverage**: Verify coverage thresholds met
4. **Performance Impact**: Assess against KPI targets
5. **Documentation**: Public APIs documented

**Rationale**: Peer review catches issues early, transfers knowledge, maintains consistent quality.

**Process**: Minimum one approval required. Architecture changes require technical lead approval.

### Testing Requirements

Before implementation:

1. **Write tests first** - demonstrate understanding of requirements
2. **Tests MUST fail** - prove test validity before implementation
3. **Implement minimum** - satisfy failing tests only
4. **Refactor** - improve while maintaining green tests
5. **Integration validate** - ensure system works end-to-end

**Rationale**: TDD prevents over-engineering, validates requirements understanding, enables fearless refactoring.

**Exceptions**: Prototypes may defer tests for 48 hours. Production code MUST have tests before merge.

## Technology Stack

### Core Technologies

- **Language**: Rust 1.75+ (stable channel)
- **Async Runtime**: Tokio for all async operations
- **Web Framework**: Axum for HTTP services
- **CLI Framework**: Clap for command-line interfaces
- **Testing**: Standard test framework + cargo-nextest
- **Observability**: Tracing + Prometheus metrics

**Rationale**: Proven technology choices from Phase 4. Memory safety from Rust. High-performance async via Tokio.

**Constraints**: No alternative async runtimes. No alternative web frameworks without architectural review.

### Storage & ML

- **Vector Storage**: Embedded SQLite (default) or Qdrant (cloud feature)
- **Embeddings**: ONNX Runtime with gte-small model
- **File System**: Standard library fs operations
- **Configuration**: TOML for config files, environment variables for overrides

**Rationale**: Embedded SQLite enables self-contained deployment. ONNX Runtime provides cross-platform ML inference.

**Extensions**: New storage backends via VectorRepository trait. New embedding providers via EmbeddingGenerator trait.

## Operational Requirements

### Observability Standards

All services MUST provide:

- **Health Checks**: `/health` endpoint with component status
- **Metrics**: Prometheus metrics at `/metrics` endpoint
- **Structured Logging**: JSON logs with trace IDs
- **Distributed Tracing**: OpenTelemetry compatible traces

**Rationale**: Production debugging requires comprehensive observability. Standardized endpoints enable monitoring automation.

**Required Metrics**: Request rate, latency percentiles, error rate, resource utilization.

### Configuration Management

- **Type-Safe**: Serde-based configuration structs
- **Validation**: Validate on load, fail fast on invalid config
- **Defaults**: Sensible defaults for all non-required settings
- **Documentation**: Every config option documented with example

**Rationale**: Type safety prevents configuration errors. Early validation catches issues before deployment.

**Pattern**: Config structs in zero-latency-config crate. Environment variable overrides supported.

### Deployment

- **Self-Contained Binaries**: Single binary with embedded dependencies
- **macOS App Bundle**: GUI control panel + LaunchAgent integration
- **Docker Support**: Optional for cloud deployments
- **Feature Variants**: Build appropriate feature set for target environment

**Rationale**: Self-contained binaries simplify distribution. macOS bundle provides professional user experience.

**Distribution**: DMG installer for macOS. Tarballs for Linux. Docker images for cloud.

## Governance

### Constitution Authority

This constitution supersedes all other development practices and guidelines. In case of conflict:

1. Constitution takes precedence
2. Document the conflict and rationale for exception
3. Propose constitution amendment if pattern emerges
4. Get technical lead approval for exceptions

**Amendment Process**:

1. Propose change with detailed rationale
2. Document expected impact on existing code
3. Get team consensus (requires >75% agreement)
4. Update constitution with migration plan
5. Increment version following semantic versioning

**Version Bumping**:

- **MAJOR**: Breaking changes to principles (requires 100% team agreement)
- **MINOR**: New principles or significant section additions
- **PATCH**: Clarifications, typos, non-semantic improvements

### Complexity Justification

Any deviation from constitution principles MUST be justified:

| Violation | Justification Required | Review Level |
|-----------|----------------------|--------------|
| New shared crate | Why existing crates insufficient | Tech lead |
| Skip testing | Why tests not feasible for this component | Team consensus |
| >200 line file | Why splitting would reduce clarity | Code review |
| Infrastructure in domain | Why abstraction not possible | Architecture review |

**Rationale**: Exceptions indicate either valid edge cases or constitution gaps. Documentation prevents principle erosion.

### Continuous Improvement

Constitution evolves with project needs:

- **Quarterly Reviews**: Assess principle effectiveness
- **Metrics Tracking**: Measure maintainability targets
- **Retrospectives**: Identify friction points
- **Amendment Proposals**: Address systematic issues

**Success Indicators**:

- Developer satisfaction with architecture
- Consistent achievement of maintainability targets
- Decreasing time for common tasks
- Low defect rates in production

### Runtime Development Guidance

For agent-specific and detailed implementation guidance, refer to:

- `.github/copilot-instructions.md`: Copilot-specific development policies
- Architecture documentation in `docs/architecture/`
- Phase implementation plans in `docs/milestones/`

**Note**: Constitution provides overarching principles. Runtime guidance provides specific tactical patterns.

---

**Version**: 1.0.0 | **Ratified**: 2025-12-01 | **Last Amended**: 2025-12-01
