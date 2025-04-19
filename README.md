# Flyt

What Slack should have been if salesforce didn't buy it and kill it.

Insanely fast, smart and beautiful team collaboration.

# High level architecture

```scss
┌──────────┐       Rust (Tauri backend)           ┌───────────────────────────┐
│ React UI │  ←Events→  IPC  ←Commands→           │  Tokio runtime (async)    │
│ (Vite)   │                                     │ ┌────────┐  ┌──────────┐ │
│          │   (webview)                         │ │ DB     │  │ WS Client│ │
└────┬─────┘                                     │ │SQLite  │  │tungstenite│ │
     │ UI event bus <───── Tauri.event() ───────►│ └────────┘  └────┬─────┘ │
     │                                           │        broadcast│channel │
     ▼                                           └─────────────────┴────────┘
 Virtualized list                                      ▲     ▲
 (react‑virtual)                                       │     │
     ▲                                                 │     │
     └───────────────── send_message(cmd) ─────────────┘     │
                                                             │
     Rust emits `chat://message` on any incoming packet ─────┘

```

# Network architecture

```csharp
                       ┌─────────────┐
        wss://        │  Anycast LB  │  (<15 ms RTT) 🏎
   client  ──────────►│  (Envoy/NLB) │─────────────┐
                      └─────────────┘             ▼
                                           ┌─────────────────┐
                                           │  Router Node    │
                 intra‑DC NATS JetStream   │  (Rust, Axum)   │
                                           │  ▸ WebSocket GW │
                                           │  ▸ NATS client  │
                                           │  ▸ WAL store    │
                                           └────────┬────────┘
                                                    │ publish/subscribe
                                                    ▼
                                           ┌─────────────────┐
                                           │  NATS Cluster   │  🏎
                                           │  (JetStream)    │
                                           └────────┬────────┘
                                                    │ async consumer
                                                    ▼
                                           ┌─────────────────┐
                                           │ SQL persistence │
                                           │ (Cockroach / PG)│
                                           └─────────────────┘

```

Latency budget: <16 ms render, <100 ms RTT on LAN/Wi‑Fi.

# Core Design Principles

Principle | Why it matters | How we enforce it
Keep the UI thread idle | All perceived jank comes from blocking the webview’s main thread. | ‑ All network + DB work happens in Rust. ‑ Front‑end uses virtualization & suspense boundaries.
Single, long‑lived connection | Eliminates TLS handshakes and HTTP polling. | Native WebSocket client in Rust via tauri‑plugin‑websocket Tauri
Binary, zero‑copy payloads | JSON parsing is your enemy at scale. | Protobuf (prost) ➜ ts‑proto for typed JS.
Local‑first persistence | Messages appear instantly, sync later. | SQLite + WAL (write‑ahead log) wrapped by rusqlite.
Batched UI updates | React re‑render cost dominates after 500 msgs. | Tauri event bus -> queue -> startTransition batch render. v1.tauri.app
