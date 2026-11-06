# fleet-agent

Background service installed on each managed machine (Linux only, systemd unit). It connects out to the
receiver over TLS and reports on the machine it runs on.

## How it runs

1. **Start-up.** The agent reads two files that sit next to the binary for now:
   - `agent.toml`: settings. If it is missing, a default one is created with a fresh `agent_id`; set `receiver_url`.
   - `host.pem`: the receiver's certificate, pinned as the only trusted one.

   It keeps two more files there itself: `.token.txt`, the hash of its token (written owner-only; the agent never
   holds the token itself), and `.probation`, an empty marker that means "registration was sent, waiting for a decision".
2. **Connecting** (`tcp/connector.rs`). A background task opens a TCP connection to `receiver_url`, does the TLS
   handshake, and splits the stream into a shared, locked read half and write half. Failed connects are retried
   with exponential backoff and jitter behind a circuit breaker. All of it is configured in `agent.toml`.
   Every new connection runs a session: registration first, then the working session.
3. **Registration** (`registration_process`). If `.token.txt` exists the agent is registered and this step ends at once.
   Otherwise it sends `Registration` (unless `.probation` already exists) and waits: `RegistrationProbation` creates
   `.probation`; `RegistrationAccepted` carries the token hash, which is saved to `.token.txt` before `.probation`
   is removed and `RegistrationFinished` (with that token hash as proof) is sent to the receiver.
4. **Session** (`process/session_process.rs`). A heartbeat task sends a packet every `heart_beat_interval` with the
   agent id, CPU (overall and per core), load average, memory and uptime, carrying the token hash.
   If the stream breaks, the task ends and the connector reconnects.
5. **File overwatch** (`process/fs_scan_process.rs`). Another task, started with each session, rescans a directory
   (the home directory by default) every `watch_interval`, down to `scan_depth` levels, and keeps the result in a
   shared window. It skips unreadable entries and does not follow symlinks into directories.

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
