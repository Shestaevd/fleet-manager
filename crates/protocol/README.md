# fleet-protocol

Wire protocol shared by the agent, the receiver and the client. The two links have separate message type maps:

- `agent_receiver`: between an agent and the receiver
- `receiver_client`: between the receiver and an operator client (placeholder for now)

## Packet format

```
[ 4 bytes length ][ 1 byte message type ][ payload ]
```

| Part | Size | Notes |
|---|---|---|
| length | 4 bytes | big-endian; counts everything after it (type and payload)|
| message type | 1 byte | identifies the message, see the table below |
| payload | the rest | the message serialized with `postcard` |

`Package::encode` builds this layout.

## Message types (`agent_receiver`)

| Byte | Message |
|---|---|
| `0xF0` | `HeartBeat`: agent id, CPU (overall and per core), load average, memory, uptime |
| `0xF1` | `FileMetaUpdate` not implemented |
| `0xF2` | `MoveWindow`  not implemented |

Only `HeartBeat` has a message struct so far.
