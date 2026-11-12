use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use serde_with::{DurationMilliSeconds, serde_as};

pub const CONFIG_FILE_NAME: &str = "receiver.toml";

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct ReceiverConfig {
    #[serde(default, rename = "server")]
    pub server_config: ServerConfig,
    #[serde(default, rename = "agents")]
    pub agents_config: AgentsConfig,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_handshake_timeout")]
    pub handshake_timeout: Duration,
    #[serde(default = "default_cert_path")]
    pub cert_path: PathBuf,
    #[serde(default = "default_key_path")]
    pub key_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgentsConfig {
    #[serde(default = "default_agent_store_path")]
    pub agent_store_path: PathBuf,
    #[serde(default = "default_agent_limit")]
    pub agent_limit: u8,
    #[serde(default = "default_agent_store_size_bytes")]
    pub agent_store_size_bytes: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            max_connections: default_max_connections(),
            handshake_timeout: default_handshake_timeout(),
            cert_path: default_cert_path(),
            key_path: default_key_path(),
        }
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            agent_store_path: default_agent_store_path(),
            agent_limit: default_agent_limit(),
            agent_store_size_bytes: default_agent_store_size_bytes(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_port() -> u16 {
    7000
}

fn default_max_connections() -> usize {
    1_024
}

fn default_handshake_timeout() -> Duration {
    Duration::from_millis(10_000)
}

fn default_cert_path() -> PathBuf {
    PathBuf::from("self_sign.pem")
}

fn default_key_path() -> PathBuf {
    PathBuf::from("self_sign.key")
}

fn default_agent_store_path() -> PathBuf {
    PathBuf::from(".agents")
}

fn default_agent_limit() -> u8 {
    100
}

fn default_agent_store_size_bytes() -> usize {
    1_073_741_824
}

/// The text of a new `receiver.toml`. Every value in it is the default, so deleting a line changes nothing.
pub const DEFAULT_CONFIG: &str = r#"# Fleet receiver configuration. Every duration is in milliseconds.

[server]
# Address and port agents connect to.
host = "127.0.0.1"
port = 7000
# How many connections are served at once; more are dropped.
max_connections = 1024
# How long a client has to finish the TLS handshake.
handshake_timeout = 10000
# The receiver's certificate and private key. A relative path is relative to the binary's directory.
cert_path = "self_sign.pem"
key_path = "self_sign.key"

[agents]
# Where the receiver keeps what it knows about each agent.
agent_store_path = ".agents"
# How many agents (verified and on probation) the receiver accepts.
agent_limit = 100
# How many bytes of files cached from agents may be kept (1 GiB).
agent_store_size_bytes = 1073741824
"#;
