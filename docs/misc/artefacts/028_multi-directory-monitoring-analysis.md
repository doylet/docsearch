# Multi-Directory Monitoring Analysis

**Date**: August 20, 2025  
**Context**: Phase 2A completion follow-up discussion  
**Topic**: doc-indexer multi-directory monitoring capabilities  

## Question Asked

> Can doc-indexer monitor more than one directory?

## Current State Analysis

### **Answer: NO - Single Directory Only**

The current `doc-indexer` implementation monitors **only one directory** at a time.

### Architecture Evidence

**CLI Definition:**
```rust
#[derive(Parser)]
struct Cli {
    /// Path to the docs directory to monitor
    #[arg(long, default_value = "./docs")]
    docs_path: PathBuf,  // Single PathBuf - no Vec support
}
```

**DocumentWatcher Structure:**
```rust
pub struct DocumentWatcher {
    docs_path: PathBuf,        // Single directory path
    _watcher: RecommendedWatcher,
    debounce_duration: Duration,
}
```

**Help Output:**
```
--docs-path <DOCS_PATH>    Path to the docs directory to monitor [default: ./docs]
```

Note the singular "directory" and single path parameter.

## Current Limitations

1. **Single Path Argument**: CLI accepts only one `--docs-path`
2. **Single Watcher Instance**: One `DocumentWatcher` per process
3. **Single Collection**: All documents go to one Qdrant collection
4. **No Path Multiplexing**: No internal support for multiple directory sources

## Workaround Solutions

### Option 1: Multiple Process Instances
Run separate doc-indexer processes for each directory:

```bash
# Terminal 1 - Main docs
./target/release/doc-indexer --docs-path ./docs \
  --collection-name main_docs \
  --api-port 8081

# Terminal 2 - API documentation  
./target/release/doc-indexer --docs-path ./api-docs \
  --collection-name api_docs \
  --api-port 8082

# Terminal 3 - Tutorial content
./target/release/doc-indexer --docs-path ./tutorials \
  --collection-name tutorial_docs \
  --api-port 8083
```

**Pros:**
- ✅ Immediate solution with current code
- ✅ Separate collections for different content types
- ✅ Independent monitoring and indexing
- ✅ Fault isolation (one failure doesn't affect others)

**Cons:**
- ❌ Resource overhead (multiple processes)
- ❌ Multiple API endpoints to manage
- ❌ No unified search across all content

### Option 2: Symbolic Link Aggregation
Create a unified directory with symbolic links:

```bash
mkdir combined-docs
ln -s /absolute/path/to/docs combined-docs/main-docs
ln -s /absolute/path/to/api-docs combined-docs/api-docs  
ln -s /absolute/path/to/tutorials combined-docs/tutorials

./target/release/doc-indexer --docs-path ./combined-docs
```

**Pros:**
- ✅ Single process and API endpoint
- ✅ Unified search across all content
- ✅ Works with current implementation
- ✅ Maintains directory structure context

**Cons:**
- ❌ Platform-dependent (symlinks)
- ❌ Requires manual setup
- ❌ Path resolution complexity in search results

## Future Enhancement Possibilities

### Architecture Changes Needed

**1. CLI Enhancement:**
```rust
struct Cli {
    /// Paths to directories to monitor (can specify multiple)
    #[arg(long = "docs-path", value_delimiter = ',')]
    docs_paths: Vec<PathBuf>,
    
    // OR support multiple instances
    #[arg(long, action = clap::ArgAction::Append)]
    docs_path: Vec<PathBuf>,
}
```

**2. Multi-Watcher Support:**
```rust
pub struct MultiDirectoryWatcher {
    watchers: Vec<DocumentWatcher>,
    path_mappings: HashMap<PathBuf, String>, // path -> collection mapping
}
```

**3. Collection Strategy:**
- **Single Collection**: All content in one collection with path metadata
- **Multiple Collections**: Separate collections per directory
- **Hybrid**: User-configurable collection assignment

### Implementation Complexity

**Medium Effort Required:**
- ✅ CLI changes are straightforward  
- ✅ Multiple watcher instantiation is manageable
- ⚠️ Event routing and path management needs care
- ⚠️ Collection strategy decisions impact search UX
- ⚠️ Configuration complexity increases

## Recommendation

### Immediate Use (Current Implementation)
**Use Option 2: Symbolic Link Aggregation**
- Most practical for current needs
- Single API endpoint 
- Unified search experience
- Minimal complexity

### Future Development Priority
**Medium Priority Enhancement**
- Not critical for Phase 2A completion
- Good candidate for Phase 3 or later
- Would benefit from user feedback on preferred collection strategy
- Could be part of broader configuration management improvements

## Related Files Referenced

- `services/doc-indexer/src/main.rs` (CLI definition)
- `services/doc-indexer/src/watcher_v2.rs` (DocumentWatcher implementation)
- `services/doc-indexer/src/indexer.rs` (DocumentIndexer integration)

## Build Information

- **Release Binary**: `./target/release/doc-indexer`
- **Last Built**: August 20, 2025, 21:04
- **Binary Size**: 16.6 MB (optimized release)
- **Current Version**: Phase 2A complete

---

**Summary**: doc-indexer currently supports single directory monitoring only. Symbolic link aggregation provides the best immediate workaround, while native multi-directory support would be a valuable future enhancement with medium implementation complexity.
