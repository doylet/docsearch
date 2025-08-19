# Zero Latency — Product Architecture (One‑Pager)

> **Positioning:** An enterprise AI platform that pairs **centralized policy & observability** with **embedded, sandboxed, device‑native tools**, delivered through a thin client and a policy‑enforcing Model Host.

---

## High‑level Topology

```mermaid
flowchart LR
    subgraph Client["Client UI (Web/Desktop)"]
      U[User Session<br/>SSO token, traceparent]
    end

    subgraph Cloud["Cloud (Gateway/BFF + Control Plane)"]
      G[Gateway/BFF<br/>Public API, authZ, quotas]
      CP[Control Plane<br/>Tenants, policy bundles, audit & metrics]
    end

    subgraph Host["Model Host (Rust)"]
      A[Agent Loop<br/>plan → act(MCP) → reflect → respond]
      P[Policy Enforcement<br/>budgets, allow/deny, redaction]
      M[Memory + LLM Providers]
    end

    subgraph Edge["MCP Server (Swift)"]
      T[Native Tools<br/>fs.search/read, calendar.read, ...]
      S[Sandbox<br/>seatbelt/App Sandbox, caps]
    end

    U -->|REST (OpenAPI)| G
    G <--> |gRPC (agent.proto)| A
    A --> P
    A <--> |MCP stdio/HTTP (JSON Schema)| T
    T --> S
    G --> CP
    A -->|Audit + Metrics| CP
```

---

## Components (authoritative definitions)

1. **Client (UI/UX; “dumb” front end)**  
   - Web and/or desktop shell.  
   - Carries identity (SSO), `traceparent`, `session_id`.  
   - No model keys, no tool logic. Streams tokens & events.

2. **Cloud**  
   **Gateway/BFF** (hot path): Public REST, authZ/quotas, request shaping, pass‑through streaming.  
   **Control Plane** (off path): Tenants, **policy bundle authoring & signing**, fleet mgmt, audit lake, metrics, update rings.

3. **Model Host (Rust)**  
   - Agentic orchestration loop (plan/act/reflect/respond).  
   - **Policy enforcement point**: tool allow/deny, semver ranges, budgets (time/tokens/steps), redaction, egress gating.  
   - LLM provider adapters + memory; emits **tool receipts** and audit.  
   - gRPC **bidi** stream with the Gateway/BFF.

4. **MCP Server (Swift, device‑native)**  
   - Exposes **JSON‑schema’d** tools/resources via MCP (stdio & HTTP/SSE).  
   - Runs **sandboxed** (seatbelt/App Sandbox), with path & egress allow‑lists and per‑tool caps.  
   - Produces deterministic **receipts** for each tool call.

---

## Contracts (source of truth)

- **UI ↔ Gateway**: `api/public/openapi.yaml` (OpenAPI 3.1) — sessions, `/chat/stream`, health.  
- **Gateway ↔ Model Host**: `api/internal/agent.proto` (gRPC) — `StreamChat`, `StreamEvent`.  
- **Model Host ↔ MCP**: **MCP** tool/resource contracts — `api/mcp/tools/<tool>.args.schema.json` & `...result.schema.json`.  
- **Governance**: `policy/schemas/policy-bundle.v1.json` and `tool-receipt.v1.json` (signed bundles, deterministic receipts).

---

## Policy & Audit Lifecycle (happy path)

1. **Control Plane** issues a signed **policy bundle** (TTL, allow/deny, caps, egress allow‑list).  
2. **Client** attaches tenant/role; **Gateway** injects `traceparent`/`audit_id`.  
3. **Model Host** verifies bundle, enforces budgets and tool rules **per call**.  
4. **MCP Server** executes tools in sandbox; returns results + **tool receipts** (hashes, lat, caps used).  
5. **Host** streams assistant tokens; **Gateway** forwards to Client.  
6. **Host** ships audit/metrics to **Control Plane** (async if offline).

---

## Security Guarantees (default‑on)

- **Deny‑by‑default tools**; allow‑list with semver ranges.  
- **Short‑TTL signed policies**; grace window; sensitive tools disabled on expiry.  
- **Sandboxed tools** (no write/exec unless allowed; network off unless proxied).  
- **Path & egress validation** (canonical paths, domain allow‑list, size/time caps).  
- **Redaction** pre‑model and pre‑log; no secrets in UI/BFF logs.  
- **End‑to‑end trace** (`traceparent`) + **append‑only audit** with receipts.

---

## Deployments (reference)

- **Enterprise Mode (recommended)**  
  Client → **Gateway/BFF** → **Model Host** → **MCP (device)**; **Control Plane** for policy/audit.

- **Direct Mode (labs/SMB)**  
  Client → **Model Host** (Host exposes a BFF‑shaped API); same contracts.

- **Offline Edge**  
  Cached policy bundle; Host queues audit until reconnect; sensitive tools degrade gracefully.

---

## SLOs & KPIs (starter set)

- **p95 time‑to‑first‑token**: ≤ 800 ms (cached auth).  
- **Tool call success rate**: ≥ 99% (denials are policy, not failures).  
- **Audit coverage**: 100% of tool calls with valid receipts.  
- **Policy freshness**: ≥ 95% nodes on latest bundle ≤ 24h.  
- **Security posture**: 100% sandboxed tools, 0 network‑off exceptions without policy.

---

## Messaging (buyer‑friendly)

- **“Central control, local capability.”** Policies and monitoring in the cloud; execution on the user’s device.  
- **“Safe by design.”** Signed policies, sandboxed tools, deterministic receipts.  
- **“Plug‑and‑play tools.”** MCP + JSON Schemas avoid bespoke integrations; works across Mac fleets.

---

## Next Steps (implementation quick wins)

1. Create folders per the monorepo plan; move **Swift MCP** → `edge/zerolatency-mcp`, **Rust Host** → `services/model-host`.  
2. Add contract stubs (`openapi.yaml`, `agent.proto`) and policy schemas.  
3. Stand up gRPC **`StreamChat`** skeleton; wire a single MCP tool (`fs.read`) end‑to‑end under policy.  
4. Enable audit receipts and OTLP tracing; add CI checks for contract linting.  
