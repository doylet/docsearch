# Sprint Plan: Configuration Architecture Implementation

**Sprint ID:** ZL-002  
**Sprint Name:** Configuration Architecture Implementation  
**Start Date:** August 28, 2025  
**End Date:** September 2, 2025  
**Duration:** 6 days  
**Sprint Goal:** Eliminate all hardcoded configuration values and implement centralized configuration management  
**Related:** [ADR 040](../adr/040-configuration-architecture-centralization.md), Configuration Analysis  

---

## 🎯 Sprint Objective

Transform the Zero-Latency system from scattered hardcoded configuration values to a centralized, hierarchical configuration architecture that supports environment variables, configuration files, and dynamic test configuration.

**Success Criteria:**
- [ ] Zero hardcoded network configuration in source code
- [ ] Zero hardcoded file paths (except fallback resolution logic)
- [ ] Zero hardcoded collection names in tests
- [ ] All tests pass with unique ports and collections
- [ ] Full environment variable support for all configuration
- [ ] Configuration file support working correctly
- [ ] Backward compatibility maintained

---

## 📋 Sprint Backlog

### **Epic 1: Core Configuration Infrastructure**
**Story Points:** 13  
**Priority:** Critical  

#### **ZL-002-001: Enhance zero-latency-config Crate** 
**Story Points:** 5  
**Priority:** Critical  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] Implement comprehensive configuration structs (AppConfig, ServerConfig, ClientConfig, TestConfig)
- [ ] Add Default implementations with all current hardcoded values centralized
- [ ] Create environment variable mapping with ZL_ prefix convention
- [ ] Implement ConfigLoader trait with file and environment loaders
- [ ] Add ConfigResolver with precedence handling (env > file > defaults)

**Tasks:**
- [ ] Define configuration structs in `src/models.rs`
- [ ] Implement Default traits with current hardcoded values
- [ ] Create EnvConfigLoader and FileConfigLoader implementations
- [ ] Add ConfigResolver with merge logic
- [ ] Add comprehensive configuration validation
- [ ] Write unit tests for configuration loading

#### **ZL-002-002: Create Test Configuration Utilities**
**Story Points:** 5  
**Priority:** Critical  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] TestConfigHelper with unique port allocation using atomic counters
- [ ] Dynamic collection name generation with timestamp uniqueness
- [ ] Binary path resolution across multiple candidate locations
- [ ] Complete test configuration factory method
- [ ] Environment variable override support for test configuration

**Tasks:**
- [ ] Implement TestConfigHelper with atomic port counter
- [ ] Add unique collection name generation
- [ ] Create binary path resolution with fallback logic
- [ ] Add test fixture path resolution
- [ ] Implement complete test configuration factory
- [ ] Add environment variable support for test overrides

#### **ZL-002-003: Configuration File Support**
**Story Points:** 3  
**Priority:** High  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] TOML configuration file parsing
- [ ] Default configuration file locations (~/.config/zero-latency.toml)
- [ ] Configuration file validation and error handling
- [ ] Support for partial configuration files (defaults fill gaps)

**Tasks:**
- [ ] Add serde and toml dependencies
- [ ] Implement TOML file parsing
- [ ] Add configuration file location resolution
- [ ] Implement configuration merging logic
- [ ] Add validation and error handling

---

### **Epic 2: CLI Integration**
**Story Points:** 8  
**Priority:** High  

#### **ZL-002-004: Update CLI Commands**
**Story Points:** 5  
**Priority:** High  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] Server command uses ConfigResolver for all settings
- [ ] CLI arguments override configuration file values
- [ ] All client commands load configuration consistently  
- [ ] --config-file flag for custom configuration paths

**Tasks:**
- [ ] Modify server command to use ConfigResolver
- [ ] Update argument parsing to override config values
- [ ] Add --config-file argument support
- [ ] Ensure all commands use consistent configuration loading
- [ ] Update help text to document configuration options

#### **ZL-002-005: Update CLI HTTP Clients**
**Story Points:** 3  
**Priority:** High  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] All API clients use ClientConfig for URLs and timeouts
- [ ] Replace hardcoded localhost:8081 with config-driven URLs
- [ ] Proper timeout and retry configuration from config

**Tasks:**
- [ ] Modify SearchApiClient to use ClientConfig
- [ ] Update IndexApiClient, DocumentApiClient, etc.
- [ ] Replace hardcoded URLs with config.server_url()
- [ ] Add timeout and retry configuration

---

### **Epic 3: Test Refactoring**
**Story Points:** 13  
**Priority:** High  

#### **ZL-002-006: Refactor Smoke Tests**
**Story Points:** 8  
**Priority:** High  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] Replace hardcoded ports (18081, 18082) with TestConfigHelper.get_unique_port()
- [ ] Use dynamic collection names from TestConfigHelper
- [ ] Replace hardcoded binary paths with resolved paths
- [ ] Remove user-specific paths (/Users/...)
- [ ] Use config-driven URLs and timeouts

**Tasks:**
- [ ] Update smoke_test_advanced_query_enhancement_and_ranking()
- [ ] Update smoke_test_end_to_end_index_and_search()
- [ ] Update smoke_test_cli_runs_with_docs_path()
- [ ] Replace all hardcoded ports with unique port allocation
- [ ] Replace all hardcoded collection names with dynamic names
- [ ] Update binary path resolution

#### **ZL-002-007: Create Test Utilities Module**
**Story Points:** 3  
**Priority:** Medium  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] Test setup/teardown helpers
- [ ] Server startup/shutdown utilities  
- [ ] Test data management helpers
- [ ] Common test configuration patterns

**Tasks:**
- [ ] Create test utilities module in services/doc-indexer/tests/
- [ ] Add server lifecycle management
- [ ] Create test data setup helpers
- [ ] Add common assertion helpers

#### **ZL-002-008: Integration Test Updates**
**Story Points:** 2  
**Priority:** Medium  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] All integration tests use dynamic configuration
- [ ] Environment variable support for CI/CD
- [ ] Test isolation guarantees

**Tasks:**
- [ ] Apply configuration changes to integration tests
- [ ] Add CI/CD environment variable support
- [ ] Verify test isolation

---

### **Epic 4: Service Integration**
**Story Points:** 8  
**Priority:** Medium  

#### **ZL-002-009: Update Doc-Indexer Service**
**Story Points:** 5  
**Priority:** Medium  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] Service startup loads configuration using ConfigResolver
- [ ] All hardcoded values replaced with config references
- [ ] Configuration validation at startup
- [ ] Graceful handling of configuration errors

**Tasks:**
- [ ] Modify main.rs to load configuration at startup
- [ ] Update ServiceContainer to accept configuration
- [ ] Replace hardcoded values in service initialization
- [ ] Add configuration validation and error handling

#### **ZL-002-010: Update Container/Dependency Injection**
**Story Points:** 3  
**Priority:** Medium  
**Status:** Ready  

**Acceptance Criteria:**
- [ ] ServiceContainer uses configuration for all service creation
- [ ] Configuration-based feature toggling
- [ ] Proper configuration propagation to all services

**Tasks:**
- [ ] Modify ServiceContainer to accept AppConfig
- [ ] Update all service constructors to use config
- [ ] Add configuration-based feature flags
- [ ] Ensure configuration is properly injected

---

## 📊 Sprint Metrics

**Total Story Points:** 42  
**Velocity Target:** 7 points per day  
**Risk Buffer:** 10% (included in estimates)  

**Daily Breakdown:**
- **Day 1:** Epic 1 (13 points) - Core Configuration Infrastructure
- **Day 2:** Epic 2 (8 points) - CLI Integration  
- **Day 3-4:** Epic 3 (13 points) - Test Refactoring
- **Day 5:** Epic 4 (8 points) - Service Integration
- **Day 6:** Testing, documentation, and polish

---

## 🛠️ Technical Implementation Plan

### **Dependencies Required:**
```toml
# Add to zero-latency-config/Cargo.toml
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "5.0"  # For config file location resolution

# Add to services/doc-indexer/Cargo.toml  
zero-latency-config = { path = "../../crates/zero-latency-config" }
```

### **Environment Variables Convention:**
```bash
# Server configuration
ZL_SERVER_HOST=0.0.0.0
ZL_SERVER_PORT=8081
ZL_DOCS_PATH=/path/to/docs
ZL_COLLECTION_NAME=production_docs
ZL_LOG_LEVEL=info

# Client configuration
ZL_CLIENT_SERVER_URL=http://server:8081
ZL_CLIENT_TIMEOUT_MS=30000

# Test configuration
ZL_TEST_PORT_BASE=19000
ZL_TEST_FIXTURE_PATH=./test-fixtures
ZL_TEST_BINARY_PATH=./custom/doc-indexer
```

---

## 🔍 Testing Strategy

### **Unit Tests:**
- [ ] Configuration loading from environment variables
- [ ] Configuration loading from TOML files
- [ ] Configuration precedence (env > file > defaults)
- [ ] TestConfigHelper unique port allocation
- [ ] Binary path resolution logic

### **Integration Tests:**
- [ ] End-to-end configuration loading in CLI
- [ ] Service startup with various configuration sources
- [ ] Test isolation with unique ports and collections

### **Manual Testing:**
- [ ] Configuration file creation and loading
- [ ] Environment variable override behavior
- [ ] CLI argument override behavior
- [ ] Test execution with configuration

---

## 📈 Success Metrics

### **Code Quality:**
- [ ] Zero grep matches for hardcoded ports (8081, 18081, 18082)
- [ ] Zero grep matches for hardcoded localhost/127.0.0.1
- [ ] Zero grep matches for hardcoded collection names in tests
- [ ] Zero grep matches for user-specific paths (/Users/)

### **Test Reliability:**
- [ ] All tests pass with parallel execution
- [ ] No port conflicts in concurrent test runs
- [ ] No collection name collisions

### **Configuration Coverage:**
- [ ] All CLI commands support configuration files
- [ ] All services load configuration consistently
- [ ] All environment variables documented and working

---

## 🚀 Sprint Deliverables

### **Code Artifacts:**
- [ ] Enhanced zero-latency-config crate with full configuration support
- [ ] TestConfigHelper utilities for reliable test execution
- [ ] Refactored CLI commands using configuration
- [ ] Updated smoke tests with dynamic configuration
- [ ] Modified doc-indexer service with configuration loading

### **Documentation:**
- [ ] Configuration guide with examples
- [ ] Environment variable reference  
- [ ] Migration guide for existing deployments
- [ ] Troubleshooting guide for configuration issues

### **Testing:**
- [ ] Comprehensive test suite for configuration loading
- [ ] Updated integration tests using dynamic configuration
- [ ] CI/CD configuration for environment variables

---

## 🔧 Development Environment Setup

### **Required Tools:**
- Rust 1.70+ with cargo
- Editor with Rust language support
- Local testing environment

### **Setup Commands:**
```bash
# Install additional dependencies
cargo install --locked cargo-edit

# Add new dependencies
cd crates/zero-latency-config
cargo add serde --features derive
cargo add toml
cargo add dirs

# Verify compilation
cargo check --all
```

---

## 📝 Sprint Retrospective Planning

### **Review Questions:**
1. Which configuration patterns were most effective?
2. What configuration loading challenges were encountered?
3. How well did the TestConfigHelper prevent test conflicts?
4. What documentation gaps remain?

### **Continuous Improvement:**
- Configuration pattern standardization
- Test reliability improvements  
- Development experience optimization
- Deployment configuration simplification

---

**Sprint Master:** GitHub Copilot  
**Created:** August 27, 2025  
**Status:** 🚀 SPRINT READY - Configuration Architecture Implementation  
**Next Review:** Daily standups, Sprint end review September 2, 2025  

---

## 🔗 Related Documentation

- **ADR 040:** [Configuration Architecture Centralization](../adr/040-configuration-architecture-centralization.md)
- **Sprint 001:** [Advanced Search Pipeline Activation](sprint-001-advanced-search-pipeline-activation.md)
- **Implementation Guide:** Configuration setup and migration instructions
