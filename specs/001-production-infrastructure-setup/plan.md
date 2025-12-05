# Implementation Plan: Production Infrastructure Setup

**Branch**: `001-production-infrastructure-setup` | **Date**: 2 December 2025 | **Spec**: [spec.md](./spec.md)
**Input**: Complete production-ready infrastructure with Docker Compose (local dev), Monorepo tooling (Turborepo/Nx), Kubernetes manifests (cloud deployment), and CI/CD automation

**Note**: This plan follows the SpecKit workflow for infrastructure implementation.

## Summary

Establish production-grade infrastructure enabling:
1. **Single-command local development** with Docker Compose for rapid iteration
2. **Fast incremental builds** via monorepo tooling (Turborepo) with intelligent caching
3. **Cloud-native deployment** using Kubernetes manifests with autoscaling and resilience
4. **Automated CI/CD pipeline** with GitHub Actions for reliable releases

**Current State**: Backend has 87 compilation errors blocking Docker builds. Frontend MVP complete and merged. Basic Docker/K8s files exist but need enhancement.

**Technical Approach**: Incremental implementation prioritizing P1 (Docker fix) → P2 (Monorepo + K8s) → P3 (CI/CD), ensuring each phase delivers independent value.

## Technical Context

**Language/Version**: Rust 1.90+ (backend), Node.js 20+ / TypeScript 5+ (frontend)
**Primary Dependencies**:
- **Backend**: Tokio (async), Axum (web), cargo workspace with 9 crates
- **Frontend**: Next.js 16, React 19, TanStack Query, Zustand
- **Infrastructure**: Docker 24.0+, Docker Compose v2, Turborepo, Kubernetes 1.28+

**Storage**: SQLite (embedded mode) with vector extensions, file system for documents
**Testing**: cargo test (Rust), jest/vitest (Node.js), contract tests for API endpoints
**Target Platform**:
- **Local**: macOS/Linux with Docker Desktop
- **Cloud**: Kubernetes clusters (EKS/GKE/AKS compatible)

**Project Type**: Monorepo with web application (backend + frontend)
**Performance Goals**:
- Docker: <2 min cold start (all services)
- Builds: <2 min with warm cache (50% reduction)
- K8s: <60s deployment, <30s pod ready time
- CI/CD: <15 min full pipeline (test → build → deploy)

**Constraints**:
- Backend must compile successfully before Docker work (87 errors currently)
- Zero-downtime deployments required for production
- Must work with existing Clean Architecture patterns
- No breaking changes to API contracts during migration

**Scale/Scope**:
- 2 services (backend + frontend)
- 9 Rust crates in workspace
- 4 deployment targets (local, dev, staging, production)
- Support 2-5 concurrent developers initially

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| **Clean Architecture** | ✅ PASS | Infrastructure work does not modify domain/application layers |
| **SOLID Principles** | ✅ PASS | Configuration and deployment code follows SRP and DIP |
| **Shared Domain Crates** | ✅ PASS | Reuses existing zero-latency-* crates, no new shared crates needed |
| **ServiceContainer DI** | ✅ PASS | No changes to DI patterns, only deployment configuration |
| **Feature Flags** | ✅ PASS | Maintains existing `embedded` and `cloud` feature architecture |
| **Test Coverage** | ✅ PASS | Infrastructure changes tested via smoke tests and health checks |

### Quality Standards

| Standard | Status | Implementation |
|----------|--------|----------------|
| **Performance Requirements** | ✅ PASS | No impact on startup time, search latency, or memory usage |
| **Code Quality Metrics** | ✅ PASS | Configuration files <200 lines, YAML linting enforced |
| **Maintainability Targets** | ✅ PASS | Infrastructure changes improve maintainability (faster setup, reliable deployment) |

### Development Workflow

| Workflow | Status | Implementation |
|----------|--------|----------------|
| **Branch Strategy** | ✅ PASS | Feature branch `001-production-infrastructure-setup` follows convention |
| **Code Review** | ✅ PASS | Infrastructure changes reviewed for security and best practices |
| **Testing Requirements** | ⚠️ MODIFIED | Infrastructure tested via smoke tests, not unit tests (acceptable for DevOps work) |

### Violations Requiring Justification

**NONE** - All constitution principles either unaffected or properly maintained by infrastructure changes.

### Post-Phase-1 Re-check

After design phase, verify:
- [ ] No new shared crates introduced without justification
- [ ] Kubernetes manifests follow principle of least privilege
- [ ] CI/CD pipeline respects test coverage requirements
- [ ] Monorepo structure preserves Clean Architecture boundaries

## Project Structure

### Documentation (this feature)

```text
specs/001-production-infrastructure-setup/
├── spec.md              # Feature specification (complete)
├── plan.md              # This file (in progress)
├── research.md          # Phase 0 output (pending)
├── data-model.md        # Phase 1 output (pending)
├── quickstart.md        # Phase 1 output (pending)
├── contracts/           # Phase 1 output (pending)
│   ├── docker-compose.contract.md
│   ├── turborepo.contract.md
│   ├── kubernetes.contract.md
│   └── github-actions.contract.md
└── checklists/
    └── requirements.md  # Quality validation (complete)
```

### Source Code (repository root)

```text
# Current structure (Cargo workspace with frontend)
/
├── Cargo.toml                  # Workspace manifest
├── Cargo.lock                  # Dependency lock file
├── Dockerfile                  # Backend Docker image (needs fixes)
├── docker-compose.yml          # Multi-service orchestration (needs enhancement)
├── Makefile                    # Build and dev commands
│
├── crates/                     # Rust workspace crates (8 crates)
│   ├── cli/                    # CLI binary crate
│   ├── zero-latency-api/       # API client
│   ├── zero-latency-config/    # Configuration
│   ├── zero-latency-contracts/ # Shared contracts
│   ├── zero-latency-core/      # Core domain
│   ├── zero-latency-observability/  # Metrics/logging
│   ├── zero-latency-search/    # Search domain
│   └── zero-latency-vector/    # Vector storage
│
├── services/                   # Service binaries
│   └── doc-indexer/           # Main backend service (HAS COMPILATION ERRORS)
│
├── frontend/                   # Next.js application
│   ├── package.json           # Frontend dependencies
│   ├── Dockerfile             # Frontend Docker image (working)
│   ├── src/
│   │   ├── app/              # Next.js 16 app directory
│   │   ├── components/       # React components
│   │   ├── lib/              # Utilities
│   │   └── types/            # TypeScript types
│   └── public/               # Static assets
│
├── k8s/                       # Kubernetes manifests (EXISTS, needs enhancement)
│   ├── README.md
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── ingress.yaml
│   └── hpa.yaml
│
├── .github/                   # GitHub configuration
│   ├── workflows/            # GitHub Actions (TO BE CREATED)
│   └── copilot-instructions.md
│
├── docker/                    # Docker configuration
│   ├── prometheus/           # Prometheus config
│   └── grafana/              # Grafana config
│
└── scripts/                   # Build and utility scripts
    ├── build-dmg.sh
    ├── install.sh
    └── README.md
```

### Target Structure (after monorepo migration - Phase 2)

```text
# Monorepo structure with Turborepo
/
├── package.json              # Root package.json with workspaces
├── turbo.json               # Turborepo pipeline configuration (TO BE CREATED)
├── pnpm-workspace.yaml      # PNPM workspace config (TO BE CREATED)
│
├── apps/                    # Applications
│   ├── backend/            # Backend service (moved from services/doc-indexer)
│   │   ├── Cargo.toml
│   │   ├── package.json    # For Turborepo integration
│   │   └── src/
│   └── frontend/           # Frontend app (current frontend/ moved here)
│       ├── package.json
│       └── src/
│
├── packages/               # Shared packages
│   ├── rust-crates/       # Existing crates/ directory moved here
│   │   └── [all zero-latency-* crates]
│   ├── config/            # Shared configuration
│   │   └── tsconfig/      # Shared TypeScript configs
│   └── eslint-config/     # Shared ESLint configs
│
├── infra/                 # Infrastructure as code
│   ├── docker/           # Docker configs
│   ├── k8s/              # Kubernetes manifests (enhanced)
│   │   ├── base/         # Base manifests
│   │   └── overlays/     # Kustomize overlays (dev/staging/prod)
│   └── terraform/        # (Future: IaC for cloud resources)
│
└── .github/
    └── workflows/         # CI/CD pipelines
        ├── pr-validation.yml
        ├── staging-deploy.yml
        └── production-deploy.yml
```

**Structure Decision**:

**Phase 1 (Docker Compose Fix)**: Work within current structure, enhance existing Dockerfile and docker-compose.yml

**Phase 2 (Monorepo Migration)**: Incremental migration to monorepo structure:
1. Create root package.json with workspace configuration
2. Add turbo.json for pipeline orchestration
3. Gradually move apps/ and packages/ while maintaining backward compatibility
4. Use symbolic links during transition to avoid breaking existing tooling

**Phase 3 (Kubernetes Enhancement)**: Organize k8s/ with Kustomize base + overlays pattern

**Phase 4 (CI/CD)**: Create .github/workflows/ with staged deployment pipeline

This approach minimizes disruption while enabling each phase to deliver independent value.

## Complexity Tracking

> **Infrastructure work does not violate constitution principles**

This feature enhances deployment and development infrastructure without modifying core architecture or introducing new patterns that conflict with the constitution.

### Why No Violations

| Area | Rationale |
|------|-----------|
| **No new shared crates** | Reuses existing zero-latency-* crates, only adds configuration |
| **Preserves Clean Architecture** | Infrastructure changes are in deployment layer, not domain/application |
| **Maintains DI patterns** | ServiceContainer patterns unchanged, only deployment configuration added |
| **Testing approach modified** | Infrastructure tested via smoke tests and health checks (industry standard for DevOps work) rather than unit tests |

### Complexity Justifications (If Any)

**NONE REQUIRED** - All work aligns with existing architecture and constitution principles.

### Pre-existing Issues

**Backend Compilation Errors**: 87 errors in `services/doc-indexer` block Docker builds. This is a pre-existing issue unrelated to infrastructure work and must be resolved as Phase 0 prerequisite before Docker Compose enhancements.

---

## Phase 0: Research & Investigation

**Goal**: Resolve all NEEDS CLARIFICATION items and establish technical foundation for implementation.

### Research Tasks

#### R1: Backend Compilation Fix Strategy
**Question**: What's causing the 87 compilation errors in `services/doc-indexer`?
**Research Approach**:
1. Run `cargo check --bin doc-indexer` to get full error list
2. Categorize errors: type mismatches, missing traits, undefined methods
3. Identify root causes: API changes, dependency updates, incomplete refactoring
4. Determine fix strategy: targeted fixes vs broader refactoring

**Decision Criteria**: Can errors be fixed in <4 hours? If yes, fix inline. If no, create separate bug fix task.

**Output**: `research.md` section documenting error categories and fix strategy

---

#### R2: Docker Multi-Stage Build Best Practices
**Question**: How to optimize Docker builds for Rust + Node.js monorepo?
**Research Approach**:
1. Review Docker BuildKit features for parallel builds
2. Investigate cargo-chef for Rust dependency caching
3. Research npm/pnpm workspace compatibility with Docker
4. Benchmark different caching strategies

**Decision Criteria**: Which approach gives best build time with smallest final image size?

**Output**: `research.md` section with recommended Docker build strategy

---

#### R3: Turborepo vs Nx Comparison
**Question**: Which monorepo tool best fits Rust + Node.js polyglot workspace?
**Research Approach**:
1. Compare Turborepo and Nx feature matrices
2. Test Turborepo with Rust via custom task runners
3. Evaluate Nx Rust plugin if available
4. Benchmark caching effectiveness for both

**Decision Criteria**: Tool must support both Rust and Node.js builds with effective caching. Prefer simpler setup.

**Output**: `research.md` decision with rationale and configuration approach

**Preliminary Recommendation**: Turborepo (simpler, proven with Next.js, custom task runners for Rust)

---

#### R4: Kustomize Overlay Pattern
**Question**: How to structure Kustomize overlays for dev/staging/prod?
**Research Approach**:
1. Review Kustomize best practices documentation
2. Examine common overlay patterns (base + environments)
3. Design secrets management strategy (sealed-secrets vs external)
4. Plan ConfigMap vs Secret usage

**Decision Criteria**: Clear separation of concerns, minimal duplication, easy to understand

**Output**: `research.md` section with overlay structure and examples

---

#### R5: GitHub Actions Matrix Strategy
**Question**: How to optimize CI/CD pipeline for monorepo with matrix builds?
**Research Approach**:
1. Review GitHub Actions monorepo examples
2. Design job dependency graph (test → build → deploy)
3. Plan caching strategy (Rust target/, Node node_modules/)
4. Determine parallelization opportunities

**Decision Criteria**: Pipeline completes in <15 minutes, uses caching effectively, clear failure modes

**Output**: `research.md` section with pipeline architecture and job definitions

---

#### R6: Zero-Downtime Deployment Strategy
**Question**: How to achieve zero-downtime deployments on Kubernetes?
**Research Approach**:
1. Review K8s rolling update strategies
2. Design readiness/liveness probe configuration
3. Plan database migration handling (if needed)
4. Test rollback procedures

**Decision Criteria**: No dropped requests during deployment, automatic rollback on failure

**Output**: `research.md` section with deployment strategy and rollback plan

---

### Research Output Structure

File: `specs/001-production-infrastructure-setup/research.md`

```markdown
# Research: Production Infrastructure Setup

## R1: Backend Compilation Fix
**Decision**: [Fix inline | Create separate task]
**Rationale**: [Error analysis and fix complexity]
**Implementation**: [Step-by-step fix approach]

## R2: Docker Build Strategy
**Decision**: [cargo-chef | standard multi-stage | custom approach]
**Rationale**: [Build time vs image size tradeoffs]
**Implementation**: [Dockerfile structure with caching layers]

## R3: Monorepo Tool Selection
**Decision**: [Turborepo | Nx]
**Rationale**: [Feature comparison and fit analysis]
**Implementation**: [turbo.json configuration approach]

## R4: Kustomize Overlay Pattern
**Decision**: [base + overlays structure]
**Rationale**: [Separation of concerns, DRY principles]
**Implementation**: [Directory structure and example overlays]

## R5: GitHub Actions Pipeline
**Decision**: [Pipeline architecture with job matrix]
**Rationale**: [Speed vs cost vs clarity tradeoffs]
**Implementation**: [Workflow file structure and caching strategy]

## R6: Zero-Downtime Deployments
**Decision**: [Rolling update strategy with health checks]
**Rationale**: [Availability requirements and rollback needs]
**Implementation**: [K8s deployment configuration and testing plan]
```

---

## Phase 1: Design & Contracts

**Prerequisites**: Phase 0 research complete, backend compilation errors fixed

**Goal**: Create detailed technical designs and API contracts for each infrastructure component.

### D1: Data Model (`data-model.md`)

Infrastructure entities and their relationships:

```markdown
# Data Model: Production Infrastructure

## Entities

### DockerService
- **Properties**: name, image, ports, volumes, healthCheck, environment, dependencies
- **Relationships**: depends_on other DockerServices
- **State**: stopped | starting | healthy | unhealthy
- **Validation**: port conflicts, volume paths exist, health check endpoints valid

### MonorepoPackage
- **Properties**: name, path, dependencies, outputs, tasks (build/test/lint/dev)
- **Relationships**: depends on other MonorepoPackages
- **State**: clean | building | built | cached
- **Validation**: circular dependency detection, output paths valid

### KubernetesResource
- **Properties**: kind, apiVersion, metadata, spec, namespace
- **Relationships**: Services reference Deployments, Ingress references Services
- **State**: pending | available | degraded
- **Validation**: YAML schema compliance, resource limits set, labels consistent

### CIPipeline
- **Properties**: trigger (push/pr/manual), jobs, matrix, caching, secrets
- **Relationships**: jobs depend on other jobs, workflows trigger other workflows
- **State**: queued | running | success | failed
- **Validation**: secret references valid, job dependencies acyclic, timeouts set

### Deployment
- **Properties**: version, imageTag, configHash, timestamp, status, healthStatus
- **Relationships**: references KubernetesResources, tracks previous Deployment for rollback
- **State**: deploying | healthy | degraded | rolled_back
- **Validation**: image exists in registry, config valid, health checks defined
```

### D2: API Contracts (`contracts/`)

#### `docker-compose.contract.md`
```yaml
# Docker Compose Contract
version: '3.8'
services:
  <service>:
    build: <build-config>
    ports: ["<host>:<container>"]
    environment: [<KEY>: <value>]
    volumes: [<source>:<target>]
    healthcheck:
      test: ["CMD", "<command>"]
      interval: <duration>
    depends_on:
      <service>: {condition: service_healthy}
```

#### `turborepo.contract.md`
```json
{
  "pipeline": {
    "<task>": {
      "dependsOn": ["^<task>"],
      "outputs": ["<glob>"],
      "cache": true|false
    }
  }
}
```

#### `kubernetes.contract.md`
```yaml
# Base Kubernetes Manifest Contract
---
apiVersion: <group>/<version>
kind: <Kind>
metadata:
  name: <name>
  namespace: <namespace>
  labels: {app: <app>, component: <component>}
spec:
  # Kind-specific spec following K8s API conventions
```

#### `github-actions.contract.md`
```yaml
# GitHub Actions Workflow Contract
name: <workflow-name>
on: {<trigger>: {<filter>}}
jobs:
  <job>:
    runs-on: <runner>
    steps:
      - uses: <action>@<version>
        with: {<param>: <value>}
```

### D3: Quickstart Guide (`quickstart.md`)

```markdown
# Quickstart: Production Infrastructure

## Prerequisites
- Docker Desktop 24.0+ with 8GB RAM allocated
- Node.js 20+ and npm 10+
- Rust 1.90+ toolchain
- kubectl 1.28+ (for K8s deployment)
- make (GNU Make or compatible)

## Local Development (2 minutes)

1. **Start all services**:
   ```bash
   make docker-up
   ```

2. **Verify health**:
   ```bash
   curl http://localhost:8081/health  # Backend
   curl http://localhost:3000          # Frontend
   ```

3. **View logs**:
   ```bash
   make docker-logs
   ```

4. **Stop services**:
   ```bash
   make docker-down
   ```

## Monorepo Builds (Phase 2)

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Build all packages**:
   ```bash
   turbo build
   ```

3. **Run tests**:
   ```bash
   turbo test
   ```

4. **Development mode**:
   ```bash
   turbo dev
   ```

## Kubernetes Deployment (Phase 3)

1. **Local cluster (minikube)**:
   ```bash
   minikube start
   kubectl apply -k infra/k8s/overlays/dev
   ```

2. **Production cluster**:
   ```bash
   kubectl apply -k infra/k8s/overlays/production
   ```

3. **Check status**:
   ```bash
   kubectl get pods -n docsearch
   kubectl get ing -n docsearch
   ```

## CI/CD (Phase 4)

1. **Pull request**: Automatic test and build validation
2. **Merge to staging**: Auto-deploy to staging environment
3. **Merge to main**: Manual approval → production deployment

## Troubleshooting

See `specs/001-production-infrastructure-setup/TROUBLESHOOTING.md`
```

### Agent Context Update

After Phase 1 design completion, run:
```bash
.specify/scripts/bash/update-agent-context.sh copilot
```

This updates `.github/copilot-instructions.md` with:
- New infrastructure components (Docker, Turborepo, K8s, CI/CD)
- Monorepo structure conventions
- Deployment procedures
- Testing strategies for infrastructure code

---

## Implementation Phases Summary

This plan follows SpecKit workflow phases. After completing Phase 0 and Phase 1 (above), proceed to task breakdown.

### Next Steps

1. **Execute Phase 0 Research** (automated by plan workflow):
   - Run research agents for each R1-R6 task
   - Document decisions in `research.md`
   - Resolve backend compilation errors (critical blocker)

2. **Execute Phase 1 Design** (automated by plan workflow):
   - Create `data-model.md` with entity definitions
   - Create contract files in `contracts/` directory
   - Create `quickstart.md` with setup instructions
   - Run agent context update script

3. **Phase 2: Task Breakdown** (separate command):
   - Run `/speckit.tasks` to generate detailed task list
   - Tasks organized by user story priority (P1 → P2 → P3)
   - Each task includes acceptance criteria and estimated time

4. **Phase 3+: Implementation**:
   - Work through tasks incrementally
   - Test each component independently
   - Validate against success criteria from spec
   - Document learnings and edge cases

### Critical Path

**BLOCKER**: Backend compilation errors MUST be fixed before Docker Compose work can begin.

**Fastest Path to Value**:
1. Fix compilation errors (Phase 0, R1)
2. Enhance Docker Compose (P1, delivers immediate developer value)
3. Set up Turborepo (P2, improves build speed)
4. Enhance Kubernetes (P2, enables cloud deployment)
5. Add CI/CD (P3, automates delivery)

### Success Metrics Tracking

Monitor these KPIs throughout implementation:

| Metric | Baseline | Target | Phase |
|--------|----------|--------|-------|
| Local startup time | N/A (broken) | <2 min | P1 |
| Build time (warm cache) | ~4 min | <2 min | P2 |
| K8s deployment time | N/A | <60s | P2 |
| CI/CD pipeline time | N/A | <15 min | P3 |
| Deployment success rate | N/A | >95% | P3 |

### Risk Mitigation

| Risk | Phase | Mitigation |
|------|-------|------------|
| Compilation errors too complex | 0 | Time-box to 4 hours, create separate task if needed |
| Docker memory issues | 1 | Document 8GB requirement, use multi-stage builds |
| Monorepo migration breaks tooling | 2 | Incremental migration with symbolic links, test each step |
| K8s learning curve | 2 | Start with local cluster (minikube), comprehensive docs |
| CI/CD costs exceed budget | 3 | Aggressive caching, monitor usage, optimize parallelism |

---

## Plan Status

**Status**: ✅ Complete - Ready for Phase 0 Research Execution
**Last Updated**: 2 December 2025
**Next Action**: Begin Phase 0 research (see research tasks R1-R6 above)
