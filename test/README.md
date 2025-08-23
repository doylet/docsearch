# Zero-Latency Test Suite

This directory contains the comprehensive testing and validation framework for the Zero-Latency document search system.

## 🧪 **Test Scripts**

### **Primary Validation**
```bash
# Quick validation of core functionality
./test/simple_validation.sh
```
- Builds embedded variant with feature flags
- Tests JSON-RPC transport compliance
- Validates service discovery and health monitoring
- Confirms performance characteristics

### **Comprehensive Testing**
```bash
# Full multi-variant validation
./test/pipeline_validation.sh
```
- Tests all feature combinations (embedded/cloud/full)
- Performance benchmarking across variants
- Integration testing with automated reporting
- Complete workflow validation

## 📊 **Validation Results**

### **Performance Benchmarks**
- **Build Time**: 9-10 seconds (release builds)
- **Startup Time**: <1 second to operational state
- **Memory Footprint**: ~50MB baseline (embedded variant)
- **Response Latency**: <100ms (JSON-RPC service calls)

### **Service Capabilities Validated**
```json
{
  "capabilities": {
    "document_indexing": true,
    "health_monitoring": true,
    "realtime_updates": false,
    "vector_search": true
  },
  "transport": ["stdio", "http"],
  "protocol_version": "2.0"
}
```

## 🎯 **Feature Variant Testing**

### **Embedded Build**
```bash
cargo build --release --features embedded --no-default-features
```
- Local SQLite vector storage
- ONNX Runtime for ML inference
- Self-contained deployment
- No external dependencies

### **Cloud Build**
```bash
cargo build --release --features cloud --no-default-features
```
- Qdrant vector database integration
- OpenAI API for embeddings
- External service connectivity
- Scalable deployment

### **Full Build**
```bash
cargo build --release --features full
```
- Complete feature set
- Development and testing
- All capabilities enabled

## 📁 **Directory Structure**

```
test/
├── simple_validation.sh      # Quick validation script
├── pipeline_validation.sh    # Comprehensive testing
├── results/                  # Test output and logs
└── integration/              # Integration test files
    └── README.md            # Integration testing documentation
```

## 🔍 **Test Coverage**

### **Build System Validation**
- ✅ All feature combinations compile successfully
- ✅ Dependency isolation working correctly
- ✅ Feature flag behavior validated
- ✅ Binary optimization confirmed

### **Transport Layer Testing**
- ✅ JSON-RPC 2.0 specification compliance
- ✅ HTTP and stdio transport operational
- ✅ Service discovery endpoints responding
- ✅ Health monitoring comprehensive

### **Performance Validation**
- ✅ Sub-second startup times achieved
- ✅ Memory usage within acceptable ranges
- ✅ Response times meeting targets
- ✅ Build performance optimized

### **Integration Testing**
- ✅ End-to-end workflow validation
- ✅ Service lifecycle management
- ✅ Error handling and recovery
- ✅ Configuration management

## 🚀 **Usage Examples**

### **Quick Development Check**
```bash
# Fast validation during development
./test/simple_validation.sh
```

### **Pre-Release Validation**
```bash
# Comprehensive testing before release
./test/pipeline_validation.sh
```

### **Feature-Specific Testing**
```bash
# Test specific build variant
cd services/doc-indexer
cargo build --release --features embedded --no-default-features
cargo test --features embedded --no-default-features
```

### **Performance Monitoring**
```bash
# Monitor build and runtime performance
time cargo build --release
./target/release/doc-indexer --port 8081 &
curl -X POST http://localhost:8081/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"health.check","id":1}'
```

## 📋 **Quality Assurance**

This test suite ensures:
- **Production Readiness**: All deployment scenarios validated
- **Performance Standards**: Sub-second startup, <100ms responses
- **Feature Isolation**: No bleeding between build variants  
- **Transport Compliance**: Full JSON-RPC 2.0 specification adherence
- **Error Handling**: Graceful failure modes and recovery
- **Documentation Coverage**: Complete guides and API documentation

The validation framework provides comprehensive quality assurance for the Zero-Latency system across all supported deployment configurations.
