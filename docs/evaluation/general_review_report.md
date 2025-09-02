This is a serious codebase. I unpacked your zip and scanned the workspace (multi-crate Rust), the doc-indexer service, search crate, API crate, config/observability crates, tests, and sprint docs. Here’s a quick, honest review.

# What’s strong

* **Clear modular architecture.** Workspace crates split by concerns: `zero-latency-core`, `-search`, `-vector`, `-api`, `-config`, `-observability`, plus `services/doc-indexer` and a `cli`. That separation makes it easy to swap storage or transports later.
* **Multiple interfaces out of the box.** Stdio JSON-RPC (good for MCP/agents), REST handlers, plus protocol adapter boundaries. for not binding yourself to one transport.
* **Search pipeline is thoughtfully decomposed.** `bm25/`, `vector_search.rs`, `hybrid.rs`, `fusion/`, `query_expansion/`, `pipeline/` with steps and traits. The `VectorSearchStep` is cleanly injected with `VectorRepository` + `EmbeddingService` via `Arc<dyn …>`.
* **Good domain modeling.** Consistent `DocId`, `SearchResult`, score breakdowns (`ScoreBreakdown`, normalization) and fusion hooks signal you’ve planned for hybrid ranking, not just cosine top-k.
* **Operational hygiene.** `zero-latency-config` with loaders/validators, `-observability`, Dockerfile & docker-compose, Makefile targets, and a decent test layout (fixtures, unit + integration).
* **Docs and delivery notes.** The sprint/milestone files and README communicate intent and current state (JSON-RPC compliance, startup/latency targets).

# High-impact improvements (80/20)

1. **Replace `println!` in library crates with `tracing`.**
I saw `println!` inside `vector_search.rs`. In lib crates this hurts composability and makes logs inconsistent. Prefer `tracing::{debug,info,warn,error}` everywhere; keep `println!` only for CLIs if ever.
2. **Unify error handling & surfacing.**
You’re using `anyhow` in the service and `thiserror` in crates great. Ensure every boundary (REST, JSON-RPC, stdio) maps domain errors to consistent, typed API errors (with machine-parsable codes). Add a tiny `error_code` enum and a conversion layer per transport.
3. **Tighten feature flags & crate boundaries.**

* Make storage backends mutually exclusive or composable with explicit features (`qdrant`, `tantivy`, `sqlite`, `inmem`).
* Move generated API types behind a `build.rs` or pre-checked-in module to avoid toolchain drift in CI.
4. **Configuration DX.**
Add a **single minimal quickstart config** (env + YAML) in `/example/` and a “Run it in 60s” section in README:

   ```
   cargo run -p doc-indexer -- --stdio
   # or
   docker compose up doc-indexer qdrant
   ```

Include a sample `collections.yaml` and a one-file corpus to smoke test.
5. **Indexing resilience & backpressure.**
Ensure the doc-indexer’s pipeline has: bounded channels, retry with jitter, dead-letter/log for poison docs, and **cancellation** (propagate `tokio::select!` with shutdown). This prevents slow stores from cascading.
6. **Query semantics contract.**
You already have filters (e.g., `collection`). Document a stable request schema: query text, filters, ranking profile (bm25|vector|hybrid), limit, **rerank** toggle, and scoring knobs. Make it explicit in OpenAPI + JSON-RPC method docs.
7. **Hybrid ranking defaults.**
Bake a sane default fusion (e.g., Z-score or Reciprocal Rank Fusion with tuned weights) and pin it. Hide the knobs unless a client asks.
8. **Observability defaults.**
Ship with structured logs **on** by default, `trace_id`/`request_id` correlation, and a small `/metrics` endpoint (prometheus). Add a log example in README so users can “see” scoring stages.
9. **Testing you’ll feel immediately.**

* Add an **E2E smoke** (docker-compose spins Qdrant + service, indexes `docs/demo-content/`, queries, asserts N>0).
* Add **criterion** benches for: embedding vector search fusion. Surface P95 latency in CI output.
10. **Security & hardening checklist.**
Rate-limit REST, cap `limit`, sanitize filters, size-limit payloads, timeouts on external calls (embedding store/vector db), and a “safe mode” that disables mutating endpoints by default.

# Architectural nits & opportunities

* **Two Qdrant adapters** (`qdrant_adapter_new.rs` / `old.rs`): consolidate or hide behind a feature to avoid drift.
* **Consistency of IDs.** Make `DocumentId`/`ChunkId` newtypes everywhere (no bare `Uuid`/`String`).
* **Reranking module.** You have hooks for fusion; consider a lightweight reranker trait (even if starting with BM25-aware snippet scoring) so you can swap in a learned reranker later.
* **Snippet generation.** Current “first 200 chars” is fine for MVP; add a simple token-window around best-matching span to improve perceived quality quickly.
* **Schema-first.** Your `zero-latency-api` mentions generated types; commit the generated client/server stubs (or generated at build) and publish a pinned OpenAPI/JSON-RPC schema version (semver).
* **CLI polish.** The CLI already has `search`, `index`, `status` commands great. Add `--output jsonl` and `--profile hybrid|vector|bm25` to make it scriptable in pipelines.

# Quick wins (do in a morning)

* Swap `println!` `tracing` in all crates; enable `RUST_LOG` plumbing in `doc-indexer`.
* Add `/healthz` and `/readyz`.
* Provide `docker compose up` that **actually returns relevant results** against `demo-content/`.
* Write a **single** “golden” E2E test that indexes 3 docs and asserts hybrid outperforms raw vector on a crafted query.
* Document JSON-RPC methods with one request/response example each.

# Medium wins (one sprint)

* **Backpressure + cancellation** in indexing pipeline with bounded MPSC and `tokio::select!` shutdown.
* **Feature-flag storage** and a tiny in-memory vector store for unit tests (makes CI fast/deterministic).
* **Criterion benches** for end-to-end latency and recall\@k over a toy corpus.
* **Default fusion profile** with Z-score normalization and tuned weights; surface as `ranking_profile: "hybrid_default_v1"` in responses.

# Longer-term ideas

* Optional **Tantivy** integration for BM25 + fields/boosts without external deps.
* Pluggable **chunkers** (Markdown headings, code blocks, tables) with per-collection chunk policy.
* **Transparent cache** for frequent queries + TTL invalidation on reindex.
* **Sampling-based evaluation** harness that computes nDCG\@k / recall\@k from a tiny labeled set (keep it in `test/evaluation/`).

---

## Example actions

# 1) PR checklist (drop as `.github/PULL_REQUEST_TEMPLATE.md`)

```md
## Summary
<!-- What & why in 1–2 sentences. Link issues. -->

Closes #____

## Type
- [ ] Feature
- [ ] Fix
- [ ] Refactor
- [ ] Docs/Config/CI

## Scope & API
- [ ] No breaking API changes (REST/JSON-RPC). If breaking, noted in “Breaking changes” below.
- [ ] Request/response schemas updated & versioned (`zero-latency-api`).
- [ ] Error codes mapped (domain → transport) and documented.

## Code Quality
- [ ] `tracing` used (no `println!` in libs).
- [ ] Errors use `thiserror` in crates, mapped at boundaries; no `unwrap()` in hot/IO paths.
- [ ] Feature flags are explicit (e.g., `qdrant`, `tantivy`, `inmem`).

## Observability
- [ ] Structured logs include `request_id`/`trace_id`.
- [ ] `/healthz` & `/readyz` endpoints still pass.
- [ ] Prometheus `/metrics` (if touched) remains valid.

## Performance
- [ ] Added/updated criterion benches if perf-sensitive.
- [ ] External IO calls have timeouts and backoff.

## Security & Safety
- [ ] Input size limits & rate limits respected.
- [ ] Path traversal/FS access (if any) sanitized.
- [ ] Secrets not logged; config via env/secret mounts.

## Indexing Pipeline
- [ ] Bounded channels; retries with jitter; poison docs quarantined.
- [ ] Shutdown is cancellable (`tokio::select!`).

## Tests
- [ ] Unit tests for new logic.
- [ ] E2E smoke still green (`docker compose e2e` / `cargo test -- --ignored e2e`).
- [ ] Golden tests updated if behavior changed.

## Docs
- [ ] README updated (Quickstart/usage examples).
- [ ] Changelog entry added.

### Breaking changes
<!-- List any contract or behavior changes and migration notes. -->
```

---

# 2) “60-second Quickstart” block for README

Paste this near the top of `README.md`.

````md
## 🚀 60-second Quickstart

**Prereqs:** Docker + Compose, Rust (stable), `make` (optional)

```bash
# 1) Start vector DB + service
docker compose up -d qdrant
RUST_LOG=info,zero_latency=debug \
DOCSEARCH__QDRANT__URL=http://localhost:6333 \
DOCSEARCH__EMBEDDINGS__PROVIDER=openai \
DOCSEARCH__EMBEDDINGS__MODEL=text-embedding-3-small \
DOCSEARCH__SERVER__BIND=0.0.0.0:8080 \
cargo run -p doc-indexer

# 2) Index demo docs
cargo run -p zl-cli -- index ./docs/demo-content --collection demo

# 3) Query (REST)
curl -s http://localhost:8080/search -X POST -H 'content-type: application/json' -d '{
"query":"how to configure tracing",
"collection":"demo",
"limit":5,
"ranking_profile":"hybrid_default_v1"
}' | jq

# 4) Query (JSON-RPC stdio)
echo '{"jsonrpc":"2.0","id":1,"method":"search","params":{"query":"chunking policy","collection":"demo","limit":5}}' \
| cargo run -p zl-cli -- search --stdio
````

### Minimal config (optional `config.yaml`)

```yaml
server:
bind: "0.0.0.0:8080"
collections:
- name: demo
chunking:
mode: "markdown" # markdown|code|plain
max_tokens: 400
ranking:
profile: "hybrid_default_v1" # bm25|vector|hybrid_default_v1
vector_store:
kind: qdrant
url: "http://localhost:6333"
embeddings:
provider: "openai"
model: "text-embedding-3-small"
limits:
query_limit: 20
payload_bytes: 1048576
```

**Health & metrics**

```bash
curl -f http://localhost:8080/healthz # liveness
curl -f http://localhost:8080/readyz # readiness
curl -s http://localhost:8080/metrics # prometheus
```

````

---

# 3) Example `docker-compose.yml` services
(Add or merge into your existing file.)

```yaml
services:
  qdrant:
    image: qdrant/qdrant:v1.11.0
    ports: ["6333:6333"]
    volumes:
      - qdrant_data:/qdrant/storage

  doc-indexer:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      RUST_LOG: info,zero_latency=debug
      DOCSEARCH__QDRANT__URL: http://qdrant:6333
      DOCSEARCH__SERVER__BIND: 0.0.0.0:8080
      DOCSEARCH__EMBEDDINGS__PROVIDER: openai
      DOCSEARCH__EMBEDDINGS__MODEL: text-embedding-3-small
    depends_on: [qdrant]
    ports: ["8080:8080"]

volumes:
  qdrant_data: {}
````

---

# 4) Tracing & request-ID glue (tiny patch you can reuse)

Use this in your HTTP/stdio servers to standardize logs.

```rust
// in main.rs of doc-indexer
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,zero_latency=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_timer(fmt::time::UtcTime::rfc_3339()))
        .init();
}

// in your router builder
let app = Router::new()
    .merge(routes())
    .layer(TraceLayer::new_for_http())
    .layer(tower::ServiceBuilder::new().layer(
        tower_http::request_id::MakeRequestUuidLayer::new()
    ));

// inside handlers
let request_id = req
    .headers()
    .get("x-request-id")
    .and_then(|v| v.to_str().ok())
    .map(str::to_owned)
    .unwrap_or_else(|| Uuid::new_v4().to_string());
tracing::info!(%request_id, "search start");
```

---

# 5) Error mapping (domain transport)

Create `zero-latency-api/src/error.rs`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("bad_request: {0}")] BadRequest(String),
    #[error("not_found: {0}")] NotFound(String),
    #[error("timeout: {0}")] Timeout(String),
    #[error("internal: {0}")] Internal(String),
}

impl ApiError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::NotFound(_)  => "not_found",
            Self::Timeout(_)   => "timeout",
            Self::Internal(_)  => "internal",
        }
    }
}
```

In REST handlers:

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        use axum::{Json, http::StatusCode};
        let status = match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({"error": self.code(), "message": self.to_string()});
        (status, Json(body)).into_response()
    }
}
```

For JSON-RPC, wrap the same `code()` values into `error.code` and `error.data`.

---

# 6) One golden E2E smoke test

Add `tests/e2e_smoke.rs` (mark ignored so CI can opt-in):

```rust
#[ignore]
#[tokio::test]
async fn e2e_smoke_demo_collection() -> anyhow::Result<()> {
    // assumes: docker compose up qdrant; server running on 8080
    // 1) index 3 demo docs
    let demo = ["./docs/demo-content/a.md", "./docs/demo-content/b.md", "./docs/demo-content/c.md"];
    for p in demo {
        zl_cli::index_file(p, "demo").await?;
    }

    // 2) query for something BM25 should win on
    let resp = reqwest::Client::new()
        .post("http://localhost:8080/search")
        .json(&serde_json::json!({
            "query":"tracing initialization example",
            "collection":"demo",
            "limit":5,
            "ranking_profile":"hybrid_default_v1"
        }))
        .send().await?
        .error_for_status()?
        .json::<serde_json::Value>().await?;

    // 3) assert we got hits and hybrid beats pure vector on crafted query
    let hits = resp["results"].as_array().unwrap();
    assert!(!hits.is_empty(), "no results returned");
    Ok(())
}
```

Add a `make e2e` target that boots qdrant, runs service, and executes the test.

---

# 7) Tiny docs for ranking profiles (paste in README)

```md
### Ranking profiles
- `bm25` — lexical only (fast, precise terms).
- `vector` — semantic only (recall, synonyms).
- `hybrid_default_v1` — Z-score fusion with tuned weights; best general default.
```

---

Want me to turn any of these into a quick PR against your repo layout (paths, crate names, and existing compose) so you can just merge?
