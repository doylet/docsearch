The **core problem** now is scope drift: a daemon that indexes/searches has grown into an agent host that also wants to talk to the MCP broker. That’s fixable by putting hard boundaries back in place and letting a **thin agent host** call into a **robust retrieval service**.

Below is a focused architecture and an actionable sprint plan to get you there fast.

# Target Architecture (what “good” looks like)

* **Three bounded contexts**:

1. **Ingestion/Indexing** (write path): watch extract chunk embed upsert (vector + metadata).
2. **Search/Retrieval** (read path): hybrid retrieval (BM25 + vector) + filters, ranked results.
3. **Agent Host**: exposes *tools* (MCP/JSON-RPC/HTTP) that call the Retrieval API; no indexing logic here.
* **Contracts over concrete**: all cross-context calls go through traits/ports (you already have this patterndouble down).
* **Durability & idempotency**: every stage has a job record + idempotency key; safe retries; crash-resume.
* **Back-pressure & concurrency**: bounded workers per stage; explicit queues; batching where it helps (embedding/upsert).
* **Transport-agnostic adapters**: keep STDIO for local/agent, HTTP for service mode; both implement the same port.
* **Artifacts & versioning**: content-addressed artifacts (sha256) for extracted text/chunks/embeddings; never mutate, only append versions.
* **Observability**: structured logs, metrics, trace spans per stage; health & readiness per service.

---

# Sprint 1 Boundaries & Ports (stop the bleeding)

## Goal

Make “agent host” a **client** of retrieval; isolate write vs read paths behind clear ports.

## Tasks

* Define ports in `zero-latency-core` (or `contracts`):

* `IngestionPort` (enqueue, status), `IndexPort` (upsert/delete), `SearchPort` (query, filters, pagination).
* Ensure `services/doc-indexer` **implements** `IngestionPort` + `IndexPort`; ensure `zero-latency-search` **implements** `SearchPort`.
* In the agent host (or future), **only depend on `SearchPort`**.
* Unify configuration models across crates (one loader/validator).

## Acceptance Criteria

* A minimal “agent” binary (or CLI command) can call only `SearchPort`.
* Indexing code cannot be imported by the agent host crate (compile-time boundary).
* One config source enables all transports (stdio/http) without code changes.

---

# Sprint 2 Durable Ingestion Pipeline (jobs + idempotency)

## Goal

Crash-safe, retry-safe indexing with explicit job control.

## Tasks

* Introduce a small job store (SQLite or sled) with states: `queued running done/failed` and an `idempotency_key` (content\_hash + collection + stage).
* Stages: `discover extract chunk embed upsert`. Each persists inputs/outputs & timing.
* Background workers with bounded concurrency; exponential backoff on transient failures.
* CLI: `reindex <path|collection>`, `status <job_id>`, `retry <job_id>`.

## Acceptance Criteria

* Kill the process mid-index; restart resumes from last successful stage.
* Re-submitting the same input (same idempotency key) does not duplicate work.
* `make smoke`: index a sample folder, then search it successfully.

---

# Sprint 3 Hybrid Retrieval & Query Planner (read path quality)

## Goal

Better results and predictable performance for the agent host.

## Tasks

* Implement hybrid retrieval: BM25 (tantivy) + vector (Qdrant/memory), then **merge-rank**.
* Query planner: if filters present push down to metadata store first, then vector search; else run both and fuse.
* Caching hot queries (LRU) + result pagination with stable cursors.
* Add `score_breakdown` (bm25\_score, vector\_score, fused\_score) for transparency.

## Acceptance Criteria

* Benchmarks show consistent latency under concurrency for common queries.
* Filters+hybrid produce better NDCG\@10 on a small labeled set (include a mini eval).
* API returns fused results with per-source scores.

---

# Sprint 4 Transport & Integration (MCP/STDIO/HTTP)

## Goal

Clean integration with your MCP broker without port chaos.

## Tasks

* Keep **STDIO** transport for local/agent tools; keep **HTTP** service for remote calls; both implement `SearchPort`.
* Add `/healthz` and `/readyz` to service mode; `readyz` checks vector store connectivity and warm cache.
* Provide a thin MCP tool wrapper exposing `search`, `similar`, `summarize` that call `SearchPort`.
* Add a tiny **supervisor** mode (optional) to spawn the retrieval service and publish a readiness line (helps your broker supervise).

## Acceptance Criteria

* MCP broker can call the retrieval tool over stdio locally; or http in service mode**no code changes**, only config.
* Readiness gates prevent routing until vector store is reachable.
* One “flip” in config toggles stdio http.

---

# Sprint 5 Observability & SRE hooks

## Goal

You can see what’s happening and why.

## Tasks

* Structured logs (JSON) everywhere with `trace_id`/`job_id`.
* Metrics: per-stage durations, queue depth, retries, error rates, vector latency, BM25 latency, cache hit rate.
* Traces: a span per stage (ingest) and per sub-query (search) with attributes (collection, counts).
* Health detail endpoint: exposes queue size and dependency checks.

## Acceptance Criteria

* A single test run shows correlated spans from `discover` `upsert`.
* A synthetic slow embedding provider shows retries/backoff in metrics and logs.
* Health endpoint is good enough for a broker or k8s `readinessProbe`.

---

# Sprint 6 Extensibility & Safety Rails

## Goal

Add providers safely without code churn; protect the system against overload.

## Tasks

* Provider registry: embeddings (`openai`, `local`), storage (`qdrant`, `memory`), extractors (pdf/docx/html)load from config by key.
* Hard **rate limits** and **batching** for embedding providers; per-provider QoS.
* Content-addressed artifacts with SHA-256; store manifests (chunk hashes, dims, provider, ts).
* Back-pressure: if queue too deep or provider RPS exceeded, slow intake and signal 429 to the agent host.

## Acceptance Criteria

* Swapping providers is config-only; contracts enforced via traits.
* Overload tests don’t crash or spiral retries; the system sheds load gracefully.
* Manifest shows stable hashes; reindexing the same content yields zero net changes.

---

## Nice-to-have (later)

* **Reranking** (small cross-encoder) as an optional final step.
* **Online evaluation**: log user clicks as weak labels to tune fusion weights.
* **Collections & tenancy**: per-collection limits, quotas, and isolation.

---

## TL;DR (how this mitigates your pains)

* **Port headaches** vanish: STDIO by default, HTTP when you need it, all config-driven.
* **Manual lifecycle** goes away: supervisor/readyz + a single “dev up” command.
* **Scope creep** is contained: the agent host is thin and calls a dedicated Retrieval API; indexing is a durable pipeline behind a queue.
* **Reliability** jumps: idempotency, retries, back-pressure, and bounded concurrency make the system predictable.

If you’d like, I can generate a minimal patch set for Sprint 1 (ports/interfaces + config unification) tailored to your existing crates, and a template `jobs` table for the SQLite queue.