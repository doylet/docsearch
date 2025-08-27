# 040 - Configuration Architecture Decision Record

**Date:** August 27, 2025  
**Status:** PROPOSED  
**Decision Makers:** GitHub Copilot, Architecture Review  
**Related:** [039](039-json-rpc-mcp-protocol-compliance.md), Configuration Management Analysis  

---

## Problem Statement

The Zero-Latency codebase contains numerous hardcoded configuration values scattered throughout multiple components:

### **Current Issues:**
- **Network Configuration**: Hardcoded ports (`8081`, `18081`, `18082`) and hosts (`localhost`, `127.0.0.1`)
- **File Paths**: Binary paths (`../../target/debug/doc-indexer`), user-specific paths (`/Users/...`)
- **Collection Names**: Test collections (`advanced_test_collection`, `smoke_test_collection`) and defaults
- **Timeouts & Limits**: Request timeouts, retry counts, health check intervals
- **Log Levels**: Hardcoded `"info"`, `"debug"` levels
- **Test Isolation**: Port conflicts and collection name collisions in concurrent tests

### **Impact:**
- Poor maintainability and deployment flexibility
- Test reliability issues due to hardcoded ports and collections
- Difficulty in environment-specific configuration
- Violation of 12-factor app configuration principles

---

## Decision

We will implement a **hierarchical configuration architecture** with the following design:

### **1. Configuration Hierarchy (Precedence Order)**
```
Environment Variables (highest precedence)
    ↓
Configuration Files (~/.config/zero-latency.toml)
    ↓ 
Centralized Defaults (zero-latency-config crate)
```

### **2. Configuration Structure**
```rust
// Core configuration types
pub struct AppConfig {
    pub server: ServerConfig,
    pub client: ClientConfig, 
    pub logging: LoggingConfig,
    pub storage: StorageConfig,
    pub test: Option<TestConfig>,
}

pub struct ServerConfig {
    pub host: String,           // Default: "localhost"
    pub port: u16,              // Default: 8081  
    pub docs_path: PathBuf,     // Default: "./docs"
    pub collection_name: String, // Default: "zero_latency_docs"
    pub request_timeout_ms: u64, // Default: 30000
}

pub struct TestConfig {
    pub base_port: u16,         // Default: 18000
    pub fixture_path: PathBuf,  // Default: "tests/fixtures"
    pub binary_path: PathBuf,   // Resolved dynamically
    pub collection_prefix: String, // Default: "test_"
}
```

### **3. Configuration Loading**
```rust
pub trait ConfigLoader {
    fn load(&self) -> Result<AppConfig>;
}

pub struct ConfigResolver {
    loaders: Vec<Box<dyn ConfigLoader>>,
}

impl ConfigResolver {
    pub fn resolve(&self) -> Result<AppConfig> {
        // Apply precedence: env vars > config files > defaults
    }
}
```

### **4. Test Configuration Utilities**
```rust
pub struct TestConfigHelper;

impl TestConfigHelper {
    pub fn get_unique_port() -> u16;        // Atomic port allocation
    pub fn get_unique_collection() -> String; // Timestamp-based names
    pub fn get_binary_path() -> PathBuf;    // Multi-location resolution
    pub fn create_test_config() -> TestConfig; // Complete test setup
}
```

---

## Rationale

### **Why This Architecture:**

1. **Eliminates Hardcoding**: Single source of truth for all configuration values
2. **Deployment Flexibility**: Environment variables enable container-friendly configuration
3. **Test Reliability**: Unique ports and collections prevent conflicts
4. **Maintainability**: Centralized configuration management
5. **Backward Compatibility**: Preserves existing CLI argument parsing

### **Alternative Considered:**
- **Config-only approach**: Rejected due to lack of environment variable support
- **Environment-only approach**: Rejected due to complexity for local development
- **Per-component configs**: Rejected due to duplication and inconsistency

---

## Implementation Strategy

### **Phase 1: Core Infrastructure (2-3 hours)**
- Enhance `zero-latency-config` crate with configuration structs
- Implement `ConfigLoader` trait and resolvers  
- Create `TestConfigHelper` utilities

### **Phase 2: CLI Integration (1-2 hours)**
- Update CLI commands to use `ConfigResolver`
- Modify HTTP clients to use `ClientConfig`
- Ensure CLI args override config values

### **Phase 3: Test Refactoring (2-3 hours)**
- Replace hardcoded ports with `TestConfigHelper.get_unique_port()`
- Use dynamic collection names and paths
- Remove user-specific hardcoded paths

### **Phase 4: Service Integration (1-2 hours)**
- Update doc-indexer service startup to load configuration
- Modify ServiceContainer to use configuration
- Replace all remaining hardcoded values

---

## Environment Variable Convention

```bash
# Server configuration
ZL_SERVER_HOST=0.0.0.0
ZL_SERVER_PORT=8081
ZL_DOCS_PATH=/path/to/docs
ZL_COLLECTION_NAME=production_docs
ZL_LOG_LEVEL=info

# Test configuration
ZL_TEST_PORT_BASE=19000
ZL_TEST_FIXTURE_PATH=./test-fixtures
ZL_TEST_BINARY_PATH=./custom/doc-indexer
```

## Configuration File Example

```toml
# ~/.config/zero-latency.toml
[server]
host = "0.0.0.0"
port = 8081
docs_path = "/opt/docs"
collection_name = "production_docs"

[client]
server_url = "http://localhost:8081"
request_timeout_ms = 30000

[logging]
level = "info"
format = "json"

[test]
base_port = 19000
fixture_path = "./custom-fixtures"
```

---

## Success Criteria

- [ ] **Zero hardcoded network configuration** in source code
- [ ] **Zero hardcoded file paths** (except fallback resolution logic)  
- [ ] **Zero hardcoded collection names** in tests
- [ ] **All tests pass** with unique ports and collections
- [ ] **Full environment variable support** for all configuration
- [ ] **Configuration file support** working correctly
- [ ] **Backward compatibility** maintained for existing usage
- [ ] **Documentation complete** with examples and troubleshooting

---

## Risks & Mitigations

### **Risk: Breaking Changes**
- **Mitigation**: Maintain backward compatibility, gradual migration

### **Risk: Increased Complexity**  
- **Mitigation**: Clear documentation, simple configuration patterns

### **Risk: Test Failures During Migration**
- **Mitigation**: Implement TestConfigHelper first, migrate tests incrementally

---

## Related Work

- **ADR 008**: Contract-first daemon strategy provides foundation
- **ADR 039**: JSON-RPC MCP compliance establishes configuration patterns
- **Sprint 001**: Advanced search pipeline activation demonstrates current configuration challenges

---

**Decision Status:** PROPOSED  
**Next Actions:** Begin Phase 1 implementation of core configuration infrastructure  
**Review Date:** After Phase 1 completion  
**Approval Required:** Architecture team review before Phase 2
