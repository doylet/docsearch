# Rust Model Host — Implementation Blueprint

This document is a practical blueprint to stand up the **Rust Model Host** that sits between your Business App (BFF) and the MCP layer (e.g., Swift "Zero Latency"), providing agent orchestration, policy enforcement, observability, and streaming.

---

## 0) Scope & Responsibilities

**Owns**
- Agent loop: **Plan → (optional) Act via MCP → Reflect → Respond** (token streaming)
- Enterprise controls: time/token/step budgets, role-based tool gating, redaction
- Observability: distributed tracing, metrics, audit ledger per turn
- Stable transport: **gRPC** bidirectional stream to BFF
- MCP client: stdio (local) and HTTP/SSE (remote); JSON Schema validation for tools

**Does _not_ own**
- Public REST surface (that's BFF)
- UI/UX
- Tool execution environments (that's MCP servers)

---

## 1) Workspace Layout

```
services/model-host/                  # binary crate (entrypoint)
crates/
  agent-core/                         # state machine, turn loop
  agent-proto/                        # generated gRPC types (from api/internal/agent.proto)
  mcp-gateway/                        # MCP transport (stdio/http), tool registry, schemas
  policy-engine/                      # allow/deny, budgets, redaction
  providers/                          # LLM adapters (traits + impls)
  memory/                             # memory traits + adapters (vector/kv)
  audit-telemetry/                    # audit sink + OpenTelemetry wiring

api/
  internal/agent.proto                # BFF ↔ Model Host (gRPC bidi)
  mcp/tools/*.schema.json             # args/results for each tool

policy/
  schemas/*.json                      # policy-bundle, tool-receipt, etc.
```

**Contracts are the source of truth; code is an implementation detail.**

---

## 2) Core Crates & Key Traits

### `agent-core`
Finite-state machine for a turn:
```
Idle → Planning → Acting (MCP) ⇄ Reflecting → Responding → Idle
```
Guards: `max_tool_depth`, `wall_ms`, `tokens_in/out`, cancellation. Emits events for streaming.

**Traits**
```rust
pub trait LlmProvider: Send + Sync {
    async fn stream_complete(
        &self,
        prompts: Vec<Message>,
        opts: InferenceOpts,
        on_token: impl FnMut(&str) + Send
    ) -> anyhow::Result<CompletionStats>;
}

pub trait ToolGateway: Send + Sync {
    async fn call(
        &self,
        call: ToolCall,          // { name, version, args: serde_json::Value }
        ctx: ToolCtx             // { tenant, role, audit_id, budgets }
    ) -> anyhow::Result<ToolResult>;
}

pub trait MemoryStore: Send + Sync {
    async fn recall(&self, q: &str, k: usize) -> anyhow::Result<Vec<KV>>;
    async fn remember(&self, item: KV) -> anyhow::Result<()>;
}
```

### `mcp-gateway`
- MCP client over **stdio** (spawn local Swift MCP process) and **HTTP/SSE** (remote servers)
- Loads tool **JSON Schemas**; validates args/results
- Enforces per-tool caps (time/bytes), produces **tool receipts**

### `policy-engine`
- Loads **signed policy bundle**
- Checks: `allow_tool`, version ranges (`^1.0.0`), egress allow-list, path allow-list (if proxied), **redaction**

### `providers`
- LLM adapters behind `LlmProvider` (Anthropic/OpenAI/etc) with fallback + breakers

### `memory`
- Namespaced by tenant/region; simple KV + vector store adapters

### `audit-telemetry`
- `tracing` + `opentelemetry-otlp`
- Metrics and structured audit records

---

## 3) Dependencies (Cargo)

```toml
# Common
anyhow = "1"
thiserror = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version="0.3", features=["env-filter","fmt","json"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "process", "time"] }
futures = "0.3"

# gRPC
tonic = { version = "0.12", features = ["tls"] }
prost = "0.13"

# HTTP client
reqwest = { version="0.12", features=["json", "stream", "gzip", "brotli", "deflate", "rustls-tls"] }

# Policies & schemas
jsonschema = "0.18"
semver = "1.0"
regex = "1.10"

# Telemetry
opentelemetry = "0.23"
opentelemetry-otlp = "0.16"
tracing-opentelemetry = "0.24"
```

---

## 4) gRPC Server Skeleton (Bidirectional Streaming)

```rust
// services/model-host/src/main.rs
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use agent_proto::zerolatency::agent::v1::{
    agent_service_server::{AgentService, AgentServiceServer},
    StreamChatRequest, StreamEvent,
};
use tracing::{error};

pub struct AgentServer {
    // inject Arc<dyn LlmProvider>, Arc<dyn ToolGateway>, Arc<Policy> etc.
}

#[tonic::async_trait]
impl AgentService for AgentServer {
    type StreamChatStream = tokio_stream::wrappers::ReceiverStream<Result<StreamEvent, Status>>;

    async fn stream_chat(
        &self,
        mut req: Request<tonic::Streaming<StreamChatRequest>>,
    ) -> Result<Response<Self::StreamChatStream>, Status> {
        let mut in_stream = req.into_inner();
        // Expect one StreamChatRequest, then emit many StreamEvent frames
        let first = in_stream.message().await?
            .ok_or_else(|| Status::invalid_argument("missing first message"))?;

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn({
            let tx = tx.clone();
            async move {
                if let Err(e) = run_turn(first, tx).await {
                    error!(error=%e, "turn failed");
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").json().init();
    let addr = "0.0.0.0:7443".parse()?;
    let svc = AgentServer { /* inject deps */ };

    Server::builder()
        .add_service(AgentServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}

// Minimal proof of streaming
async fn run_turn(
    _first: StreamChatRequest,
    tx: tokio::sync::mpsc::Sender<Result<StreamEvent, Status>>
) -> anyhow::Result<()> {
    use agent_proto::zerolatency::agent::v1::StreamEvent;
    for chunk in ["Hel", "lo, ", "world!", "\n"] {
        tx.send(Ok(StreamEvent{ r#type: "assistant.delta".into(), data: chunk.as_bytes().to_vec() })).await.ok();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    tx.send(Ok(StreamEvent{ r#type: "assistant.done".into(), data: vec![] })).await.ok();
    Ok(())
}
```

> Replace `run_turn` with your `agent-core` loop.

---

## 5) Agent Loop (First Useful Version)

**Planning (v0)**
- Concatenate last N messages + retrieved memory
- Deterministic system prompt (role/tenant, budgets, tool catalog summary)
- Ask model for JSON plan: `{"tool":"fs.search" | null, "args":{...}, "response_style":"brief"}`

**Acting**
- If tool present, call `ToolGateway.call()` with policy checks + caps; stream `tool.call.*` events

**Responding**
- Compose final prompt with observations; stream model tokens as `assistant.delta`

**Guards (day 1)**
- `max_tool_depth = 1`
- `wall_ms ≤ 15_000`
- `tokens_in/out` budgets
- Redaction before model & before logs

---

## 6) MCP Gateway (First Slice)

**Trait**
```rust
pub struct ToolCall { pub name: String, pub version: String, pub args: serde_json::Value }
pub struct ToolResult { pub ok: bool, pub result: serde_json::Value, pub bytes: usize, pub ms: u128 }

#[async_trait::async_trait]
pub trait McpTransport {
    async fn list_tools(&self) -> anyhow::Result<Vec<ToolMeta>>;
    async fn call(&self, call: &ToolCall, caps: Caps) -> anyhow::Result<ToolResult>;
}
```

**Transports**
- `StdioMcpTransport`: spawn Swift MCP executable; speak MCP over stdio
- `HttpMcpTransport`: call remote MCP server

**Validation**
- Load `api/mcp/tools/<tool>.args.schema.json` and `...result.schema.json`
- Validate args **before** call; validate result **after**
- Emit **tool receipt** (hashes, elapsed, caps used)

---

## 7) Policy & Redaction (Must‑Have)

- Load **signed policy bundle** and pin to the turn
- On each tool call:
  - `allow_tool?` + version range check
  - Egress domain allow-list (for HTTP tools)
- Redact PII before LLM calls and before logging/auditing

---

## 8) Telemetry (Day One)

- **Tracing**: propagate `traceparent`; add `audit_id`, `tenant`, `session_id` attributes
- **Metrics**:
  - `agent_turn_latency_ms`, `planning_ms`, `acting_ms`, `responding_ms`
  - `tool_calls_total{tool,outcome}`, `tokens_in/out`, `cost_usd{provider}`
- **Audit**: record per turn: model name, tools + receipts, budgets used, outcome

---

## 9) Minimal Provider Adapter

```rust
pub struct AnthropicProvider { http: reqwest::Client, api_key: String }

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn stream_complete(
        &self,
        _prompts: Vec<Message>,
        _opts: InferenceOpts,
        mut on_token: impl FnMut(&str) + Send
    ) -> anyhow::Result<CompletionStats> {
        // call streaming API, feed chunks into on_token(...)
        on_token("Hello from Anthropic!");
        Ok(CompletionStats{ tokens_in: 0, tokens_out: 0, cost_usd: 0.0 })
    }
}
```

Wire behind a provider factory with optional fallback chain.

---

## 10) First 10 Days Plan

**Day 1–2**: Workspace + gRPC server boots; `run_turn` streams tokens  
**Day 3**: `agent-core` with echo provider; budgets + cancellation  
**Day 4–5**: `mcp-gateway` scaffolding + mock transport; tool receipts  
**Day 6**: `policy-engine` (allow/deny, semver, redaction v0)  
**Day 7**: OpenTelemetry traces/metrics; structured audit JSON  
**Day 8**: Swap echo for a real LLM provider  
**Day 9**: Hook `StdioMcpTransport` to Swift MCP; demo `fs.read` happy path  
**Day 10**: E2E: BFF/UI → gRPC stream → tool call → streamed answer; audit saved

**Exit criteria**
- `StreamChat` streams tokens
- One tool works end-to-end under policy & caps
- Audit record with receipt; traces visible in APM
- Guards enforce: time, steps, tokens (fail closed)

---

## 11) Testing Strategy

- **Unit**: policy checks, schema validation, redaction, semver rules  
- **Integration**: mock MCP tool returns huge payload → enforce caps + receipt  
- **Contract**: compile `agent.proto`; golden-stream test for `StreamChat`  
- **Chaos**: break LLM provider, ensure graceful error/fallback

---

## 12) Security Defaults

- Deny all tools unless allow‑listed by policy
- `max_tool_depth = 1` initially
- Timeouts everywhere; request size caps
- PII redaction for model input and audit/log output
- No secrets in logs; TLS everywhere; Rustls trust store

---

## 13) Quickstart Commands

```bash
# gRPC codegen (recommended: buf)
brew install bufbuild/buf/buf
buf generate

# Run Model Host (dev)
cd services/model-host
RUST_LOG=info cargo run

# Hit from BFF (pseudo)
grpcurl -plaintext localhost:7443 list
```

---

## 14) Makefile Targets (suggested)

```make
# top-level Makefile
.PHONY: build host test lint

build: host

host:
	cargo build -p model-host

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets -- -D warnings
```

---

**That's it.** This blueprint is designed to be implemented incrementally while keeping your enterprise controls first-class from day one.
