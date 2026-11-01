# Fleet

Self-hosted fleet management for machines I own, written in Rust (Tokio). Internal administration only.

```
client ──TLS──▶ receiver ◀──TLS── agent
```

Agents connect out to the receiver and report on their machine. Operators use the client to talk to agents through
the receiver. Agents authenticate with a token and trust only the receiver's pinned certificate.

## Crates

| Crate | Description |
|---|---|
| [`fleet-agent`](crates/agent/README.md) | Service on each managed machine (Linux only) |
| [`fleet-receiver`](crates/receiver/README.md) | In-memory broker routing operator requests to agents |
| [`fleet-client`](crates/client/README.md) | Operator CLI |
| [`fleet-protocol`](crates/protocol/README.md) | Packet format and message types shared by all three |
| [`fleet-crypto`](crates/crypto/README.md) | Certificates, tokens and secrets |
| `fleet-config` | Reads TOML config files that sit next to the binary |
| `fleet-model` | Shared ids and error types |

## Build and test

```
cargo build --workspace
cargo test --workspace
```
