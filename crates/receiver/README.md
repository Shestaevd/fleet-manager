# fleet-receiver

Broker between agents and operators: it admits agents, keeps track of them, and will pass operator commands on to
them.

## What works

- Two TLS listeners, one for agents and one for operators. The operator side is a stub.
- Admission of agents: a new agent goes on probation, an agent approved while offline gets its token when it connects
  again, and a registered agent is checked by its token and MAC address. A verified connection is held open; nothing
  is done with it yet.
- Users for the operator side: login with a password, session tokens that replace the password while they are valid,
  adding and removing users.

## Storage

Kept next to the binary:

- `agents/<agent id>/`: `.token`, `status`, `mac` and `.banned`. An agent without a `.token` is on probation.
- `users/<login>/`: `.pwd` (Argon2id hash, hex) and `session_token` (`{expiry}:{token}`). The most recently used
  users are cached in memory (`cached_user_num`).

## Config

`receiver.toml` is created on the first run. Sections: `[server]` (agents), `[agents]`, `[clients]` (operators) and
`[users]` (`cached_user_num`, `refresh_token_lifetime`). All durations are in milliseconds. The listeners need
`self_sign.pem` and `self_sign.key` (paths in the config).

## Not done

Operator login flow and commands, admin rights, approving agents (no operator side yet), graceful shutdown,
enforcing `agent_limit`.
