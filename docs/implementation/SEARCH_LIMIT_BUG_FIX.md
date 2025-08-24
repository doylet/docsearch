# Search Limit Bug Fix Implementation

## Problem Description

The CLI search command's `--limit` parameter was being ignored, causing searches to always return 10 results regardless of the user-specified limit. This also led to duplicate results being displayed.

### Example of the Bug
```bash
mdx search "architecture" --limit 3
# Expected: 3 results
# Actual: 10 results with duplicates
```

## Root Cause Analysis

The search limit parameter was not properly flowing through the entire request pipeline:

1. ✅ **CLI Layer**: Correctly captured `--limit` argument
2. ❌ **Domain Model**: `SearchQuery` lacked limit support  
3. ❌ **Service Layer**: CLI service wasn't passing limit to domain model
4. ❌ **Infrastructure**: API client hardcoded `limit: 10`

## Solution Implementation

### 1. Enhanced SearchQuery Domain Model

**File**: `crates/zero-latency-core/src/values.rs`

```rust
// Added limit field to SearchQuery
pub struct SearchQuery {
    pub raw: String,
    pub normalized: String,
    pub enhanced: Option<String>,
    pub limit: u32,  // NEW: Default limit support
}

impl SearchQuery {
    pub fn new(raw: impl Into<String>) -> Self {
        // ... existing code ...
        Self {
            raw,
            normalized,
            enhanced: None,
            limit: 10,  // Default limit
        }
    }

    // NEW: Builder method for setting limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }
}
```

### 2. Updated CLI Service Layer

**File**: `crates/cli/src/application/services/cli_service.rs`

```rust
pub async fn search(&self, request: SearchCommand) -> ZeroLatencyResult<()> {
    // FIXED: Now passes limit to domain model
    let search_query = SearchQuery::new(request.query).with_limit(request.limit);
    
    let response = self.api_client.search(search_query).await?;
    self.output_formatter.format_search_results(response, &request.format).await?;
    Ok(())
}
```

### 3. Fixed API Client Infrastructure

**File**: `crates/cli/src/infrastructure/http/api_client.rs`

```rust
pub async fn search(&self, query: SearchQuery) -> ZeroLatencyResult<SearchResponse> {
    let response = self.client
        .post(&url)
        .json(&serde_json::json!({
            "query": query.effective_query(),
            "limit": query.limit,  // FIXED: Use query limit instead of hardcoded 10
            "collection_name": self.collection_name
        }))
        // ... rest of method
}
```

## Request Flow After Fix

```
CLI Command: mdx search "query" --limit 3
    ↓
SearchCommand { limit: 3 }
    ↓
SearchQuery::new("query").with_limit(3)
    ↓
API Request: {"query": "query", "limit": 3}
    ↓
VectorSearchStep: searches with limit 3
    ↓
Result: Exactly 3 search results
```

## Verification

### Direct API Test
```bash
curl -X POST http://localhost:8081/api/search \
  -H "Content-Type: application/json" \
  -d '{"query": "test", "limit": 3}'
```

**Server Logs:**
```
🔍 VectorSearchStep: Searching vector database with limit 3
📊 VectorSearchStep: Found 3 vector results
```

### CLI Test
```bash
mdx search "architecture" --limit 3
# Now correctly returns exactly 3 results
```

## Files Modified

- `crates/zero-latency-core/src/values.rs`: Enhanced SearchQuery domain model
- `crates/cli/src/application/services/cli_service.rs`: Updated service to pass limit
- `crates/cli/src/infrastructure/http/api_client.rs`: Fixed hardcoded limit bug
- `services/doc-indexer/src/infrastructure/http/handlers.rs`: Temporary debug logging (removed)

## Impact

- ✅ Search limit parameter now properly respected across entire pipeline
- ✅ Eliminates duplicate results in search output
- ✅ Improves user experience and search precision
- ✅ Maintains backward compatibility (default limit remains 10)

## Testing

The fix has been verified through:
1. Direct API calls with various limit values
2. Vector search step logging showing correct limits
3. Result count validation matching requested limits

This resolves the search limit parameter bug while maintaining clean architecture principles and proper separation of concerns.
