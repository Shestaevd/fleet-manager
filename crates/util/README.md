# fleet-util

Small helpers shared by the agent and the receiver.

- `fs`: async file helpers on `tokio::fs` with `FsError` errors: `read_file`, `write_file`, `write_secret_file` (owner-only),
  `create_dir_if_not_exist` and `executable_dir`
