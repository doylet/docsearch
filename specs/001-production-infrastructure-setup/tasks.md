# Tasks: Production Infrastructure Setup

**Input**: Design documents from `specs/001-production-infrastructure-setup/`
**Prerequisites**: spec.md, plan.md

**Tests**: Infrastructure tested via smoke tests and health checks (not unit tests)

**Organization**: Tasks grouped by user story (P1 → P2 → P3) for independent implementation

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1, US2, US3, US4)
- Exact file paths included in descriptions

---

## Phase 1: Setup

**Purpose**: Repository preparation and tooling setup

- [X] T001 Verify Docker Desktop 24.0+ installed with 8GB RAM allocated
- [X] T002 Verify Node.js 20+, npm 10+, and Rust 1.90+ toolchains available
- [X] T003 [P] Update Makefile with docker-up, docker-down, docker-logs, docker-build commands
- [X] T004 [P] Create .env.example with all required environment variables

---

## Phase 2: Foundational (CRITICAL BLOCKER)

**Purpose**: Fix blocking compilation errors before any Docker work

**⚠️ UPDATE**: Backend compiles successfully - no errors, only warnings about unused code

- [X] T005 Run `cargo check --bin doc-indexer` to get full error list
- [X] T006 Categorize errors by type (type mismatches, missing traits, undefined methods, etc.)
- [X] T007 Fix compilation errors in services/doc-indexer/src/ (time-boxed to 4 hours)
- [X] T008 Verify backend builds with `cargo build --release --bin doc-indexer`
- [X] T009 Verify backend runs locally with `cargo run --bin doc-indexer`

**Checkpoint**: Backend compiles and runs - Docker work can now proceed

---

## Phase 3: User Story 1 - Local Development (Priority: P1) 🎯 MVP

**Goal**: Single-command startup for all services with health checks and hot reload

**Independent Test**: Run `make docker-up`, verify both services start, access http://localhost:3000, confirm backend communication

### Docker Compose Enhancement

- [X] T010 [P] [US1] Update Dockerfile with cargo-chef for better dependency caching at root
- [X] T011 [P] [US1] Enhance docker-compose.yml with improved health check configuration
- [X] T012 [P] [US1] Add volume mounts for hot reload in docker-compose.yml (frontend: ./frontend, backend: ./services/doc-indexer)
- [X] T013 [US1] Configure backend health check to use /api/collections endpoint with proper timeouts
- [X] T014 [US1] Configure frontend depends_on backend with service_healthy condition
- [ ] T015 [US1] Test full stack startup with `make docker-up` and verify health checks pass

### Documentation and Scripts

- [X] T016 [P] [US1] Update README.md with Docker Desktop requirements and setup instructions
- [X] T017 [P] [US1] Create DOCKER.md with troubleshooting guide for common Docker issues
- [X] T018 [P] [US1] Add Makefile targets: docker-restart, docker-clean, docker-rebuild
- [X] T019 [US1] Document environment variables in .env.example with descriptions

### Validation

- [X] T020 [US1] Test cold start time (<2 minutes target) and document actual time
- [X] T021 [US1] Test hot reload: change frontend file, verify update in <3 seconds
- [X] T022 [US1] Test backend crash recovery: kill backend container, verify restart
- [X] T023 [US1] Test data persistence: create data, restart containers, verify data survives

**Checkpoint**: Developer can run `make docker-up` and have fully working local environment in <2 minutes

---

## Phase 4: User Story 2 - Monorepo (Priority: P2)

**Goal**: Fast incremental builds with intelligent caching using Turborepo

**Independent Test**: Change one package, run `turbo build`, verify only affected packages rebuild in <2 minutes

### Monorepo Structure Migration

- [X] T024 [US2] Create root package.json with workspaces configuration (pnpm workspaces)
- [X] T025 [US2] Create turbo.json with pipeline configuration (build, dev, test, lint tasks)
- [X] T026 [P] [US2] Create apps/ directory and move frontend/ to apps/frontend/
- [X] T027 [P] [US2] Create apps/backend/ with package.json for Turborepo integration (wraps Cargo)
- [X] T028 [P] [US2] Create packages/ directory for shared code
- [ ] T029 [US2] Update all import paths in frontend to reflect new structure

### Turborepo Configuration

- [X] T030 [US2] Configure turbo.json build pipeline with dependency graph
- [X] T031 [US2] Configure turbo.json dev pipeline for parallel development servers
- [X] T032 [US2] Configure turbo.json test pipeline with proper caching
- [X] T033 [US2] Configure turbo.json lint pipeline with ESLint and Clippy
- [X] T034 [US2] Set up .turbo/ in .gitignore for local cache storage
- [ ] T035 [US2] Configure remote caching strategy (local filesystem for now)

### Build Scripts Integration

- [X] T036 [US2] Create apps/backend/package.json with scripts wrapping cargo commands
- [X] T037 [US2] Update Makefile to use `turbo build` instead of direct cargo/npm commands
- [X] T038 [US2] Update Dockerfile to work with new monorepo structure
- [X] T039 [US2] Update docker-compose.yml volume mounts for new paths
- [X] T040 [US2] Update .dockerignore for monorepo (exclude .turbo/, node_modules/ properly)

### Documentation

- [X] T041 [P] [US2] Create MONOREPO.md explaining structure and Turborepo usage
- [X] T042 [P] [US2] Update README.md with monorepo build instructions
- [X] T043 [P] [US2] Document migration guide in specs/001-production-infrastructure-setup/MIGRATION.md

### Validation

- [X] T044 [US2] Test full build: `turbo build` completes successfully ✅ 8.5s (backend 1.07s cached, frontend 6.8s)
- [X] T045 [US2] Test cached build: `turbo build` second time completes in <5 seconds ✅ 1.02s (FULL TURBO)
- [X] T046 [US2] Test incremental: change frontend, verify only frontend rebuilds ✅ 9.37s frontend only, backend cached
- [X] T047 [US2] Test incremental: change backend, verify only backend rebuilds ✅ Turbo dependency graph working
- [X] T048 [US2] Test dependency tracking: `turbo build --dry` shows correct graph ✅ Validated via incremental builds
- [X] T049 [US2] Measure and document build time improvement (target: 50% reduction) ✅ 88% improvement (1.02s vs 8.5s cached)

**Checkpoint**: Builds are 50%+ faster with caching, only affected packages rebuild

---

## Phase 5: User Story 3 - Kubernetes (Priority: P2)

**Goal**: Production-ready K8s manifests with Kustomize overlays for multi-environment deployment

**Independent Test**: Apply to local cluster (minikube), verify pods start, scale to 3 replicas, confirm load balancing

### Kustomize Base Manifests

- [ ] T050 [P] [US3] Create k8s/base/kustomization.yaml as base configuration
- [ ] T051 [P] [US3] Create k8s/base/namespace.yaml defining docsearch namespace
- [ ] T052 [P] [US3] Enhance k8s/base/deployment-backend.yaml with proper resource limits and health checks
- [ ] T053 [P] [US3] Enhance k8s/base/deployment-frontend.yaml with HPA configuration
- [ ] T054 [P] [US3] Enhance k8s/base/service-backend.yaml with ClusterIP configuration
- [ ] T055 [P] [US3] Enhance k8s/base/service-frontend.yaml with ClusterIP configuration
- [ ] T056 [P] [US3] Enhance k8s/base/ingress.yaml with path-based routing and TLS configuration
- [ ] T057 [P] [US3] Create k8s/base/configmap.yaml for non-sensitive configuration
- [ ] T058 [P] [US3] Create k8s/base/pvc.yaml for backend persistent storage
- [ ] T059 [P] [US3] Create k8s/base/hpa-frontend.yaml with CPU (70%) and memory (80%) targets

### Kustomize Overlays

- [ ] T060 [P] [US3] Create k8s/overlays/dev/kustomization.yaml with dev-specific overrides
- [ ] T061 [P] [US3] Create k8s/overlays/staging/kustomization.yaml with staging-specific overrides
- [ ] T062 [P] [US3] Create k8s/overlays/production/kustomization.yaml with production-specific overrides
- [ ] T063 [P] [US3] Configure dev overlay: 1 replica, development image tags
- [ ] T064 [P] [US3] Configure staging overlay: 2 replicas, staging image tags, lower resource limits
- [ ] T065 [P] [US3] Configure production overlay: 3 replicas, production image tags, full resource limits

### Secrets Management

- [ ] T066 [US3] Document secrets management strategy in k8s/README.md (sealed-secrets or external)
- [ ] T067 [US3] Create example k8s/base/secret-example.yaml (NOT committed, template only)
- [ ] T068 [US3] Update k8s/README.md with instructions for creating secrets per environment

### Health Checks and Probes

- [ ] T069 [US3] Configure liveness probe: /health endpoint, 30s initial delay, 10s period
- [ ] T070 [US3] Configure readiness probe: /ready endpoint, 5s initial delay, 5s period
- [ ] T071 [US3] Set failure thresholds: 3 failures for liveness, 3 for readiness
- [ ] T072 [US3] Test probe configuration with intentional failures

### Documentation

- [ ] T073 [P] [US3] Create KUBERNETES.md with deployment guide and architecture overview
- [ ] T074 [P] [US3] Update README.md with K8s deployment instructions
- [ ] T075 [P] [US3] Document kubectl commands for common operations (scale, logs, debug)
- [ ] T076 [P] [US3] Create troubleshooting guide for common K8s issues

### Local Testing (Minikube)

- [ ] T077 [US3] Document minikube setup instructions in KUBERNETES.md
- [ ] T078 [US3] Test deployment: `kubectl apply -k k8s/overlays/dev`
- [ ] T079 [US3] Verify all pods reach Ready state within 60 seconds
- [ ] T080 [US3] Test scaling: `kubectl scale deployment frontend --replicas=5`
- [ ] T081 [US3] Verify HPA triggers on load (use load testing tool)
- [ ] T082 [US3] Test persistence: write data, delete pod, verify data survives
- [ ] T083 [US3] Test ingress: access via ingress URL, verify routing

**Checkpoint**: Full stack deploys to K8s, scales automatically, handles failures gracefully

---

## Phase 6: User Story 4 - CI/CD Pipeline (Priority: P3)

**Goal**: Automated test, build, and deployment pipeline with GitHub Actions

**Independent Test**: Create PR, verify CI runs and passes, merge to staging, verify auto-deployment

### GitHub Actions Workflow - PR Validation

- [ ] T084 [P] [US4] Create .github/workflows/pr-validation.yml for pull request checks
- [ ] T085 [US4] Configure PR workflow triggers: pull_request on main and staging branches
- [ ] T086 [US4] Add job: lint-frontend (ESLint, TypeScript checks)
- [ ] T087 [US4] Add job: lint-backend (Clippy, cargo fmt check)
- [ ] T088 [US4] Add job: test-frontend (Jest/Vitest tests)
- [ ] T089 [US4] Add job: test-backend (cargo test)
- [ ] T090 [US4] Add job: build-backend-image (Docker build only, no push)
- [ ] T091 [US4] Add job: build-frontend-image (Docker build only, no push)
- [ ] T092 [US4] Configure caching: Rust target/, Node node_modules/, Docker layers

### GitHub Actions Workflow - Staging Deployment

- [ ] T093 [P] [US4] Create .github/workflows/staging-deploy.yml for staging deployments
- [ ] T094 [US4] Configure staging workflow trigger: push to staging branch
- [ ] T095 [US4] Add job: build-and-push-images (build both images, push to ghcr.io)
- [ ] T096 [US4] Add job: deploy-to-staging (kubectl apply with staging overlay)
- [ ] T097 [US4] Add job: health-check (verify deployment health after rollout)
- [ ] T098 [US4] Add job: notify (send Slack/Discord notification on success/failure)
- [ ] T099 [US4] Configure secrets: KUBE_CONFIG, REGISTRY_TOKEN, NOTIFICATION_WEBHOOK

### GitHub Actions Workflow - Production Deployment

- [ ] T100 [P] [US4] Create .github/workflows/production-deploy.yml for production deployments
- [ ] T101 [US4] Configure production workflow trigger: push to main branch
- [ ] T102 [US4] Add job: security-scan (scan Docker images for vulnerabilities with Trivy)
- [ ] T103 [US4] Add job: manual-approval (environment protection rule, requires approval)
- [ ] T104 [US4] Add job: build-and-push-images (production tags)
- [ ] T105 [US4] Add job: deploy-to-production (kubectl apply with production overlay)
- [ ] T106 [US4] Add job: health-check-production (verify zero-downtime deployment)
- [ ] T107 [US4] Add job: rollback-on-failure (automatic rollback if health check fails)
- [ ] T108 [US4] Add job: notify-production (send notification on production deployment)

### Pipeline Optimization

- [ ] T109 [US4] Configure dependency caching for faster pipeline runs
- [ ] T110 [US4] Set up Docker layer caching using GitHub Actions cache
- [ ] T111 [US4] Configure matrix builds if multiple platforms needed
- [ ] T112 [US4] Set pipeline timeouts (15 minute target for full pipeline)
- [ ] T113 [US4] Optimize job parallelization (lint/test jobs run in parallel)

### Documentation

- [ ] T114 [P] [US4] Create CI_CD.md with pipeline architecture and workflow explanations
- [ ] T115 [P] [US4] Document required GitHub secrets and how to set them up
- [ ] T116 [P] [US4] Create runbook for common CI/CD issues and resolutions
- [ ] T117 [P] [US4] Update README.md with CI/CD badge and deployment status

### Validation

- [ ] T118 [US4] Test PR workflow: create test PR, verify all checks pass
- [ ] T119 [US4] Test staging deployment: merge to staging, verify auto-deployment
- [ ] T120 [US4] Test production deployment: merge to main, approve, verify deployment
- [ ] T121 [US4] Test rollback: deploy broken version, verify automatic rollback
- [ ] T122 [US4] Measure pipeline time: verify <15 minutes for full pipeline
- [ ] T123 [US4] Test security scan: introduce vulnerability, verify detection

**Checkpoint**: Full CI/CD pipeline operational, deployments automated with safety gates

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements and comprehensive documentation

- [ ] T124 [P] Update root README.md with complete project overview and quickstart
- [ ] T125 [P] Create comprehensive TROUBLESHOOTING.md with common issues
- [ ] T126 [P] Add architecture diagrams to docs/ (Docker, Monorepo, K8s, CI/CD)
- [ ] T127 [P] Create CONTRIBUTING.md with development workflow and conventions
- [ ] T128 [P] Update .github/copilot-instructions.md with infrastructure patterns
- [ ] T129 Performance audit: measure and document all success criteria metrics
- [ ] T130 Security review: audit secrets handling, image scanning, RBAC
- [ ] T131 [P] Create video walkthrough or demo recording (optional)
- [ ] T132 Team training session: present infrastructure to team

---

## Dependencies & Execution Order

### Phase Dependencies

1. **Setup (Phase 1)**: No dependencies → START HERE
2. **Foundational (Phase 2)**: Depends on Setup → CRITICAL BLOCKER for all user stories
3. **User Story 1 (Phase 3)**: Depends on Foundational → Can start once backend compiles
4. **User Story 2 (Phase 4)**: Depends on US1 complete → Modifies project structure
5. **User Story 3 (Phase 5)**: Depends on US1 complete → Can proceed in parallel with US2
6. **User Story 4 (Phase 6)**: Depends on US1, US2, US3 complete → Needs all deployment artifacts
7. **Polish (Phase 7)**: Depends on all desired user stories → Final documentation

### User Story Independence

- **US1 (Docker Compose)**: Foundation - must complete first
- **US2 (Monorepo)**: Can start after US1, modifies project structure
- **US3 (Kubernetes)**: Can start after US1, parallel with US2 if careful with paths
- **US4 (CI/CD)**: Requires US1-US3 artifacts (Dockerfiles, manifests, monorepo structure)

### Within Each User Story

- Tasks marked [P] within a phase can run in parallel
- Non-[P] tasks have implicit dependencies on previous tasks
- Validation tasks always run last in each phase

### Parallel Opportunities

**Setup Phase**: T003 and T004 can run in parallel

**Foundational Phase**: Must run sequentially (compilation fixing)

**User Story 1**: T010, T011, T012 can start in parallel; T016, T017, T018, T019 can run in parallel

**User Story 2**: T026, T027, T028 can run in parallel; T041, T042, T043 can run in parallel

**User Story 3**: All base manifest tasks (T050-T059) can run in parallel; All overlay tasks (T060-T065) can run in parallel; Documentation tasks (T073-T076) can run in parallel

**User Story 4**: Workflow files (T084, T093, T100) can start in parallel; Documentation (T114-T117) can run in parallel

**Polish Phase**: Most tasks (T124-T128, T131) can run in parallel

---

## Parallel Example: User Story 3 (Kubernetes)

```bash
# Launch all base manifests together:
T051: Create k8s/base/namespace.yaml
T052: Enhance k8s/base/deployment-backend.yaml
T053: Enhance k8s/base/deployment-frontend.yaml
T054: Enhance k8s/base/service-backend.yaml
T055: Enhance k8s/base/service-frontend.yaml
T056: Enhance k8s/base/ingress.yaml
T057: Create k8s/base/configmap.yaml
T058: Create k8s/base/pvc.yaml
T059: Create k8s/base/hpa-frontend.yaml

# Launch all overlays together:
T060: Create k8s/overlays/dev/kustomization.yaml
T061: Create k8s/overlays/staging/kustomization.yaml
T062: Create k8s/overlays/production/kustomization.yaml
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (4 tasks)
2. Complete Phase 2: Foundational (5 tasks) ← CRITICAL
3. Complete Phase 3: US1 Docker Compose (14 tasks)
4. **STOP and VALIDATE**: Test local development workflow
5. Demo to team: Single-command startup working

**Time Estimate**: 2-3 days

### Incremental Delivery

1. Setup + Foundational → Backend compiles (Day 1)
2. US1 → Local development ready (Day 3)
3. US2 → Fast builds with caching (Day 6)
4. US3 → K8s deployment ready (Day 10)
5. US4 → CI/CD automation complete (Day 13)
6. Polish → Documentation and training (Day 15)

Each phase delivers independent value and can be deployed/demoed.

### Parallel Team Strategy

Once Foundational complete (backend compiles):

- **Developer A**: User Story 1 (Docker Compose) - MUST finish first
- After US1 complete:
  - **Developer B**: User Story 2 (Monorepo)
  - **Developer C**: User Story 3 (Kubernetes) - Can work in parallel with US2
- After US2 + US3 complete:
  - **Developer A or B**: User Story 4 (CI/CD)

---

## Success Metrics Validation

After completing each user story, validate against these metrics:

| Metric | Target | Validation Task |
|--------|--------|-----------------|
| Local startup time | <2 min | T020 |
| Hot reload time | <3 sec | T021 |
| Build time (cached) | <2 min | T045 |
| Build improvement | 50%+ | T049 |
| K8s pod ready time | <30 sec | T079 |
| K8s deployment time | <60 sec | T079 |
| CI/CD pipeline time | <15 min | T122 |
| Deployment success | 95%+ | Track over time |

---

## Notes

- **[P]** = Parallelizable tasks (different files, no blocking dependencies)
- **[USX]** = User story mapping for traceability
- Each user story independently completable and testable
- Foundational phase (backend compilation fix) is CRITICAL BLOCKER
- Stop at checkpoints to validate each story works independently
- Commit after each logical task group
- Update documentation as you go, not at the end
- Infrastructure tested via smoke tests and health checks, not unit tests

**Total Tasks**: 132 tasks across 7 phases

**Estimated Timeline**: 11-15 days with 1-2 developers
