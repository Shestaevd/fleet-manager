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

The length may not exceed `MAX_PACKET_SIZE` (64 KiB); `encode` and `decode` are the only places that check it and return
`PacketTooLarge`.

`Package::encode(token_hash)` and `UnverifiedPackage::encode()` write a whole packet, length prefix included.
`Package::decode(body)` and `UnverifiedPackage::decode(body)` take the packet **without** the length prefix, as an owned
`Vec<u8>`: its length is the size of the message, and the reader has already used the 4 length bytes to know how much
to read. They check the size (`MAX_PACKET_SIZE`), the message type and the payload; `Package::decode` also splits off
the token hash and returns it as a `Secret` together with the message.

## The MAC address

Every message an agent sends has a mandatory `mac_address: MacAddress` field, next to `agent_id` and `package_id`, and
reads it through `AgentReceiverPackage::mac_address()`. A message is built with
`new(agent_id, mac_address)` (`IntoHeartBeat::into_heart_beat(agent_id, mac_address)` for heart beats). The agent reads the
address once at start-up and reuses it. A machine without a physical network card sends all zeros
(`MacAddress::UNSPECIFIED`, `is_unspecified()`), and all such machines look the same.

The address is reported by the agent itself, so it is not proof of identity; it lets the receiver recognise a machine and
later ban it by MAC as well as by id.

## Message types (`agent_receiver`)

| Byte | Message | Direction | Packet kind |
|---|---|---|---|
| `0xA0` | `RegistrationStart`: agent id, package id and MAC address; asks to be admitted | agent to receiver | `UnverifiedPackage` |
| `0xA2` | `AwaitingProbation`: agent id, package id and MAC address; first message of an agent that is on probation | agent to receiver | `UnverifiedPackage` |
| `0xA3` | `Registered`: agent id, package id and MAC address; first message of an agent that already has its token | agent to receiver | `Package` |
| `0xC0` | `HeartBeat`: agent id, package id, MAC address, CPU (overall and per core), load average, memory, uptime | agent to receiver | `Package` |
| `0xC1` | `FileMetaUpdate`, not implemented | agent to receiver | `Package` |
| `0xD0` | `RegistrationReceived`: the request id it answers; the agent waits for a decision | receiver to agent | `UnverifiedPackage` |
| `0xD1` | `RegistrationAccepted`: the request id and the `Secret` to save | receiver to agent | `UnverifiedPackage` |
| `0xD2` | `RegistrationVerified`: the request id of the `Registered` message it answers | receiver to agent | `UnverifiedPackage` |
| `0xF0` | `MoveWindow`, not implemented | receiver to agent | |

Messages the receiver sends implement `ReceiverAgentPackage` (a `request_id`). Messages an agent sends implement
`AgentReceiverPackage` (`agent_id`, `package_id` and `mac_address`). `0x00` is never a valid message type.
