Here’s how I’d evolve **docsearch** into a crisp “agent layer” between your chat client and the MCP worldwithout bloating it.

---

# 0) Two clean deployment modes (choose per environment)

* **Mode A MCP Server (tools provider):** docsearch exposes high-quality retrieval tools to any MCP broker (stdio/HTTP). Great when you want other agents to *use* docsearch.
* **Mode B Agent Host (planner):** docsearch is the chat-facing agent that *calls out* to an MCP broker (including your other servers) and composes answers. Use this when you want docsearch to orchestrate.

> Keep the same core engine either way. Only the “edge” (MCP server vs MCP client) changes.

---

# 1) Agent contract: the minimal, powerful tool set

Make these tools first-class and stable (JSON Schema / OpenAPI). They cover 90% of RAG needs and keep policies simple.

**Core tools**

1. `search` hybrid retrieval (BM25 + vector), with filters
**in:** `{query, top_k, filters?, collection?}`
**out:** `{hits: [{id, title, snippet, uri, score, source, chunk_id}]}`

2. `lookup` exact fetch by id/uri
**in:** `{id|uri}`
**out:** `{doc: {id, title, text, metadata}}`

3. `summarize` focused summarization with citations
**in:** `{ids|passages, focus?, style?}`
**out:** `{summary, citations:[{id, span}...]}`

4. `answer_with_citations` (RAG compose)
**in:** `{question, top_k?, filters?}`
**out:** `{answer, citations:[{id, uri, quote}]}`

**Nice-to-have**

* `similar` (more-like-this for an ID)
* `extract_entities` (structured pulls for downstream automation)
* `pin` / `save_highlight` (user memory)

> In **Mode A** you serve these as MCP tools. In **Mode B** the agent calls these internally and also calls *external* MCP tools via your broker.

---

# 2) Retrieval quality upgrades (high impact, low surface area)

* **Hybrid by default:** BM25 (tantivy) + vector (Qdrant or memory) **fuse** with a simple normalized sum. Return both sub-scores in results.
* **Multi-query expansion:** generate 23 paraphrases, merge their top-k before rerank (big recall win).
* **Reranking:** small cross-encoder (or an LLM “fast judge”) for the final top-k; cache aggressively.
* **Self-query filters:** parse time/filetype/tags from the question and push down to metadata.
* **Chunk policy:** 5121k tokens with **overlap**; store **semantic title** and **section path** for better snippets.

---

# 3) Planner & policy (when docsearch is the agent host)

* **Budgeted tool loop:** ReAct-style, but with ceilings: `max_tool_calls`, `max_latency_ms`, `min_confidence_to_answer`.
* **Tool selection rules:** if question is *factual* or asks “according to the docs…”, always hit `search`; if answerable from **session memory**, avoid calls.
* **Halting conditions:** stop when marginal gain from another fetch is below threshold (e.g., reranker score delta).
* **Always cite** in the final answer.

---

# 4) Memory model (keeps chats sharp without polluting retrieval)

* **Ephemeral session memory:** running summary + pinned facts (TTL).
* **Durable user memory (optional):** saved highlights and pins go to a separate “notes” collection.
* **Never** mix user memory into primary corpus indexes; treat it as a separate collection that can be included by policy.

---

# 5) Transport & lifecycle (kill port pain)

* **stdio** for local/MCP tool use; **HTTP** for service mode; optional **UDS** locally (no ports at all).
* Health endpoints: `/healthz` (process), `/readyz` (vector store reachable, cache warm).
* Supervisor mode (dev): spawn dependencies, parse a `SERVER_READY {...}` line, register dynamically.

---

# 6) Security & permissions

* **ACL check on every fetch** (collection-level & per-doc if you have it).
* **Redaction filter** (PII/keys) at render time, not during indexing.
* **Audit trail:** query doc IDs returned final citations.

---

# 7) Observability & evaluation

* **Structured events:** `query.received`, `retrieval.run`, `rerank.run`, `compose.answer`, each with timings and counts.
* **Metrics:** P50/P95 per stage, cache hit rate, tool calls per answer, fail/open-circuit counts.
* **Eval harness:** tiny labeled set; track NDCG\@10, Hit\@k, and “citation correctness” (answer spans must exist in cited docs).

---

# 8) Performance tricks that matter

* **Result cache** keyed on `(normalized_query, filters, top_k)`; invalidate by collection version.
* **Warmup:** embed common terms, preload collection stats, and JIT-compile reranker on start.
* **Batch embeddings** and **rate limit** providers with backpressure.

---

# 9) Minimal schemas (copy/paste into your contracts)

```json
// tool: search
{
  "type":"object",
  "properties":{
    "query":{"type":"string"},
    "top_k":{"type":"integer","minimum":1,"maximum":50,"default":8},
    "filters":{"type":"object","additionalProperties":true},
    "collection":{"type":"string"}
  },
  "required":["query"]
}
```

```json
// tool: answer_with_citations (RAG compose)
{
  "type":"object",
  "properties":{
    "question":{"type":"string"},
    "top_k":{"type":"integer","default":6},
    "filters":{"type":"object"},
    "collection":{"type":"string"},
    "style":{"type":"string","enum":["concise","default","technical"],"default":"default"}
  },
  "required":["question"]
}
```

**Response (common)**

```json
{
  "answer":"...",
  "citations":[{"id":"...","uri":"...","quote":"..."}],
  "debug":{"bm25":[...],"vector":[...],"rerank":[...],"latency_ms":1234}
}
```

---

Awesome here’s a **comprehensive, end-to-end sprint plan** to evolve **docsearch** into a clean agent layer between a chat client and your MCP broker/tools. Each sprint has a **Goal, Tasks, and Acceptance Criteria**. It assumes two run modes:

* **Mode A: MCP Server (tools provider)** docsearch exposes retrieval tools (stdio/HTTP).
* **Mode B: Agent Host (planner)** docsearch is the chat-facing agent that calls tools via your MCP broker (including docsearch’s own tools if running as a provider elsewhere).

---

# Sprint 1 Contracts & Tool Surface (foundation)

## Goal

Define a minimal, powerful tool surface and stable schemas used in both modes.

## Tasks

* Define JSON Schemas (and MCP tool descriptors) for:

* `search`, `lookup`, `similar`, `summarize`, `answer_with_citations`.
* Normalize result shape (ids, URIs, snippet, section path, scores, citations).
* Create a thin **Ports** layer (traits/interfaces) for `SearchPort`, `IndexPort` (read-only here), `SummarizePort`.
* Add `/healthz` and `/readyz` endpoints; `readyz` checks vector store reachability.

**MCP tool descriptor examples (copy/paste)**

```json
// tool: search
{
  "name": "search",
  "description": "Hybrid search over indexed documents with optional filters.",
  "input_schema": {
    "type": "object",
    "properties": {
      "query": {"type":"string"},
      "top_k": {"type":"integer","minimum":1,"maximum":50,"default":8},
      "filters": {"type":"object","additionalProperties":true},
      "collection": {"type":"string"}
    },
    "required": ["query"]
  }
}
```

```json
// tool: answer_with_citations
{
  "name": "answer_with_citations",
  "description": "Retrieve, (re)rank and compose a cited answer.",
  "input_schema": {
    "type": "object",
    "properties": {
      "question":{"type":"string"},
      "top_k":{"type":"integer","default":6},
      "filters":{"type":"object"},
      "collection":{"type":"string"},
      "style":{"type":"string","enum":["concise","default","technical"],"default":"default"}
    },
    "required":["question"]
  }
}
```

## Acceptance Criteria

* MCP registry shows the five tools with valid schemas.
* `search` returns `{hits:[{id, uri, title, snippet, section, score, source}]}`.
* `answer_with_citations` returns `{answer, citations:[{id, uri, quote}]}`.
* `GET /healthz` returns 200; `GET /readyz` returns 200 only if dependencies are reachable.

---

# Sprint 2 Hybrid Retrieval + Multi-Query Expansion + Reranking

## Goal

Lift retrieval quality and robustness while keeping latency tight.

## Tasks

* Implement **hybrid retrieval**: BM25 (tantivy) + vector (Qdrant/in-mem).
* **Multi-Query Expansion (MQE)**: generate 23 paraphrases; union results (dedupe).
* **Reranker**: small cross-encoder or fast LLM judge over top-k (configurable).
* Return **score breakdown**: `{bm25_score, vector_score, rerank_score, fused_score}`.
* Snippet builder: include **semantic title** and **section path**.

## Acceptance Criteria

* On a tiny labeled set, **NDCG\@10** improves ≥10% vs baseline BM25-only.
* P95 latency for `search` ≤ 350 ms (no rerank) and ≤ 900 ms (with rerank) on your dev box.
* Response includes score breakdowns and stable ranking with reranker enabled.

---

# Sprint 3 Durable Ingestion (jobs + idempotency) *(read-only agent still works without this, but you’ll want it)*

## Goal

Crash-safe indexing with explicit jobs and idempotency; enables safe re-index and deletes.

## Tasks

* `jobs` table (SQLite is fine): `id, stage, payload, state, attempts, started_at, finished_at, idempotency_key`.
* Stages: `discover extract chunk embed upsert`.
* Idempotency key = `{collection}:{content_hash}:{stage}`; skip if seen.
* CLI: `docsearch index <path|collection>`, `status <job_id>`, `retry <job_id>`, `reindex --since <ts>`.

## Acceptance Criteria

* Kill process mid-index; restart resumes from last successful stage automatically.
* Re-submitting same content doesn’t duplicate chunks/embeddings.
* `index search` demo produces expected results.

---

# Sprint 4 Transport & Lifecycle (stdio/HTTP/UDS + supervisor)

## Goal

Make ports disappear (dev) and lifecycle boring.

## Tasks

* Implement **stdio** transport for MCP mode; **HTTP** for service mode; optional **UDS** locally.
* Add **Supervisor (dev)**: spawn dependencies, parse `SERVER_READY { transport, addr }`, auto-register.
* Config toggle to switch stdio http without code changes.

## Acceptance Criteria

* Local dev: `docsearch serve --mode mcp-stdio` exposes tools with **no open TCP port**.
* Service mode: `docsearch serve --mode http` exposes `/tools/*` and health endpoints.
* Supervisor brings up vector store (if local), waits readiness, and tears down cleanly.

---

# Sprint 5 Agent Planner (Mode B) with Budgets & Halting

## Goal

A deterministic, budgeted tool-use loop that always cites.

## Tasks

* Implement a **budgeted planner** (ReAct-ish): ceilings for `max_tool_calls`, `max_latency_ms`, `max_tokens`.
* **Halting rules**: stop when rerank gain < ε, or confidence ≥ τ, or budget exceeded.
* **Session memory** (ephemeral): running conversation summary + short-term facts.
* Always include citations; never hallucinate beyond retrieved spans.

**Planner pseudocode**

```python
def answer(question, session):
    budget = Budget(max_calls=3, max_ms=2000)
    ctx = session.memory.peek()
    plan = propose_initial_plan(question, ctx)

    results = []
    while budget.ok():
        q_list = expand_queries(question, ctx)          # MQE
        hits = hybrid_search(q_list, filters=plan.filters, top_k=8)
        reranked = rerank(hits, question)               # returns list with scores
        results.append(reranked)

        if confident_enough(reranked) or marginal_gain_small(results):
            break
        refine(plan, results, ctx)                      # narrow filters or add synonyms

    support = fuse(results)
    answer, cites = compose_with_citations(question, support, style=plan.style)
    session.memory.update(answer, cites)
    return answer, cites
```

## Acceptance Criteria

* On a 10-question mixed set, avg **tool calls ≤ 3**; 100% answers include **valid citations**.
* Planner obeys latency budget (e.g., ≤ 2 s wall time) and stops on halting thresholds.
* Manual eval shows significantly fewer unnecessary tool calls vs naive loop.

---

# Sprint 6 Memory, Personalization & Pins

## Goal

Sharper conversations without polluting the corpus.

## Tasks

* **Ephemeral session memory**: rolling summary + salient facts with TTL.
* **Pins**: user-approved snippets saved to a separate “notes” collection.
* Retrieval policy: `search(collections=[primary, notes])` when appropriate.
* Provide `pin_highlight` tool; surface “You pinned this” in answers.

## Acceptance Criteria

* Pinned content is retrievable and cited distinctly (`source:"notes"`).
* Session memory improves follow-ups (qualitatively) without extra tool calls in >50% of tested threads.
* No “pins” leak into primary collection indexes.

---

# Sprint 7 Observability & Evaluation

## Goal

Make behavior inspectable and measurable.

## Tasks

* **Structured logs** (JSON) with `trace_id`, `session_id`, `tool_call_id`, per-stage timings.
* **Metrics**: P50/P95 latency per stage, cache hit rate, tool calls per answer, failures/retries.
* **Tracing** (OTel): spans for `search`, `rerank`, `compose`, and planner iterations.
* **Eval harness**: NDCG\@10, Hit\@k, citation correctness (answer spans must exist in cited docs).

## Acceptance Criteria

* One “ask answer” trace shows nested spans with timings and IDs.
* `docsearch eval run` prints metrics; CI enforces “don’t regress beyond X%” guardrails.
* Dash-like log line for each answer: `{q, calls, ms, cited_docs, confidence}`.

---

# Sprint 8 Security, ACLs, Redaction & Audit

## Goal

Safe by default in teams.

## Tasks

* **Collection ACLs**: user/session allowed collections; enforce on every fetch.
* **Redaction hook**: PII/key redaction at render (not during indexing).
* **Audit log**: question, doc IDs accessed, timestamps, and outcome.

## Acceptance Criteria

* A user without access to `collection B` never sees hits from it (unit + E2E tests).
* Redaction visibly masks configured patterns in snippets/answers.
* Audit trail can reconstruct which docs supported each answer.

---

# Sprint 9 Performance & Scaling

## Goal

Lower latency, higher throughput, graceful under load.

## Tasks

* **Result cache** for `(norm_query, filters, top_k)` with TTL; invalidate by collection version.
* **Batch embeddings**; rate-limit providers; back-pressure on ingest queue.
* **Warmup**: pre-load collection stats, hot vectors, and compile models.

## Acceptance Criteria

* P95 `search` (no rerank) ≤ 250 ms, with 2× concurrency vs Sprint 2 baseline.
* Ingest can sustain X docs/minute without error spikes; back-pressure prevents meltdown.
* Cache hit rate > 40% on a realistic query set.

---

# Sprint 10 DevEx & Ops Polish

## Goal

Boring to run, easy to contribute to.

## Tasks

* `make dev|test|serve|eval|smoke`; `docker compose up` for service mode.
* Pre-commit: fmt/lint/typecheck; “doctor” checks (naming, manifests).
* Runbooks: “Operate & Debug,” “Index & Reindex,” “Latency tuning.”

## Acceptance Criteria

* New dev can run `make dev` and complete a smoke chat in <5 minutes.
* CI: unit + E2E (stubbed providers) + eval + lint/type all green.
* Docs cover common failure modes and quick fixes.

---

## Global Definition of Ready / Done

**DoR**

* Tool schemas agreed; non-ambiguous inputs/outputs.
* For retrieval sprints: a tiny labeled dataset is prepared.

**DoD**

* Docs updated (README + `/docs/`).
* CI green; smoke test passes.
* Metrics & logs present for all new behavior.

---

## Stretch (later)

* **Rerankers:** switchable cross-encoder vs LLM judge with cost/latency budget.
* **Online learning:** collect clicks/accepts as weak labels to auto-tune fusion weights.
* **Multi-tenant isolation:** per-tenant quotas, index sharding, and routing.

