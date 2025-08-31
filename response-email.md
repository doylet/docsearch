Here’s a crisp review of your **docsearch** (Rust workspace) and the **ZL-008 sprint plan** you attached.

## What’s already strong

* Clear crate separation (`zero-latency-core/config/search/vector`, `services/doc-indexer`, `cli`).
* Existing **vector** path with enhancement hooks (`traits.rs`, `vector_search.rs`) you can plug a BM25 adapter into.
* Sprint ZL-008 focuses on **hybrid retrieval, MQE, and evaluation**exactly the right bets for quality/latency.&#x20;

---

## Gaps & high-impact fixes (project + plan)

### 1) Document ID & dual-index sync (must-have)

**Why:** Hybrid requires stable, shared IDs across **BM25** and **vector** stores.

* Define a single **`DocId`** (e.g., `{collection, external_id, version}`) and store it in *both* indices.
* Add an **Index Sync contract** (upsert/delete/batch) that dual-writes (vector + BM25) and logs a version/seqno.
* Add **reconciliation**: a background task compares counts & sample IDs; alerts on drift.

**Sprint tweak:** Add an acceptance criterion to ZL-008-004 (“Index sync verified via reconciliation report”).

---

### 2) Score normalization & fusion (be explicit)

Your plan says “normalized\_sum”; define *how*:

* **Normalize**: min-max per list OR z-score + clamp to \[0,1].
* **Fuse**: `fused = w_bm25 * bm25_norm + w_vec * vec_norm` (weights configurable per collection).
* Include both **raw** and **normalized** scores in the response for transparency.

**Code sketch (orchestrator)**

```rust
// run both in parallel and fuse
let (bm25, vecs) = tokio::try_join!(
    bm25_engine.search(&q, k, &filters),
    vector_engine.search(&q, k, &filters),
)?;
let bm25n = normalize(&bm25, |h| h.score);
let vecsn = normalize(&vecs, |h| h.score);
let fused = merge_by_docid(bm25n, vecsn, |b,v| w_bm25*b + w_vec*v);
let ranked = reranker.maybe_rerank(&q, fused, top_k_final)?;
```

**Sprint tweak:** ZL-008-003 should specify the chosen normalization (and add unit tests with known inputs/outputs).&#x20;

---

### 3) Parallelism & time budgets (keep P95 ≤ 350 ms)

* **Parallel** BM25 + vector with `try_join!`; short **per-engine timeouts**.
* If one engine misses its budget, **use the other** and annotate `partial=true` in debug metadata.
* Pre-warm BM25 reader and pool vector clients on boot.

**Sprint tweak:** Add “partial failure behavior documented & tested” to ZL-008-005 acceptance.&#x20;

---

### 4) MQE (multi-query expansion) without latency spikes

* Generate ≤3 paraphrases **concurrently**, but cap total time (e.g., 120150 ms).
* **Union + dedupe** hits before rerank; keep per-variant provenance for debugging.
* Cache paraphrases for 515 min keyed by `(norm_query, filters)`.

**Sprint tweak:** ZL-008-006 add “global MQE latency budget” + cache acceptance. ZL-008-007 add tests for dedupe stability under overlapping variants.&#x20;

---

### 5) Result model: make diagnostics first-class

Extend `SearchResult`:

```rust
pub struct Hit {
  pub doc_id: DocId,
  pub uri: String,
  pub title: String,
  pub snippet: String,
  pub section_path: Vec<String>,
  pub scores: Scores, // raw + normalized + fused
  pub from: FromSignals, // {bm25:bool, vector:bool, variants: Vec<usize>}
}
```

This lets UIs & tests explain “why” an item ranked.

**Sprint tweak:** Add “score breakdown present in API & docs” to ZL-008-005.&#x20;

---

### 6) Caching & invalidation (prevent phantom wins)

* Cache full results by `(norm_query, filters, top_k, features{hybrid,mqe,rerank})`.
* Invalidate by **collection version** (bump on ingest), not time alone.
* Keep a small **rerank cache** (`(doc_id, query_hash) rerank_score`) to cut tail latency.

**Sprint tweak:** In ZL-008-009, include “cache key spec & invalidation contract” + hit-rate metric.&#x20;

---

### 7) Evaluation you can trust (CI-grade)

* Dataset: 50100 labeled queries with graded relevance (0/1/2).
* Metrics: **NDCG\@10**, **Hit\@k**, **P\@k**; print **A/B deltas with CIs**.
* CI gate: “fail if NDCG\@10 drops > 3%” (configurable).
* Include **cost & latency** panels (e.g., tokens, rerank time).

**Sprint tweak:** ZL-008-002 add CI gating step + small “golden” snapshot to catch regressions. ZL-008-008 add “A/B with statistical significance method” (e.g., randomization test).&#x20;

---

### 8) Ops: feature flags & safe rollout

* Flags: `hybrid_search`, `multi_query_expansion`, `rerank`, `result_caching`.
* Roll out by **collection**; default new collections to hybrid, keep legacy on vector-only until benchmarks pass.
* Emit structured logs per query: `{q, flags, times, engines_used, partial, top_ids}`.

**Sprint tweak:** Add “per-collection flagging & rollout doc” to ZL-008-005/ZL-008-009.&#x20;

---

## Specific acceptance-criteria refinements

* **ZL-008-004**: “Dual-index upsert/delete produces consistent counts; reconciliation job yields zero drift over a 1k-doc sample.”
* **ZL-008-005**: “Parallel engine search with per-engine timeout; marked `partial=true` when one engine times out.”
* **ZL-008-006**: “MQE total budget ≤150 ms; cache paraphrases; fallback cleanly if generator errors.”
* **ZL-008-007**: “Dedup stable across doc reorderings; ties broken by fused score then recency.”
* **ZL-008-008**: “Report includes per-query diffs and aggregated deltas with CI.”
* **ZL-008-009**: “Cache key includes flags + collection version; ≥40% cache hit-rate on a mixed workload.”&#x20;

---

## Risks to watch (and how you’ve addressed them)

* **BM25 integration complexity:** mitigate via adapter + shared `DocId`; prototype early (Week 3).
* **Latency creep:** strict per-stage budgets; partial-results policy.
* **Quality regressions masked by caching:** versioned invalidation + CI A/B.
* **Index drift:** reconciliation job + drift alerts.

---

## Quick wins you can land this week

1. Define `DocId` + extend `SearchResult` with score breakdown & provenance.
2. Implement **parallel** vector+BM25 execution with timeout guards (even before Tantivy is fully optimized).
3. Ship the **eval harness + CI gate** (NDCG\@10 baseline now, hybrid stubbed).

These changes keep the sprint on-track while making your hybrid rollout safer, explainable, and easier to iterate.