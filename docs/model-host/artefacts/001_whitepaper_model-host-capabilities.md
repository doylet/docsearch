# Zero Latency — Rust Model Host Capabilities

This document summarizes the capabilities of the **Rust Model Host** in the Zero Latency architecture.

---

## Core Orchestration
- **Agent loop**: plan → (optional) act via MCP tools → reflect → respond.
- **Token streaming**: low-latency incremental deltas to the client; fast time-to-first-token.
- **Back-pressure & cancellation**: cooperative cancellation and bounded streaming queues.
- **Guardrails**: step limits, wall-clock timeouts, token in/out budgets.

## Policy, Security & Governance
- **Authoritative enforcement point**: deny-by-default; per-tool allow-lists with **semver** version ranges.
- **Signed policy bundles**: verification, short TTLs, graceful degrade on expiry.
- **Redaction**: PII/secret masking **before** model calls and **before** logging/audit.
- **Multi-tenant/roles**: apply tenant/role context on every turn and tool call.
- **Secrets isolation**: provider keys reside in Host (KMS-backed in prod), never in the client.
- **Transport security**: TLS (rustls), optional mTLS; strict header/trace propagation from BFF.

## MCP Tool Integration
- **MCP client**: stdio (spawn local Swift MCP) and HTTP/SSE (remote).
- **Contract-first tools**: JSON Schema for **args/results** per tool; **pre-call** and **post-call** validation in Host.
- **Tool receipts**: deterministic hashes, elapsed ms, and caps used per call; appended to audit.
- **Tool registry**: discover tools/versions/capabilities; enforce per-tool caps and egress/path allow-lists.

## LLM Providers & Reasoning
- **Provider adapters**: pluggable `LlmProvider` trait (Anthropic, OpenAI, local, etc.).
- **Streaming completions** with token/cost accounting (when available).
- **Fallbacks & circuit breakers**: provider health metrics and auto failover.
- **Prompt scaffolding**: deterministic system prompts, JSON plan extraction, recovery prompts.

## Memory & Retrieval
- **Memory abstraction**: short-term/episodic memory via trait; KV or vector backends.
- **Selective recall/remember**: tenant/region-scoped; size/time bounded.
- **Optional RAG**: retrieval hooks before planning/responding.

## Interfaces & Contracts
- **gRPC (bidi) to BFF**: `StreamChat` emitting `assistant.delta`, `tool.call.started`, `tool.call.delta`, `assistant.done`, `error`.
- **Contracts as source of truth**:
  - `api/internal/agent.proto` — service boundary (BFF ↔ Host)
  - `api/mcp/tools/*/*.schema.json` — MCP tool args/results schemas
  - `policy/schemas/*.json` — policy bundles & tool receipts

## Observability & Audit
- **Tracing**: OpenTelemetry; propagate `traceparent`; attach `tenant`, `session_id`, `audit_id`.
- **Metrics**: plan/act/respond latencies, tool success %, breaker states, tokens & (optional) cost.
- **Audit ledger**: immutable records per turn with tool receipts; offline queue & backfill.

## Reliability & Operations
- **Stateless by default**: horizontal scale; caches for schemas/policies/tool catalogs.
- **Config via env/flags**; safe defaults; feature-gated providers/tools.
- **Graceful shutdown**: drain streams, finalize audit, cancel in-flight MCP work.

## Performance Posture
- **Fast first-token** target (sub-second with cached auth/policy).
- **Bounded memory/IO** on tools; stream large outputs with caps.
- **Async throughout** (Tokio), efficient JSON handling.

## Extensibility
- **Crate boundaries**: `agent-core`, `mcp-gateway`, `policy-engine`, `providers`, `memory`, `audit-telemetry`.
- **New tools**: add JSON Schemas + manifest; Host typically needs only registry config.
- **New providers**: implement `LlmProvider` once; reusable across agents.

## Explicit Non-Goals
- No public REST surface (that’s the **BFF**).
- No UI/UX.
- No unsandboxed tool execution (tools run via **MCP** only).
- No long-term business data ownership (that’s the control plane).

---

## MVP vs. Near-Term Add-Ons

**MVP (ship first)**
- gRPC `StreamChat` + token streaming
- One provider adapter
- MCP stdio transport + one tool (e.g., `fs.read`) under policy
- JSON Schema validation (pre/post) + tool receipts
- Tracing + minimal metrics + audit JSON
- Budgets (time/steps/tokens) and redaction v1

**Near-term**
- Provider fallback tree + circuit breakers
- HTTP/SSE MCP transport
- Vector memory adapter + simple retrieval
- Policy live-reload and cache warming
- mTLS between BFF↔Host, structured cost telemetry
