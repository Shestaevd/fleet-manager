# fleet-agent

Background service installed on each managed machine (Linux only, systemd unit).

Planned responsibilities:
- Maintain a TLS (token-authenticated) connection to the receiver
- Report telemetry: OS, CPU, memory, uptime
- Execute one-shot commands on request
- Upload/download files in chunks on request
- Install/uninstall itself as a service with a configurable display name
