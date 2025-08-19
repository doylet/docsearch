# 0) Ground rules (foundations you’ll want right now)
- **Content IDs & versioning**
  - `doc_id`: stable per file (hash of absolute path).
  - `rev_id`: content hash (e.g., xxHash of bytes). Upsert only when `rev_id` changes.
- **Event coalescing**
  - Debounce FS events (e.g., 250–500ms) and coalesce per file to avoid thrash.
- **Delete semantics**
  - On file delete/rename → tombstone old `doc_id` and remove all its chunks.
- **Schema stability**
  - Fix your Qdrant payload schema before scale (see §2). Add a `schema_version`.

---

# 1) Qdrant integration (Rust)
**Goal:** production-ready upsert/query/delete with batch handling and backpressure.

## Collection design
- **Collection name:** `md_corpus_v1`
- **Vectors:** 768 or 1024 (match your embedding model)
- **Distance:** `Cosine`
- **Sharding:** default; you can tune later
- **Quantization (optional, later):** Scalar/IvfpQ for memory savings

## Payload (per chunk/point)
```json
{
  "doc_id": "sha256(path)",
  "chunk_id": "doc_id:00042",
  "rev_id": "xxhash(content)",
  "rel_path": "notes/design.md",
  "abs_path": "/Users/you/notes/design.md",
  "title": "Design Notes",
  "h_path": ["# Design Notes", "## Architecture"],
  "start_byte": 12345,
  "end_byte": 14567,
  "chunk_index": 42,
  "chunk_total": 87,
  "created_at": "2025-08-19T10:10:10Z",
  "updated_at": "2025-08-19T10:10:10Z",
  "tags": ["markdown", "codeblock?"],
  "emb_model": "gte-small",
  "schema_version": 1
}
```

## Rust crate pointers
- `qdrant-client` (gRPC): create collection (if missing), `upsert_points`, `search_points`, `delete_points`.
- **Batching:** 64–256 points per upsert; exponential backoff on 429/timeout.
- **Backpressure:** a small bounded channel between indexer → embedder → qdrant writer.

**Acceptance criteria**
- Creates `md_corpus_v1` if missing (idempotent).
- Upsert latency (p50) < 1s for a 1k-chunk file (after warm-up).
- Delete removes all `doc_id` points within 500ms of FS event.

---

# 2) Advanced chunking (Markdown-aware)
**Goal:** high-quality retrieval with predictable chunking.

## Strategy (compose in this order)
1. **Structural cuts by headings**
   - Split on `#`, `##`, `###` first; carry `h_path` breadcrumb.
2. **Semantic/size normalization**
   - Target **600–900 tokens** per chunk (or ~2.5–3.5k chars) with **~15% overlap**.
3. **Code blocks & tables**
   - Keep fenced code blocks and tables **intact** (don’t split inside fences).
4. **Inline artifacts**
   - Strip liquid/noise (`<script>`, HTML comments), normalize whitespace.
5. **Metadata injection**
   - Prepend a light header per chunk (not stored, only for embedding):
     ```
     [Title: Design Notes]
     [Path: notes/design.md]
     [Section: # Design Notes > ## Architecture]
     ---
     <chunk text here>
     ```

## Config knobs
```toml
[chunking]
target_tokens = 800
overlap_tokens = 120
max_tokens = 1000
keep_code_fences = true
keep_tables = true
```

**Acceptance criteria**
- Chunker never splits inside a fenced code block.
- Section breadcrumbs (`h_path`) present for every chunk.
- Reprocessing a file with no content change produces **identical** chunk_id sequence.

---

# 3) Embeddings pipeline (localhost API)
**Goal:** reliable local embedding calls with clear contract and rate control.

## Request/response contract
**POST** `/api/embed`
```json
{
  "model": "gte-small",
  "input": ["text 1", "text 2", "..."],
  "truncate": "END"  // or "NONE" (reject), "START"
}
```
**200 OK**
```json
{
  "model": "gte-small",
  "dimension": 768,
  "data": [
    {"index":0, "embedding":[...]},
    {"index":1, "embedding":[...]}
  ],
  "usage": {"input_tokens": 1234}
}
```
**4xx/5xx**
```json
{"error": {"type":"rate_limit","message":"...","retry_after_ms":200}}
```

## Client behavior
- **Batch size:** 16–64 chunks per call (tune).
- **Rate limit:** token-aware or fixed QPS; honor `retry_after_ms`.
- **Determinism:** same input → same vector (model/version pinned).

**Acceptance criteria**
- Throughput ≥ 200 chunks/min on M-series Mac for 768-dim model (baseline).
- All failures are retried with jittered backoff; no data loss, no duplicate points.

---

# 4) Search API (HTTP + stdio JSON-RPC)
**Goal:** a clean, stable contract for clients & CLI.

## HTTP (REST-ish)
- `POST /api/search`
  ```json
  {"query":"vector databases rust","k":10,"filters":{"path_prefix":"notes/","tag":"markdown"}}
  ```
  **200**
  ```json
  {
    "query":"...",
    "results":[
      {
        "doc_id":"...",
        "rel_path":"notes/design.md",
        "h_path":["#","##"],
        "chunk_id":"...:00042",
        "score":0.78,
        "snippet":"...<em>vector</em>...",
        "start_byte":12345,
        "end_byte":14567
      }
    ]
  }
  ```

- `GET /api/docs/{doc_id}` → doc metadata + chunk map
- `DELETE /api/docs/{doc_id}` → purge
- `POST /api/reindex` → (optional) reindex a path/prefix

## JSON-RPC (stdio) methods
```json
{"jsonrpc":"2.0","id":"1","method":"search","params":{"query":"...", "k":10}}
{"jsonrpc":"2.0","id":"2","method":"doc.meta","params":{"doc_id":"..."}}
{"jsonrpc":"2.0","id":"3","method":"doc.purge","params":{"doc_id":"..."}}
{"jsonrpc":"2.0","id":"4","method":"health.check","params":{}}
```

**Acceptance criteria**
- P95 search latency < 200ms for `k=10` on a 100k-chunk corpus (warm cache).
- Identical results via HTTP and JSON-RPC for same params.

---

# 5) CLI UX
**Goal:** fast local dev ergonomics and easy scripting.

### Commands
```bash
mdx index /path/to/folder               # one-shot index + watch
mdx watch                               # start daemon (if not already)
mdx search "retrieval augmented" -k 15   --path-prefix notes/                    --json                                # machine-friendly
mdx doc show <doc_id>                   # show metadata + TOC
mdx purge <doc_id>
mdx stats                               # collection/doc/chunk counts
```

**Acceptance criteria**
- `mdx search` returns highlighted snippets and path+section.
- `--json` mirrors HTTP schema exactly.

---

# 6) Retrieval quality & regression tests
**Goal:** ensure chunking/embeddings changes don’t silently degrade search.

### Minimal harness
- **Judged set:** 50–100 (query, gold doc/chunk) pairs in a CSV.
- **Metric:** Recall@k (k=5/10) and mAP.
- **CI check:** fail PR if Recall@10 drops by >2% absolute from baseline.

### Snippet scoring (optional)
- Use a simple BM25/fusion to reorder top-K vectors by textual relevance for better snippets.

**Acceptance criteria**
- Baseline established and checked in (`/eval/baseline.json`).
- CI job runs on every PR; report a small HTML summary.

---

# 7) Observability & ops
**Goal:** diagnose indexing/search quickly.

- **Structured logs:** JSON logs for events (`index_start`, `embed_batch`, `upsert`, `search`, `delete`) with timings and counts.
- **/api/health:** returns component statuses (fs_watcher, embedder, qdrant, queue_depths).
- **Metrics (Prometheus):**
  - `index_batches_total`, `embed_qps`, `upsert_latency_ms`, `search_latency_ms`, `queue_depth{stage}`.
- **Profiling:** add a feature flag to dump per-stage timings for a single doc.

**Acceptance criteria**
- `mdx stats` aggregates collection size, avg chunk len, orphaned points=0.

---

# 8) Security & safety (local-first)
- **Local only by default:** bind API to `127.0.0.1`, explicit `--listen 0.0.0.0` required.
- **Path allowlist:** index only under configured roots; ignore symlinks out.
- **Ignore rules:** `.gitignore`-style patterns (`node_modules`, `.git`, `*.lock`).
- **Payload scrubbing:** optionally hash emails or secrets via regex before embedding.

---

# 9) Order of execution (2–3 week slice)
1. **Qdrant client + collection bootstrap** (idempotent, schema locked)
2. **Chunker v1 (heading-aware + fences + overlap)** + determinism tests
3. **Embed client (batch + retries)** → end-to-end upsert
4. **Search API (HTTP + JSON-RPC) with kNN** (+ simple snippet)
5. **CLI search + stats**
6. **Eval harness (Recall@k) + CI**
7. **Observability (logs/metrics/health)**

---

# 10) Gotchas & tips
- **Duplicates:** always upsert by `chunk_id`; use `doc_id:NNNNN` so replacing a doc is a stable overwrite, not growth.
- **UTF-8 boundaries:** when chunking by bytes for offsets, ensure you split on char boundaries to avoid invalid slices.
- **Model drift:** pin embedding model + version in payload; re-index if you swap models/dimensions.
- **Cold start:** if the embedder is a local model, pre-warm on first run with a dummy batch.
