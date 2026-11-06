# fleet-protocol

Wire protocol shared by the agent, the receiver and the client. The two links have separate message type maps:

- `agent_receiver`: between an agent and the receiver
- `receiver_client`: between the receiver and an operator client (placeholder for now)

## Two kinds of packets

Packets with a token hash (`Package`, agent to receiver, after registration):

```
[ 4 bytes length ][ 1 byte message type ][ token hash ][ payload ]
```

Packets without a token (`UnverifiedPackage`, registration and everything the receiver sends):

```
[ 4 bytes length ][ 1 byte message type ][ payload ]
```

| Part | Size | Notes |
|---|---|---|
| length | 4 bytes | big-endian; counts everything after it |
| message type | 1 byte | identifies the message, see the table below |
| token hash | `TOKEN_HASH_BYTE_SIZE` bytes | only in `Package` packets. The size is a constant in `fleet-crypto`; it is not sent, so both sides must be built with the same value |
| payload | the rest | the message serialized with `postcard` |

`Package::encode(token_hash)` builds the first kind. `UnverifiedPackage::encode()` and `UnverifiedPackage::decode(packet)`
build and read the second; `decode` checks the length, the message type and the payload.

## Message types (`agent_receiver`)

| Byte | Message | Direction | Token |
|---|---|---|---|
| `0xE0` | `Registration`: agent id and package id; asks to be admitted | agent to receiver | no |
| `0xE1` | `RegistrationProbation`: the request id it answers; the agent waits for a decision | receiver to agent | no |
| `0xE2` | `RegistrationAccepted`: the request id and the `TokenHash` to save | receiver to agent | no |
| `0xE3` | `RegistrationFinished`: agent id and package id; sent with the saved token hash once it is on disk | agent to receiver | yes |
| `0xF0` | `HeartBeat`: agent id, CPU (overall and per core), load average, memory, uptime | agent to receiver | yes |
| `0xF1` | `FileMetaUpdate`, not implemented | agent to receiver | yes |
| `0xF2` | `MoveWindow`, not implemented | | |

Messages the receiver sends implement `ReceiverAgentPackage` (a `request_id`). Messages an agent sends implement
`AgentReceiverPackage` (an `agent_id` and a `package_id`). `0x00` is never a valid message type.
