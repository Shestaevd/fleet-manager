# fleet-agent

Background service installed on each managed machine (Linux only, systemd unit). It connects out to the
receiver over TLS and reports on the machine it runs on.

## How it runs

1. **Start-up.** The agent reads three files that sit next to the binary for now:
   - `agent.toml`: settings. If it is missing, a default one is created with a fresh `agent_id`; set `receiver_url`.
   - `host.pem`: the receiver's certificate, pinned as the only trusted one.
   - `token.txt`: the agent's token. Will be moved to the more secure space maybe as part of KV vault.
2. **Connecting** (`tcp/connector.rs`). A background task opens a TCP connection to `receiver`, does the TLS
   handshake, and splits the stream into a read half and a shared, locked write half. Failed connects are retried
   with exponential backoff and jitter behind a circuit breaker. All of it is configured in `agent.toml`.
3. **Session** (`process/session_process.rs`). Once connected, a heartbeat task sends a packet every
   `heart_beat_interval_ms` with the agent id, CPU (overall and per core), load average, memory and uptime.
   If the stream breaks, the task ends and the connector reconnects.
4. **File overwatch** (`process/fs_scan_process.rs`). Another task rescans a directory (the home directory by
   default) every `watch_interval_ms`, down to `scan_depth` levels, and keeps the result in a shared window.
   It skips unreadable entries and does not follow symlinks into directories.

## Packets

The packet format is described in the [protocol README](../protocol/README.md).

## Layout

| Path | Purpose |
|---|---|
| `domain/` | config, shared state (`Shared`), models (`FileMeta`, `Telemetry`) |
| `tcp/` | connection loop with retry and circuit breaker |
| `process/` | long-running tasks: heartbeat, file overwatch, telemetry collection |
| `fs/` | directory scanning and reading the files next to the binary |
| `test/` | unit tests, one module per area |

## Status

Done: config, connecting, heartbeat, file scanning.
Planned: running one-shot commands, uploading and downloading files, answering requests from the receiver,
installing itself as a service.
