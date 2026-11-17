# fleet-receiver

Broker between agents and operators. It keeps what it knows about agents and users on disk next to the binary.

## What works

- Two TLS listeners (`tcp/server.rs`): one for agents (`[server]`) and one for operators (`[clients]`). The operator
  listener is still a stub.
- Agent verification (`process/session_process.rs`): the first messages of a connection register a new agent on
  probation, hand a token to an agent approved while offline, or check the token and MAC address of a registered agent.
  A verified agent is then held by a listening task that does nothing yet.
- Agent store (`fs/agent_fs_io.rs`, `service/agent_store.rs`): `agents/<agent id>/` with `.token`, `status`, `mac`,
  `.banned`. An agent is on probation, registered or banned.
- User store (`fs/user_fs_io.rs`, `service/user_store.rs`): `users/<login>/` with `.pwd` (Argon2id hash as hex) and
  `session_token` (`{expiry}:{token}`). A cache of the most recently used users, `add_user`, `authenticate`,
  `validate_token` (issues a new token) and `remove_user`.

## Config

`receiver.toml` is created next to the binary on the first run. Sections: `[server]` (agents), `[agents]`, `[clients]`
(operators) and `[users]` (`cached_user_num`, `refresh_token_lifetime`). All durations are in milliseconds.
The server needs `self_sign.pem` and `self_sign.key` (paths in the config).

## Not done

Operator login flow and commands, admin rights, approving agents (no operator side yet), graceful shutdown,
enforcing `agent_limit`.
