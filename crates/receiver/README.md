# fleet-receiver

In-memory broker between agents and operators.

Planned responsibilities:
- Accept TLS connections from agents and operators and authenticate them
- Track agent liveness
- Route operator requests to agents and responses back, correlated by request ID

Key dependencies: `tokio`, `tokio-rustls`, `dashmap`, `serde` + `postcard`, `tokio-util`.

Status: scaffold only.
