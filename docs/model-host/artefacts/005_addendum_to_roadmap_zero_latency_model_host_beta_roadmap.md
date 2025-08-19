# ADR-0002 — Model Host Placement: Remote by Default with a **Device‑Initiated MCP Bridge**

- **Status:** Accepted
- **Date:** 2025-08-19
- **Owners:** Zero Latency Architecture Group
- **Decision Type:** Deployment / Topology
- **Related:** ADR‑0001 Monorepo Structure, Model Host MLP Roadmap (Beta)

---

## Decision

Run the **Model Host** in the **cloud (remote)** by default and connect to device‑native MCP tools via a **device‑initiated MCP Bridge**, comprised of:
- a **local Device Bridge** (desktop/daemon on the user’s machine) that speaks **stdio** to the local **Swift MCP Server**, and
- a **cloud Tool Relay** that terminates a persistent **WebSocket/TLS** from the Device Bridge and forwards frames to/from the Host.

**Local Host** remains a supported alternative for tenants requiring offline or ultra‑low latency; contracts remain the same.

---

## Context

- The Host orchestrates planning/acting with LLMs, enforces policy & budgets, validates tool I/O with JSON Schemas, and audits.
- MCP tools are native, sandboxed, and run locally on user devices.
- Enterprises prefer centralized governance, observability, and rapid upgrades while keeping device‑native capabilities.

---

## Topology (Remote‑by‑Default)

```
Client UI → BFF (cloud) ⇄ Model Host (cloud)
                              ⇅
                        Tool Relay (cloud)
                              ⇅  (device‑initiated WebSocket/TLS, persistent)
                     Device Bridge (local on device)
                              ⇅  (stdio)
                     Swift MCP Server (native tools, sandboxed)
```

**Dev/CI:** Host may speak stdio to a local MCP for tests.

---

## Options & Trade‑offs

| Option | Summary | Pros | Cons | Decision |
|---|---|---|---|---|
| **A. Remote Host + Device‑Initiated MCP Bridge** (Chosen) | Host in cloud; Device Bridge (local) WS→Tool Relay (cloud); stdio→local MCP | Centralized control, secrets in KMS, unified observability, scalable | Adds WAN hop per tool call; requires Bridge & Relay | ✅ |
| **B. Local Host + Local MCP** | Both on device; stdio | Lowest tool latency; offline | Hard fleet ops; secrets on devices; fragmented observability | ◻︎ |
| **C. Split Host** | Local tool runner + remote reasoning | Near‑zero tool latency + centralized reasoning | More moving parts; complex failure modes | ◻︎ |

---

## Consequences

- **Latency:** Small predictable overhead per tool call (target p95 ≤ 100 ms). Chat‑only TTFT unchanged.
- **Edge agent:** We ship a lightweight Device Bridge alongside the MCP server.
- **SRE:** Centralized metrics/audit; simpler rollbacks and incident response.
- **Security:** Outbound‑only from devices; short‑lived tokens; optional mTLS; sandbox stays on device.

---

## Security, Policy & Audit

- Deny‑by‑default tools; per‑tool **semver** ranges and caps (timeoutMs, maxBytes).
- **Signed policy bundles** (short TTL); sensitive tools disable on expiry.
- **Pre/post JSON Schema** validation in the Host.
- **Receipts** per call include `device_id`, policy id, schema digests, caps used, elapsed (incl. relay).

---

## Performance Targets & SLOs

- Bridge overhead p95 ≤ **100 ms**, p99 ≤ **150 ms**.  
- Time‑to‑first‑token (no tools) p95 ≤ **800 ms**.  
- Tool success ≥ **99%** (policy denials excluded).  
- WS session uptime ≥ **99.9%** (auto‑reconnect with backoff).

---

## Implementation Steps (Beta)

1. Finalize `agent.proto` and MCP tool schemas (`api/mcp/tools/*`).  
2. Build **Tool Relay** (cloud) and **Device Bridge** (local) with persistent WS and **device.hello** catalog.  
3. Add `remote_ws` transport to Host; keep `stdio` for dev.  
4. Extend policy scope with `device_id` / device class; enforce caps Host+Bridge.  
5. Update receipts (add `device_id`, relay timings); dashboards for RTT and queue metrics.

**Flags**
```
HOST_MODE=remote|local
MCP_TRANSPORT=remote_ws|stdio
RELAY_URL=wss://relay.example.com
TENANT_ID=...
DEVICE_ID=...
POLICY_SOURCE=url|file
```

---

## Exception Policy (When to choose Local Host)

Permit **Local Host** for tenants needing **offline** use or **<50 ms** tool loop latency. Require: explicit exception, local secret storage review, enhanced device telemetry, and audit shipping plan.

---

## Review & Revisit

Quarterly review or upon SLO breach/major customer requirements.

---
# Addendum — Device‑Initiated MCP Bridge (clarifies production transport)

**What this clarifies:** In production, the Model Host is **remote (cloud)** and the MCP server is **local (device)**. Tool calls traverse a **device‑initiated MCP Bridge** made of a **local Device Bridge** (desktop/daemon) and a **cloud Tool Relay**. Dev/CI may still use **stdio** from Host ↔ MCP on the same machine.

---

## Updated Topology

```
Client UI → BFF (cloud) ⇄ Model Host (cloud)
                              ⇅
                        Tool Relay (cloud)
                              ⇅  (device‑initiated WebSocket/TLS, persistent)
                     Device Bridge (local on device)
                              ⇅  (stdio)
                     Swift MCP Server (native tools, sandboxed)
```

---

## Scope Corrections (Roadmap)

- **Must‑Have (prod):** Remote transport via **Device Bridge (local)** ↔ **Tool Relay (cloud)** over **WebSocket/TLS** (or SSE).  
- **Dev/CI:** Keep direct **stdio** path for local development.  
- **Per‑device catalogs:** Device sends **device.hello** (tool name, version, schema digests).  
- **Policy & receipts:** Include `device_id`; enforce caps; record relay timings in receipts.

---

## Minimal Protocol Frames

**Device Hello**
```json
{
  "type": "device.hello",
  "device_id": "mac-123",
  "tenant": "acme",
  "catalog": [
    { "name": "fs.read", "version": "1.2.0",
      "schema": { "args": "urn:zl:fs.read:1.2.0:args", "result": "urn:zl:fs.read:1.2.0:result" } }
  ]
}
```

**Tool Request → Device**
```json
{
  "type": "tool.call.start",
  "correlation_id": "uuid",
  "tenant": "acme",
  "device_id": "mac-123",
  "tool": { "name": "fs.read", "version": "1.2.0" },
  "args": { },
  "caps": { "timeoutMs": 5000, "maxBytes": 1048576 },
  "policy_id": "bundle-2025-08-19"
}
```

**Streaming Responses ← Device**
```json
{ "type": "tool.call.delta", "correlation_id": "uuid", "chunk": "..." }
{ "type": "tool.call.completed", "correlation_id": "uuid", "result": { }, "elapsed_ms": 43 }
{ "type": "tool.call.error", "correlation_id": "uuid", "code": "TIMEOUT", "message": "..." }
```

---

## Latency & SLOs (unchanged targets)

- **Bridge overhead per tool call**: p95 ≤ **100 ms**, p99 ≤ **150 ms**.  
- **TTFT (no tools)**: p95 ≤ **800 ms**.  
- Mitigations: persistent WS, Host+Relay same region, streaming deltas, optional edge PoPs, binary framing.

---

## Operational Notes

- Outbound‑only from devices (no inbound ports).  
- Short‑lived device tokens; optional mTLS/attestation.  
- Sticky routing; idempotent correlators for reconnects.  
- Cache policy bundles on device; degrade sensitive tools on expiry; queue audit until reconnect.
