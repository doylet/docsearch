# 014 - Plan: Doc-Indexer Step 4 - Local Embeddings Pipeline Implementation

**Status:** 📋 PLANNING COMPLETE - READY FOR IMPLEMENTATION

**Date:** 19 August 2025

**Duration:** 3 weeks (Course correction from Step 3 deviation)

**Priority:** HIGH - Critical architecture realignment

## Overview

Step 4 implements the **Local Embeddings Pipeline** as originally planned in section 3 of the architecture document. This course-corrects the Step 3 deviation from OpenAI cloud integration back to the intended local-first, high-performance architecture.

## Problem Statement

**Current State (Step 3 Implementation):**
- Remote OpenAI API dependency (60 requests/minute)
- Internet connection required
- External API costs and rate limits
- Data privacy concerns (content sent to OpenAI)
- Performance regression vs original targets

**Target State (Original Plan):**
- Local embedding model (`gte-small` or equivalent)
- ≥200 chunks/minute processing throughput
- Zero external dependencies
- Local-first privacy preservation
- Cost-effective one-time setup

## Technical Requirements

### 1. Local Embedding Model Integration

#### **Model Selection:**
- **Primary Option**: `gte-small` (768-dimensional)
- **Alternative**: `all-MiniLM-L6-v2` (384-dimensional)
- **Format**: ONNX for cross-platform compatibility
- **Size**: <100MB model file for reasonable download

#### **Inference Framework:**
- **Primary**: `ort` (ONNX Runtime for Rust)
- **Alternative**: `candle-core` (Pure Rust ML framework)
- **Alternative**: `tch` (PyTorch bindings for Rust)

#### **Performance Requirements:**
```
Target: ≥200 chunks/minute on M-series Mac
Memory: <2GB total (model + inference + cache)
Startup: <10 seconds cold start (including model load)
Batch Size: 16-64 texts per inference call
```

### 2. Local Embedding API Endpoint

#### **API Contract** (as per original plan):

**POST** `/api/embed`

**Request:**
```json
{
  "model": "gte-small",
  "input": ["text 1", "text 2", "..."],
  "truncate": "END"
}
```

**Response:**
```json
{
  "model": "gte-small",
  "dimension": 768,
  "data": [
    {"index": 0, "embedding": [0.1, -0.2, ...]},
    {"index": 1, "embedding": [0.3, 0.1, ...]}
  ],
  "usage": {"input_tokens": 1234}
}
```

**Error Response:**
```json
{
  "error": {
    "type": "model_not_loaded",
    "message": "Embedding model not initialized",
    "retry_after_ms": 5000
  }
}
```

### 3. EmbeddingProvider Implementation

#### **LocalEmbedder Structure:**
```rust
pub struct LocalEmbedder {
    model: Arc<OnnxModel>,
    tokenizer: Arc<Tokenizer>,
    config: EmbeddingConfig,
    model_info: ModelInfo,
}

impl EmbeddingProvider for LocalEmbedder {
    async fn embed_batch(&self, texts: &[String]) -> Result<BatchEmbeddingResponse>;
    async fn embed_single(&self, text: &str) -> Result<Vec<f32>>;
    async fn health_check(&self) -> Result<bool>;
    fn config(&self) -> &EmbeddingConfig;
}
```

#### **Model Management:**
```rust
pub struct ModelManager {
    models: HashMap<String, Arc<OnnxModel>>,
    download_cache: PathBuf,
}

impl ModelManager {
    async fn load_model(&self, model_name: &str) -> Result<Arc<OnnxModel>>;
    async fn download_if_missing(&self, model_name: &str) -> Result<PathBuf>;
    fn get_model_info(&self, model_name: &str) -> Result<ModelInfo>;
}
```

### 4. Performance Optimization

#### **Batching Strategy:**
- **Optimal Batch Size**: 32-64 texts per inference call
- **Dynamic Batching**: Accumulate requests up to batch size or timeout
- **Memory Management**: Reuse buffers, avoid unnecessary allocations

#### **Caching Layer:**
```rust
pub struct EmbeddingCache {
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    max_size: usize,
    hash_algo: CacheHasher,
}
```

#### **Async Processing:**
- **Model Loading**: Background initialization with progress reporting
- **Inference Queue**: Bounded channel with backpressure handling
- **Result Streaming**: Return embeddings as soon as available

## Implementation Plan

### **Week 1: Research and Foundation**

#### **Day 1-2: Model Research**
- Evaluate ONNX Runtime vs Candle vs PyTorch bindings
- Test `gte-small` model availability and performance
- Benchmark inference speed on target hardware
- Validate model accuracy vs OpenAI embeddings

#### **Day 3-4: Architecture Design**
- Design `LocalEmbedder` implementation
- Plan model download and caching strategy
- Design embedding API endpoint architecture
- Create performance testing framework

#### **Day 5: Foundation Implementation**
- Set up ONNX Runtime integration
- Implement basic model loading
- Create embedding cache structure
- Add new dependencies to Cargo.toml

### **Week 2: Core Implementation**

#### **Day 1-2: LocalEmbedder Implementation**
```rust
// Core implementation tasks:
- OnnxModel wrapper for inference
- Tokenizer integration for text preprocessing  
- Batch processing with optimal sizing
- Memory management and buffer reuse
```

#### **Day 3-4: API Endpoint Integration**
```rust
// API server updates:
- Add POST /api/embed endpoint
- Integrate LocalEmbedder with API server
- Implement request batching and queuing
- Add error handling and health checks
```

#### **Day 5: Testing and Validation**
```rust
// Testing framework:
- Unit tests for LocalEmbedder
- Integration tests for /api/embed endpoint
- Performance benchmarks vs OpenAI
- Accuracy validation with known embeddings
```

### **Week 3: Performance Optimization and Integration**

#### **Day 1-2: Performance Tuning**
- Optimize batch sizes for throughput
- Implement embedding caching
- Tune memory usage and garbage collection
- Profile and eliminate bottlenecks

#### **Day 3-4: System Integration**
- Replace OpenAI provider with LocalEmbedder
- Update configuration management
- Implement model auto-download
- Add CLI options for local vs remote modes

#### **Day 5: Final Testing and Documentation**
- End-to-end performance validation
- Documentation updates
- Migration guide from OpenAI to local
- Performance comparison report

## Dependencies and Crates

### **New Dependencies:**
```toml
# ONNX Runtime for ML inference
ort = "2.0"

# Tokenization
tokenizers = "0.19"

# HTTP client for model downloads
reqwest = { version = "0.12", features = ["stream"] }

# Caching
lru = "0.12"

# Async file I/O
tokio = { version = "1.0", features = ["fs", "io-util"] }

# Model file handling
zip = "0.6"
tar = "0.4"
```

### **Model Assets:**
```
models/
├── gte-small/
│   ├── model.onnx          # ONNX model file
│   ├── tokenizer.json      # Tokenizer configuration
│   ├── config.json         # Model metadata
│   └── README.md           # Model documentation
└── downloads/              # Temporary download cache
```

## Configuration Updates

### **CLI Options:**
```bash
doc-indexer [OPTIONS]

# New embedding options:
--embedding-provider <PROVIDER>     # local, openai, mock [default: local]
--embedding-model <MODEL>           # gte-small, all-MiniLM-L6-v2 [default: gte-small]
--model-cache-dir <PATH>            # Model download cache [default: ~/.cache/doc-indexer]
--embedding-batch-size <SIZE>       # Batch size for inference [default: 32]
--embedding-cache-size <SIZE>       # LRU cache size [default: 10000]

# Existing options remain unchanged
--api-server                        # Start HTTP API server
--api-port <PORT>                   # API server port [default: 3000]
```

### **Configuration File Support:**
```toml
[embedding]
provider = "local"
model = "gte-small"
batch_size = 32
cache_size = 10000
model_cache_dir = "~/.cache/doc-indexer"

[embedding.local]
auto_download = true
warm_up_on_start = true
max_memory_mb = 2048

[embedding.openai]
api_key = "${OPENAI_API_KEY}"
rate_limit_rpm = 60
```

## Testing Strategy

### **Unit Tests:**
- `LocalEmbedder` embedding generation
- Model loading and initialization
- Tokenization and preprocessing
- Caching behavior and eviction
- Error handling and recovery

### **Integration Tests:**
- `/api/embed` endpoint functionality
- End-to-end search with local embeddings
- Performance under load
- Model download and setup
- Configuration management

### **Performance Tests:**
```rust
#[tokio::test]
async fn test_local_embedding_throughput() {
    // Target: ≥200 chunks/minute
    let embedder = LocalEmbedder::new("gte-small").await?;
    let chunks = generate_test_chunks(1000);
    let start = Instant::now();
    
    for batch in chunks.chunks(32) {
        embedder.embed_batch(batch).await?;
    }
    
    let duration = start.elapsed();
    let chunks_per_minute = (1000.0 / duration.as_secs_f64()) * 60.0;
    assert!(chunks_per_minute >= 200.0);
}
```

### **Accuracy Tests:**
```rust
#[tokio::test]
async fn test_embedding_accuracy() {
    // Compare local vs reference embeddings
    let local_embedder = LocalEmbedder::new("gte-small").await?;
    let reference_embeddings = load_reference_embeddings();
    
    for (text, expected) in reference_embeddings {
        let actual = local_embedder.embed_single(&text).await?;
        let similarity = cosine_similarity(&actual, &expected);
        assert!(similarity > 0.95); // 95% similarity threshold
    }
}
```

## Migration Strategy

### **Backward Compatibility:**
- Keep OpenAI provider as fallback option
- Maintain existing API contracts
- Gradual migration with feature flags
- Configuration-driven provider selection

### **Migration Steps:**
1. **Deploy with local provider disabled** (Week 1)
2. **Enable local provider in testing** (Week 2)
3. **Default to local provider** (Week 3)
4. **Deprecate OpenAI provider** (Future step)

### **Rollback Plan:**
- Feature flag to revert to OpenAI provider
- Configuration validation before startup
- Health checks to detect local model issues
- Automatic fallback on local provider failures

## Success Criteria

### **Performance Targets:**
- ✅ **Throughput**: ≥200 chunks/minute on M-series Mac
- ✅ **Latency**: <100ms for single embedding
- ✅ **Memory**: <2GB total memory usage
- ✅ **Startup**: <10 seconds cold start

### **Functional Requirements:**
- ✅ **Zero Dependencies**: No internet required for core operation
- ✅ **API Compatibility**: `/api/embed` endpoint fully functional
- ✅ **Search Quality**: Equivalent results to OpenAI embeddings
- ✅ **Error Handling**: Graceful failure and recovery

### **Quality Gates:**
- ✅ **Unit Test Coverage**: >90% for new code
- ✅ **Integration Tests**: All API endpoints functional
- ✅ **Performance Tests**: Meet throughput targets
- ✅ **Documentation**: Complete API and usage documentation

## Risk Mitigation

### **Technical Risks:**
1. **Performance Risk**: Local model slower than expected
   - **Mitigation**: Benchmark early, optimize batch processing
2. **Memory Risk**: Model too large for target hardware
   - **Mitigation**: Model size validation, memory monitoring
3. **Accuracy Risk**: Local embeddings different quality
   - **Mitigation**: Accuracy tests vs reference embeddings

### **Operational Risks:**
1. **Model Download Risk**: Large model files, slow downloads
   - **Mitigation**: CDN hosting, resumable downloads, local cache
2. **Compatibility Risk**: ONNX Runtime platform issues
   - **Mitigation**: Multiple inference backend options
3. **Migration Risk**: Breaking existing functionality
   - **Mitigation**: Feature flags, gradual rollout, rollback plan

## Documentation Deliverables

1. **API Documentation**: Updated `/api/embed` endpoint specification
2. **Configuration Guide**: Local embedding setup and tuning
3. **Performance Report**: Benchmarks vs OpenAI comparison
4. **Migration Guide**: Steps to transition from OpenAI to local
5. **Troubleshooting Guide**: Common issues and solutions

## Next Steps After Step 4

With local embeddings implemented, we'll be positioned for:

- **Step 5**: Schema alignment and advanced chunking
- **Step 6**: CLI completion with `mdx` commands  
- **Step 7**: JSON-RPC interface implementation
- **Step 8**: Performance optimization and production hardening

## Ready for Implementation

Step 4 planning is complete. Ready to:
1. Create feature branch `feature/step-4-local-embeddings`
2. Begin Week 1 research and foundation work
3. Implement local embedding pipeline as specified

**Let's restore the local-first architecture!** 🚀
