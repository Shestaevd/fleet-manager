# Fleet

Self-hosted fleet management for machines, written in Rust (Tokio). Internal administration only.

| Component | Crate | Description |
|---|---|---|
| [Agent](crates/agent/README.md) | `fleet-agent` | Service on each managed machine |
| [Receiver](crates/receiver/README.md) | `fleet-receiver` | Broker routing operator requests to agents |
| [Protocol](crates/protocol/README.md) | `fleet-protocol` | Shared message types and codec |
| [Client](crates/client/README.md) | `fleet-client` | Operator CLI |

```
client ──TLS+token──▶ receiver ◀──TLS+token── agent
```

## Build
```
cargo build --workspace
```

Status: scaffolding only; no logic implemented yet. See [CLAUDE.md](CLAUDE.md) for design notes and open questions.
