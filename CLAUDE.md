# Fleet — project lookup (read first)

Fleet is a self-hosted fleet-management system in Rust (Tokio) for machines the owner personally owns/administers (home lab, test rigs, personal devices; Linux only). Internal use only: no multi-tenant or customer-facing parts, no productization.

## Workspace layout
| Crate (dir) | Package | Role |
|---|---|---|
| `crates/agent` | `fleet-agent` | Background service on each managed machine. Persistent connection to receiver; reports telemetry (OS, CPU, memory, uptime); runs one-shot commands; file upload/download on request. Linux only. Installs as a systemd unit (configurable display name). |
| `crates/receiver` | `fleet-receiver` | In-memory broker. Authenticates agents and operators, tracks liveness, routes operator requests to agents (request-ID correlation). |
| `crates/protocol` | `fleet-protocol` | Shared lib: message types, frame codec, serialization. Depended on by the other three. |
| `crates/model` | `fleet-model` | Shared structures and errors (`AgentId`, `RequestId`, `Telemetry`, `ConfigReadingError`). Used by all crates. |
| `crates/config` | `fleet-config` | `read_config::<T>(file_name)` reads TOML next to the executable; returns `T` or `ConfigReadingError`. |
| `crates/client` | `fleet-client` | Operator CLI: list agents, inspect telemetry, run commands, transfer files. |

## Design decisions so far
- Transport: TLS via rustls/tokio-rustls. Auth is TOKEN-BASED, not client certs (no mTLS, no cert pinning for now). A token is generated per agent registration and passed by the agent when connecting. Operator auth also by token.
- Connection model: one persistent connection per agent, authenticated once when the connection is established (agent startup and every reconnect). Telemetry doubles as heartbeat. Receiver needs: connection cap, auth/idle timeouts, one connection per agent ID, bounded queues, max frame size, failed-auth rate limit.
- Command execution: through bash (MVP), no allow-list.
- Config: TOML (`toml` + `serde`) per binary. Reconnect policy deferred.
- Framing: length-prefixed frames (`tokio-util` codec); payloads serialized with `serde` + `postcard`.
- Requests multiplexed over one connection, correlated by request ID (uuid).
- File transfer: chunked, over the same TLS stream.
- Receiver state is in-memory only (no DB).
- Shared `fleet-protocol` crate is required (decided).

## Status
- [x] Workspace + 4 crates (agent, receiver, client, protocol) scaffolded with dependencies and READMEs (stub `main` only).
- [ ] No logic written yet. User explicitly asked for scaffolding only in the first step.

## Working rules
- Ask the user whenever anything is unclear; don't guess at protocol or security decisions.
- Do not write logic unless the user asks for that step.
- Keep code in the style of the surrounding code; minimal comments.
- Leftover `src/` (hello-world from the old root package) is orphaned; user may delete it.

## Open questions
1. Server-side TLS: still wanted for encryption? If so, how does the receiver get its cert (self-signed, and do clients trust it by file/fingerprint)?
2. Registration flow: how is the token issued (receiver CLI command? client command?), single-use or long-lived, how stored (hashed in memory only, lost on restart since receiver is in-memory)?
3. Operator token: how is it created/provisioned?
4. (Resolved) Agents are Linux only, so bash is always available.
5. Command limits (timeout, output size) for MVP?
