# fleet-protocol

Shared library used by agent, receiver and client.

Planned contents:
- Message types (registration, auth token, telemetry, command, file transfer, errors)
- Length-prefixed frame codec (`tokio-util`)
- `postcard` serialization, request-ID correlation types

Status: scaffold only.
