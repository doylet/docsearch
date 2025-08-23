# 032 - Native Search Integration Plan: Raycast, Spotlight & LaunchBar Integration

**Date:** August 21, 2025  
**Status:** ✅ COMPLETE  
**Context:** Alternative UI exploration for Zero-Latency documentation search  
**Strategic Analysis:** Implementation roadmap for native OS integration  
**Related:** [031](031_phase-4-strategic-roadmap-analysis.md), [033](033_phase-4b-ml-ai-implementation-plan.md), [034](034_phase-4a-frontend-ui-plans.md)  

## 🎯 **Vision: Native OS Search Integration**

Instead of traditional web interfaces, integrate Zero-Latency documentation search directly into native macOS/Windows/Linux search experiences, providing instant access through familiar OS-level search interfaces.

## 🔍 **Native Search Landscape Analysis**

### **macOS Spotlight Integration**

#### **Spotlight Importer Plugin Approach**
```objc
// Create .mdimporter bundle for documentation files
// ZeroLatencyDocumentationImporter.mdimporter/

@interface ZLDocumentationImporter : NSObject <MDImporter>
- (BOOL)importDocument:(MDDocument *)document 
                contentType:(NSString *)contentType 
                  attributes:(NSDictionary *)attributes 
                       error:(NSError **)error;
@end

// Index metadata for Spotlight consumption
[document setAttributes:@{
    (NSString *)kMDItemTitle: documentTitle,
    (NSString *)kMDItemTextContent: processedContent,
    (NSString *)kMDItemKeywords: searchKeywords,
    @"com_zerolat_semantic_score": @(semanticScore),
    @"com_zerolat_document_path": headingPath
}];
```

**Implementation Strategy:**
1. **MDImporter Plugin:** Custom Spotlight importer for documentation files
2. **Background Daemon:** Zero-Latency service that updates Spotlight index
3. **Semantic Bridging:** Map vector search results to Spotlight metadata
4. **Quick Look Integration:** Rich previews with semantic highlighting

#### **Spotlight Search Enhancement**
```bash
# Enhanced Spotlight queries
mdfind "kMDItemTextContent == '*vector database*'cd && kMDItemKind == 'Zero-Latency Documentation'"

# Semantic score weighting
mdfind "com_zerolat_semantic_score >= 0.8"
```

### **Raycast Extension Integration**

#### **Raycast Command Architecture**
```typescript
// raycast-zero-latency/src/search-docs.tsx
import { ActionPanel, List, Action } from "@raycast/api";
import { useState, useEffect } from "react";

export default function SearchDocumentation() {
  const [searchText, setSearchText] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  
  // Real-time search with Zero-Latency API
  useEffect(() => {
    if (searchText.length > 2) {
      searchZeroLatency(searchText).then(setResults);
    }
  }, [searchText]);

  return (
    <List onSearchTextChange={setSearchText} throttle>
      {results.map((result) => (
        <List.Item
          key={result.chunk_id}
          title={result.document_title}
          subtitle={result.snippet}
          accessories={[
            { text: `Score: ${result.final_score.toFixed(2)}` },
            { text: result.section }
          ]}
          actions={
            <ActionPanel>
              <Action.OpenInBrowser url={result.url} />
              <Action.CopyToClipboard content={result.content} />
            </ActionPanel>
          }
        />
      ))}
    </List>
  );
}
```

**Raycast Features:**
- **Instant Search:** Sub-100ms responses via local API
- **Rich Results:** Ranking signals, snippets, and metadata
- **Action Integration:** Copy, open, share results
- **Keyboard Navigation:** Native Raycast UX patterns

### **LaunchBar Integration**

#### **LaunchBar Action Bundle**
```applescript
-- ZeroLatencySearch.scpt
on handle_string(query)
    set apiURL to "http://localhost:8081/api/search"
    set jsonPayload to "{\"query\":\"" & query & "\",\"k\":10}"
    
    set curlCommand to "curl -X POST " & apiURL & " -H 'Content-Type: application/json' -d '" & jsonPayload & "'"
    set jsonResult to do shell script curlCommand
    
    -- Parse and format results for LaunchBar
    return parseSearchResults(jsonResult)
end handle_string
```

**LaunchBar Features:**
- **Abbreviation Support:** `zl vector database` → instant search
- **Result Ranking:** Native LaunchBar scoring integration
- **Quick Actions:** Open, copy, send results
- **Browsing Integration:** Send to browser with highlighting

## 🏗️ **Implementation Architecture**

### **Core Service Layer**
```rust
// services/doc-indexer/src/native_integration/mod.rs
pub mod spotlight;
pub mod raycast_api;
pub mod launchbar_bridge;

pub struct NativeIntegrationService {
    search_service: Arc<SearchService>,
    spotlight_indexer: Option<SpotlightIndexer>,
    api_server: Arc<ApiServer>,
}

impl NativeIntegrationService {
    pub async fn update_spotlight_index(&self, doc_id: &str) -> Result<()> {
        // Update Spotlight metadata for document
        let search_results = self.search_service.get_document_chunks(doc_id).await?;
        self.spotlight_indexer?.update_document_metadata(doc_id, search_results).await
    }
    
    pub async fn serve_raycast_api(&self) -> Result<()> {
        // Dedicated Raycast API endpoint with optimized responses
        let raycast_router = Router::new()
            .route("/raycast/search", post(raycast_search_handler))
            .route("/raycast/suggest", post(raycast_suggest_handler));
    }
}
```

### **Cross-Platform Approach**

#### **macOS Implementation**
- **Spotlight MDImporter:** Custom .mdimporter bundle
- **Background LaunchAgent:** plist-based service registration
- **CoreServices Integration:** File type association and Quick Look

#### **Windows Implementation**
- **Windows Search Protocol Handler:** Custom protocol `zldocs://search?q=query`
- **PowerToys Run Plugin:** Native PowerToys integration
- **Windows Search Service:** IFilter implementation for document indexing

#### **Linux Implementation**
- **Albert/Ulauncher Extensions:** Plugin architecture
- **DBus Integration:** System-wide search service
- **Desktop Entry Files:** `.desktop` files for application integration

## 🔧 **Technical Implementation Plan**

### **Phase 1: Local API Optimization (Week 1)**

```rust
// Optimize API for native integration
pub struct NativeSearchRequest {
    pub query: String,
    pub limit: usize,
    pub response_format: ResponseFormat, // Compact, Full, Metadata
    pub context_chars: usize,
}

#[derive(Serialize)]
pub struct CompactSearchResult {
    pub title: String,
    pub snippet: String,
    pub score: f32,
    pub url: String,
    pub breadcrumb: String,
}
```

**Optimizations:**
- **Compact Response Format:** Minimal JSON for native clients
- **Streaming Results:** Server-sent events for real-time search
- **Caching Layer:** In-memory LRU cache for common queries
- **Batch Processing:** Multi-query support for efficiency

### **Phase 2: Spotlight Integration (Week 2)**

```objc
// Spotlight metadata schema
static NSString* const kZLSemanticScore = @"com_zerolat_semantic_score";
static NSString* const kZLDocumentPath = @"com_zerolat_document_path";
static NSString* const kZLChunkID = @"com_zerolat_chunk_id";
static NSString* const kZLLastUpdated = @"com_zerolat_last_updated";

@implementation ZLSpotlightIndexer
- (void)updateDocumentMetadata:(NSString*)docID 
                    withChunks:(NSArray<ZLDocumentChunk*>*)chunks {
    CSSearchableIndex* index = [CSSearchableIndex defaultSearchableIndex];
    NSMutableArray* items = [NSMutableArray array];
    
    for (ZLDocumentChunk* chunk in chunks) {
        CSSearchableItemAttributeSet* attributes = 
            [[CSSearchableItemAttributeSet alloc] 
             initWithItemContentType:(NSString*)kUTTypeText];
        
        [attributes setTitle:chunk.title];
        [attributes setContentDescription:chunk.snippet];
        [attributes setValue:@(chunk.semanticScore) forCustomKey:kZLSemanticScore];
        
        CSSearchableItem* item = [[CSSearchableItem alloc] 
                                 initWithUniqueIdentifier:chunk.chunkID
                                          domainIdentifier:@"com.zerolat.docs"
                                              attributeSet:attributes];
        [items addObject:item];
    }
    
    [index indexSearchableItems:items completionHandler:^(NSError* error) {
        if (error) NSLog(@"Spotlight indexing error: %@", error);
    }];
}
@end
```

### **Phase 3: Raycast Extension (Week 3)**

```typescript
// raycast-extension/src/api/zero-latency.ts
interface ZeroLatencyAPI {
  search(query: string, options?: SearchOptions): Promise<SearchResult[]>;
  suggest(partial: string): Promise<string[]>;
  getDocument(docId: string): Promise<Document>;
}

class ZeroLatencyClient implements ZeroLatencyAPI {
  private baseURL = "http://localhost:8081";
  
  async search(query: string, options: SearchOptions = {}): Promise<SearchResult[]> {
    const response = await fetch(`${this.baseURL}/api/search`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        query,
        k: options.limit || 10,
        response_format: "compact",
        include_snippets: true
      })
    });
    
    if (!response.ok) throw new Error(`Search failed: ${response.statusText}`);
    
    const data = await response.json();
    return data.results.map(this.formatResult);
  }
  
  private formatResult(result: RawSearchResult): SearchResult {
    return {
      id: result.chunk_id,
      title: result.document_title,
      subtitle: result.snippet,
      score: result.final_score,
      url: result.url,
      accessories: [
        { text: `${(result.final_score * 100).toFixed(0)}%` },
        { text: result.section }
      ]
    };
  }
}
```

### **Phase 4: LaunchBar Integration (Week 4)**

```applescript
-- LaunchBar Action: Zero-Latency Search
property apiEndpoint : "http://localhost:8081/api/search"
property maxResults : 10

on handle_string(query)
    if length of query < 3 then
        return {title:"Type at least 3 characters", subtitle:"Zero-Latency Documentation Search"}
    end if
    
    set searchResults to searchZeroLatency(query)
    set formattedResults to {}
    
    repeat with result in searchResults
        set end of formattedResults to {
            title: (title of result),
            subtitle: (snippet of result),
            url: (url of result),
            icon: "DocumentIcon.icns",
            badge: round((score of result) * 100) & "%"
        }
    end repeat
    
    return formattedResults
end handle_string

on searchZeroLatency(query)
    set curlCommand to "curl -s -X POST " & apiEndpoint & " -H 'Content-Type: application/json' -d '{\"query\":\"" & query & "\",\"k\":" & maxResults & ",\"response_format\":\"compact\"}'"
    
    try
        set jsonResponse to do shell script curlCommand
        return parseJSON(jsonResponse)
    on error
        return {{title:"Search service unavailable", subtitle:"Is Zero-Latency running on localhost:8081?"}}
    end try
end searchZeroLatency
```

## 🎯 **User Experience Design**

### **Unified Search Experience**
1. **Query Input:** User types in native search interface
2. **Real-time Results:** Sub-100ms responses from local service
3. **Rich Previews:** Semantic highlighting and ranking signals
4. **Action Integration:** Open, copy, share, save results
5. **Context Awareness:** Remember search history and preferences

### **Search Result Format**
```
📄 Vector Database Configuration Guide
   Configure Qdrant for production deployment with clustering...
   Score: 89% • Getting Started • 2 min read
   
📄 Embedding Model Selection 
   Choose between local and remote embedding models...
   Score: 85% • Architecture • 5 min read
```

## 🚀 **Implementation Benefits**

### **User Experience**
- **Zero Learning Curve:** Uses familiar OS search patterns
- **Instant Access:** No browser required, always available
- **Keyboard-First:** Optimized for power users
- **Context-Aware:** Integrates with existing workflow

### **Technical Advantages**
- **Performance:** Direct API access, no web overhead
- **Reliability:** Local service, no network dependencies
- **Privacy:** All search data stays local
- **Integration:** Works with existing OS features

### **Competitive Advantages**
- **Novel Approach:** First documentation search with native OS integration
- **Developer-Friendly:** Matches developer tool preferences (Raycast, Spotlight)
- **Scalable:** Framework supports multiple native search platforms
- **Extensible:** Plugin architecture for future enhancements

## 📋 **Implementation Checklist**

### **Week 1: Foundation**
- [ ] Optimize search API for native clients
- [ ] Implement compact response formats
- [ ] Add streaming search capabilities
- [ ] Create local service discovery

### **Week 2: macOS Spotlight**
- [ ] Build MDImporter plugin
- [ ] Implement Spotlight metadata indexing
- [ ] Create background update service
- [ ] Test Spotlight search integration

### **Week 3: Raycast Extension**
- [ ] Create Raycast extension scaffold
- [ ] Implement search command
- [ ] Add result actions and navigation
- [ ] Package and test extension

### **Week 4: LaunchBar & Cross-Platform**
- [ ] Build LaunchBar action bundle
- [ ] Implement Windows Search integration
- [ ] Create Linux desktop integration
- [ ] Documentation and distribution

---

## 🎯 **Strategic Value**

This native search integration approach provides:
- **Immediate User Value:** Familiar, fast search experience
- **Competitive Differentiation:** Novel integration approach
- **Developer Adoption:** Matches developer tool preferences  
- **Scalable Architecture:** Framework for future native integrations

**Next Steps:** Begin with Phase 1 API optimization to support all native integrations, then implement platform-specific integrations based on user preferences and adoption patterns.
