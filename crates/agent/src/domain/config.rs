use std::time::Duration;

use fleet_model::AgentId;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgentConfig {
    pub agent_id: AgentId,
    pub receiver_url: Url,
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_handshake_timeout_secs")]
    pub handshake_timeout_secs: u64,
    #[serde(default = "default_backoff_min_secs")]
    pub backoff_min_secs: u64,
    #[serde(default = "default_backoff_max_secs")]
    pub backoff_max_secs: u64,
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: u64,
    /// How many times a failed connect is retried before the agent gives up. `None` retries forever.
    #[serde(default)]
    pub connect_max_retries: Option<u32>,
    #[serde(default = "default_breaker_failure_threshold")]
    pub breaker_failure_threshold: u32,
    #[serde(default = "default_breaker_open_secs")]
    pub breaker_open_secs: u64,
    #[serde(default = "default_scan_depth")]
    pub scan_depth: u8,
    #[serde(default = "default_watch_interval_ms")]
    pub watch_interval_ms: u64,
    #[serde(default = "default_heart_beat_interval_ms")]
    pub heart_beat_interval_ms: u64,
}

impl AgentConfig {
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    pub fn handshake_timeout(&self) -> Duration {
        Duration::from_secs(self.handshake_timeout_secs)
    }

    pub fn backoff_min(&self) -> Duration {
        Duration::from_secs(self.backoff_min_secs)
    }

    pub fn backoff_max(&self) -> Duration {
        Duration::from_secs(self.backoff_max_secs)
    }

    pub fn heart_beat_interval(&self) -> Duration {
        Duration::from_millis(self.heart_beat_interval_ms)
    }

    pub fn watch_interval(&self) -> Duration {
        Duration::from_millis(self.watch_interval_ms)
    }

    pub fn breaker_open_for(&self) -> Duration {
        Duration::from_secs(self.breaker_open_secs)
    }
}

fn default_connect_timeout_secs() -> u64 {
    10
}

fn default_handshake_timeout_secs() -> u64 {
    10
}

fn default_backoff_min_secs() -> u64 {
    1
}

fn default_backoff_max_secs() -> u64 {
    60
}

fn default_backoff_multiplier() -> u64 {
    2
}

fn default_breaker_failure_threshold() -> u32 {
    5
}

fn default_breaker_open_secs() -> u64 {
    30
}

fn default_scan_depth() -> u8 {
    5
}

fn default_watch_interval_ms() -> u64 {
    1000
}

fn default_heart_beat_interval_ms() -> u64 {
    1000
}

pub const CONFIG_FILE_NAME: &str = "agent.toml";

const DEFAULT_CONFIG_TEMPLATE: &str = r#"# Fleet agent configuration.
# Unique id of this agent. Generated when this file was created; keep it stable.
agent_id = "{agent_id}"

# Route to the receiver. 
# I will replace it with a system variable in the future.
receiver_url = "tls://127.0.0.1:7000"

# Connection settings (seconds unless stated otherwise).
connect_timeout_secs = 10
handshake_timeout_secs = 10
backoff_min_secs = 1
backoff_max_secs = 60
backoff_multiplier = 2
# Retries after a failed connect before the agent gives up. Leave commented out to retry forever.
# connect_max_retries = 10
breaker_failure_threshold = 5
breaker_open_secs = 30

# Filesystem overwatch: how many levels to scan and how often to rescan.
scan_depth = 5
watch_interval_ms = 1000

# How often a heart beat with telemetry is sent to the receiver (milliseconds).
heart_beat_interval_ms = 1000
"#;

/// The default config file text, with a freshly generated agent id.
pub fn default_config() -> String {
    DEFAULT_CONFIG_TEMPLATE.replace("{agent_id}", &AgentId::new().0.to_string())
}
