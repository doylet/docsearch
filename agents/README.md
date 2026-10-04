# Agent System

**Purpose**: Discrete collection of agents, each controlling a specific responsibility for efficient and productive product engineering tasks.

**Status**: Design Phase
**Last Updated**: December 2025

---

## 🎯 Overview

The Agent System is a comprehensive framework for automating product engineering tasks through specialized, autonomous agents. Each agent has a single, well-defined responsibility and operates independently while coordinating through defined interfaces.

### Key Principles

- **Discrete Responsibilities**: Each agent has one clear accountability
- **Autonomous Operation**: Agents can operate independently
- **Observable Outcomes**: All agents produce measurable results
- **Fail-Safe Design**: Agent failures are isolated
- **Composable Workflows**: Agents can be orchestrated into workflows

---

## 📚 Documentation

### Core Documentation

1. **[ARCHITECTURE.md](./ARCHITECTURE.md)** - System architecture and design principles
2. **[SPECIFICATIONS.md](./SPECIFICATIONS.md)** - Detailed agent specifications
3. **[ORCHESTRATION.md](./ORCHESTRATION.md)** - Workflow patterns and coordination
4. **[IMPLEMENTATION.md](./IMPLEMENTATION.md)** - Implementation guide (coming soon)

### Quick Reference

- **Agent Catalog**: See [SPECIFICATIONS.md](./SPECIFICATIONS.md#agent-catalog)
- **Workflow Patterns**: See [ORCHESTRATION.md](./ORCHESTRATION.md#workflow-patterns)
- **Agent Interface**: See [ARCHITECTURE.md](./ARCHITECTURE.md#agent-interface)

---

## 🤖 Agent Catalog

### Core Engineering Agents

| Agent | ID | Responsibility |
|-------|-----|----------------|
| **Code Quality Agent** | `code-quality` | Linting, formatting, static analysis |
| **Testing Agent** | `testing` | Unit, integration, E2E tests |
| **Build Agent** | `build` | Compilation, bundling, artifacts |
| **Schema Agent** | `schema` | OpenAPI validation, contract generation |
| **Security Agent** | `security` | Vulnerability scanning, dependency audits |
| **Documentation Agent** | `documentation` | Doc generation, API docs, README updates |
| **Deployment Agent** | `deployment` | Docker builds, K8s manifests, CI/CD |
| **Performance Agent** | `performance` | Benchmarks, profiling, optimization |
| **Architecture Agent** | `architecture` | Clean architecture compliance |
| **Dependency Agent** | `dependency` | Version management, updates, compatibility |

---

## 🚀 Quick Start

### Running a Single Agent

```bash
# Run Code Quality Agent
cargo run --bin agent -- code-quality --workspace .

# Run Testing Agent
cargo run --bin agent -- testing --workspace .

# Run Build Agent
cargo run --bin agent -- build --workspace . --target release
```

### Running a Workflow

```bash
# Run pre-commit workflow
cargo run --bin agent-orchestrator -- workflow pre-commit

# Run CI validation workflow
cargo run --bin agent-orchestrator -- workflow ci-validation

# Run deployment workflow
cargo run --bin agent-orchestrator -- workflow deployment --env staging
```

### Using Make Targets

```bash
# Run all agents (pre-commit workflow)
make agents-pre-commit

# Run CI validation
make agents-ci-validation

# Run specific agent
make agent-code-quality
```

---

## 📋 Common Workflows

### Pre-Commit Workflow

Fast feedback before code is committed:

```yaml
agents:
  - code-quality
  - schema
  - build
```

**Usage**:
```bash
make agents-pre-commit
```

### CI Validation Workflow

Comprehensive validation on pull requests:

```yaml
agents:
  - code-quality (parallel)
  - security (parallel)
  - schema (parallel)
  - dependency (parallel)
  - build (depends on code-quality, schema)
  - testing (depends on build)
  - architecture (depends on build)
  - documentation (depends on build, schema)
```

**Usage**:
```bash
make agents-ci-validation
```

### Deployment Workflow

Safe deployment to target environments:

```yaml
agents:
  - build (required)
  - security (required, fail on critical)
  - testing (required, coverage >= 80%)
  - schema (required)
  - deployment (depends on all above)
  - health-check (depends on deployment)
```

**Usage**:
```bash
make agents-deploy --env staging
```

---

## 🔧 Configuration

### Agent Configuration File

Create `agents/config.yaml`:

```yaml
agents:
  code-quality:
    enabled: true
    timeout: 30s
    fail_fast: true

  testing:
    enabled: true
    timeout: 5m
    coverage_threshold: 80

  security:
    enabled: true
    fail_on_critical: true
    severity_threshold: high
```

### Environment Variables

```bash
# Agent execution
export AGENT_WORKSPACE=/path/to/workspace
export AGENT_LOG_LEVEL=info
export AGENT_PARALLEL=true

# Agent-specific
export CODE_QUALITY_STRICT=true
export TESTING_COVERAGE_THRESHOLD=80
export SECURITY_FAIL_ON_CRITICAL=true
```

---

## 📊 Monitoring

### Agent Metrics

Each agent exposes metrics:

```bash
# View agent metrics
curl http://localhost:9090/metrics/agent/code-quality

# View workflow metrics
curl http://localhost:9090/metrics/workflow/pre-commit
```

### Agent Logs

```bash
# View agent logs
tail -f logs/agents/code-quality.log

# View all agent logs
tail -f logs/agents/*.log
```

---

## 🛠️ Development

### Adding a New Agent

1. **Create agent module**:
   ```bash
   mkdir -p agents/my-agent/src
   ```

2. **Implement Agent trait**:
   ```rust
   use agents::core::Agent;

   pub struct MyAgent {
       // Agent implementation
   }

   impl Agent for MyAgent {
       fn id(&self) -> &'static str { "my-agent" }
       async fn execute(&self, context: AgentContext) -> AgentResult {
           // Agent logic
       }
   }
   ```

3. **Register agent**:
   ```rust
   orchestrator.register_agent(Box::new(MyAgent::new()));
   ```

4. **Add to workflows**:
   ```yaml
   # workflows/my-workflow.yaml
   agents:
     - my-agent
   ```

### Testing Agents

```bash
# Run agent tests
cargo test --package agents --lib

# Run integration tests
cargo test --test agent_integration
```

---

## 📈 Success Metrics

### Agent Effectiveness

- **Autonomy**: Agents operate independently 95%+ of the time
- **Reliability**: Agent success rate > 99%
- **Performance**: Agent execution time < 5 minutes (95th percentile)
- **Observability**: 100% of agent executions logged and traced

### Engineering Productivity

- **Automation**: 80%+ of engineering tasks automated
- **Feedback Time**: < 2 minutes from code change to agent feedback
- **Error Detection**: 90%+ of issues caught before merge
- **Documentation**: 100% of API changes automatically documented

---

## 🔗 Related Documentation

- **[PROJECT_OVERVIEW.md](../PROJECT_OVERVIEW.md)** - Project overview
- **[docs/CURRENT_ARCHITECTURE.md](../docs/CURRENT_ARCHITECTURE.md)** - System architecture
- **[Makefile](../Makefile)** - Build commands

---

## 🤝 Contributing

When adding new agents or workflows:

1. Follow the agent specification template in [SPECIFICATIONS.md](./SPECIFICATIONS.md)
2. Implement the `Agent` trait from the core framework
3. Add comprehensive tests
4. Document agent behavior and configuration
5. Update workflow definitions as needed

---

**Status**: This agent system is in design phase. Implementation will follow the specifications outlined in the documentation.
