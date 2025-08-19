# Zero Latency — Rust Model Host MLP Roadmap (Beta)

**Goal:** Ship a **Minimum Lovable Product** (MLP) of the **Rust Model Host** that integrates smoothly into the Zero Latency system (Client ↔ BFF ↔ Host ↔ MCP), enforces enterprise policy, and delivers reliable streaming plus one production‑grade native tool.

> **Transport clarity:** Production uses a **device‑initiated MCP Bridge** composed of a **local Device Bridge** (on the user’s machine) and a **cloud Tool Relay**. The Device Bridge speaks **stdio** to the local Swift MCP server and maintains a persistent **WebSocket/TLS** to the Tool Relay. The Host is **remote (cloud)**.

---

## 1) Beta Objectives & Success Criteria

### Objectives
- Provide a **stable gRPC bidi stream** (`StreamChat`) with low‑latency token deltas.
- Enforce **signed policy bundles** (short TTL): tool allow/deny, semver ranges, caps.
- Execute **one native MCP tool** (`fs.read`) via the **Device Bridge (local) + Tool Relay (cloud)** with **pre/post JSON Schema** validation and **tool receipts**.
- Ship **observability** (OpenTelemetry traces + metrics) and **append‑only audit**.
- Hit **performance** targets for enterprise UX (fast first token; smooth streaming).

### Success Criteria (Exit to Public Beta)
- p95 **time‑to‑first‑token ≤ 800 ms** (cached auth + policy).
- **100%** of tool calls produce receipts; **≥ 99%** tool success (policy denials excluded).
- **Contracts stable**: no breaking changes to `agent.proto` or MCP schemas in patch releases.
- **Security baseline**: TLS on all hops; deny‑by‑default tools; redaction v1 (pre‑model & pre‑log).
- **Docs**: BFF integration guide, policy authoring, local dev/runbooks, troubleshooting.

---

## 2) Scope for Beta (MLP Feature Set)

### Must‑Have
- **gRPC**: `StreamChat` events — `assistant.delta`, `assistant.done`, `tool.call.started`, `tool.call.delta`, `error`.
- **LLM provider**: one adapter with streaming; token/cost accounting if available; mock fallback.
- **MCP transport (prod)**: **Device‑initiated MCP Bridge** → **Device Bridge (local)** ↔ **Tool Relay (cloud)** over **WebSocket/TLS**.  
- **MCP transport (dev/CI)**: direct **stdio** Host ↔ local Swift MCP.
- **Contracts**: `api/internal/agent.proto`; `api/mcp/tools/fs.read/v1/{args,result}.schema.json` + `manifest.json`.
- **Policy engine**: verify signed bundles; allow/deny by tool name + **semver**; caps (timeoutMs, maxBytes); egress/path allow‑lists.
- **Validation**: JSON Schema **pre‑call** (args) and **post‑call** (result) in Host.
- **Audit & Telemetry**: OTLP traces; metrics (plan/act/respond latencies, tool success %, tokens in/out, **host_to_device_rtt_ms**); append‑only audit with **tool receipts**.
- **Guardrails**: `max_tool_depth=1`, wall‑clock timeouts, token budgets, cooperative cancellation.

### Nice‑to‑Have
- **Binary framing** (protobuf/CBOR) on WS channel.
- **Provider fallback** tree + circuit breaker.
- **Vector memory** with simple recall.
- **mTLS** between BFF ↔ Host, and optionally Device Bridge ↔ Tool Relay.

### Out of Scope (beta)
- Multi‑tool plans; write‑capable tools; complex RAG; multi‑provider routing strategies.

---

## 3) Integration Points (System Contracts)

- **UI ↔ BFF**: OpenAPI (`/v1/sessions`, `/v1/chat/stream`, `/v1/health`).
- **BFF ↔ Model Host**: gRPC (`agent.proto`) — `StreamChat` + event catalog.
- **Model Host ↔ Device Bridge (via Tool Relay)**: WS frames (JSON/protobuf); tool contracts via JSON Schema.
- **Control Plane ↔ Host**: policy bundle distribution (pull with TTL), audit export (OTLP or batched HTTP).

---

## 4) Milestones & Timeline (6 Weeks)

| Week | Milestone | Deliverables | Acceptance |
|---|---|---|---|
| **1** | Contracts & Skeleton | `agent.proto` v0.1; Host gRPC server streams **fake tokens**; crate layout + CI scaffold | `grpcurl list` works; fake token stream observed |
| **2** | Provider & Streaming | LLM adapter (streaming); token accounting; cancellation/back‑pressure; first‑token tuning | p95 ≤ 1.2s (pre‑policy), stable stream under load |
| **3** | **Device‑Initiated MCP Bridge E2E** | **Tool Relay (cloud)**; **Device Bridge (local)** with persistent WS; catalog announce; `fs.read` end‑to‑end via Relay | E2E `fs.read` with typed errors and caps enforced |
| **4** | Policy & Security | Signed bundle verify; device‑scoped policy (include `device_id`); redaction v1; TLS (rustls) | Disallowed tool blocked; receipts include `device_id` |
| **5** | Observability & Audit | OTLP traces; metrics (incl. **host_to_device_rtt_ms**, relay queue); **tool receipts** (hashes, caps, ms) | 100% calls have receipts; dashboards show latencies |
| **6** | Hardening & Beta | Soak + chaos (provider down, MCP hang); perf tuning; docs/runbooks | Success criteria met; beta tag + release notes |

---

## 5) Engineering Plan (Epics → Stories)

### E1. Contracts & Runtime
- Define `agent.proto` (StreamChat + events); compile with `tonic_build`.
- Crates: `agent-core`, `mcp-gateway`, `policy-engine`, `providers`, `audit-telemetry`.
- Server skeleton: bounded channels, cooperative cancellation, graceful shutdown.

### E2. Provider Adapter
- Implement `LlmProvider` for one vendor; stream chunks → `assistant.delta`; token accounting.
- Mock provider flag for offline dev & tests.

### E3. Device‑Initiated MCP Bridge + Tool
- **Tool Relay (cloud)**: WS router + auth; per‑tenant sharding; sticky sessions.
- **Device Bridge (local)**: persistent WS; stdio to Swift MCP; **device.hello** catalog announce.
- Implement **`fs.read`** contracts; **pre/post** JSON Schema validation in Host; produce receipts.

### E4. Policy Engine
- Verify **signed** policy bundle; cache & TTL; deny‑by‑default.
- Allow/deny by tool + semver; egress and path allow‑lists; redaction pass (regex v1).

### E5. Observability & Audit
- OpenTelemetry traces; metrics (phase latency, tool success %, tokens in/out, **host_to_device_rtt_ms**).
- Audit per turn; **tool receipt**: canonical JSON digests, caps, elapsed, policy id, `device_id`.

### E6. Security & Transport
- TLS (rustls), optional mTLS; strict propagation of `traceparent`, tenant, audit id.
- Request limits; structured error model; safe timeouts & defaults.

### E7. Hardening
- Load & soak tests; chaos (kill provider/MCP); FD/memory leak checks; back‑pressure.
- Docs: BFF integration, policy authoring, local dev/runbooks, troubleshooting.

---

## 6) Performance & Reliability Targets

- **First token**: p95 ≤ 800 ms (cached auth & policy).  
- **Bridge overhead** per tool call: p95 ≤ **100 ms**, p99 ≤ **150 ms**.  
- **Stream stability**: 100 concurrent sessions/pod without buffer blowups.  
- **Tool caps**: `maxBytes` enforced; large outputs truncated + `truncated=true`.  
- **Error budgets**: breaker limits provider blast radius; MCP timeouts surfaced with receipts.

---

## 7) Testing Strategy

- **Contract tests**: golden `StreamChat` sequences; valid/invalid schema fixtures.
- **Integration**: spawn Device Bridge and Swift MCP; run `fs.read` within sandbox via Relay.
- **Soak**: 1‑hour streams; network hiccups; policy expiry mid‑stream (graceful behavior).
- **Chaos**: provider 5xx; MCP hang; confirm timeouts, receipts, typed errors.
- **Security**: path traversal & egress denial tests; PII redaction present.

---

## 8) Release Management

- **Versioning**: `agent.proto v0.1.x`; tool schemas `v1.y.z`; policy bundle `v1`.  
- **Channels**: internal alpha → private beta → public beta.  
- **Compatibility**: only additive changes in beta; breaking changes require version bump + migration notes.

---

## 9) Deliverables Checklist (Beta)

- [ ] `services/model-host` binary; `cargo build` green.  
- [ ] `agent.proto v0.1` shipped; gRPC server passes golden tests.  
- [ ] **Tool Relay** (cloud) + **Device Bridge** (local) operational; `fs.read` path working.  
- [ ] Policy enforcement; redaction v1; TLS enabled.  
- [ ] OTLP traces + dashboards; metrics (incl. bridge RTT); audit with receipts.  
- [ ] Docs: BFF integration, policy authoring, local dev/runbook, troubleshooting.  

---

## 10) Known Deferrals (Post‑Beta)

- Multi‑tool planning; write‑capable tools; advanced RAG/memory.  
- Multi‑provider routing strategies; HTTP/SSE MCP as default transport.  
- mTLS everywhere; cost telemetry parity across vendors.  
- Live policy push from control plane (use pull/TTL in beta).
