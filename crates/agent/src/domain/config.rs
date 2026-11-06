use std::time::Duration;

use fleet_model::AgentId;
use serde::Deserialize;
use serde_with::{DurationMilliSeconds, serde_as};
use url::Url;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgentConfig {
    pub agent_id: AgentId,
    pub receiver_url: Url,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: Duration,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_handshake_timeout")]
    pub handshake_timeout: Duration,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_backoff_min")]
    pub backoff_min: Duration,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_backoff_max")]
    pub backoff_max: Duration,
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: u64,
    #[serde(default)]
    pub connect_max_retries: Option<u32>,
    #[serde(default = "default_breaker_failure_threshold")]
    pub breaker_failure_threshold: u32,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_breaker_open_for")]
    pub breaker_open_for: Duration,
    #[serde(default = "default_scan_depth")]
    pub scan_depth: u8,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_watch_interval")]
    pub watch_interval: Duration,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[serde(default = "default_heart_beat_interval")]
    pub heart_beat_interval: Duration,
}

fn default_connect_timeout() -> Duration {
    Duration::from_millis(10_000)
}

fn default_handshake_timeout() -> Duration {
    Duration::from_millis(10_000)
}

fn default_backoff_min() -> Duration {
    Duration::from_millis(1_000)
}

fn default_backoff_max() -> Duration {
    Duration::from_millis(60_000)
}

fn default_backoff_multiplier() -> u64 {
    2
}

fn default_breaker_failure_threshold() -> u32 {
    5
}

fn default_breaker_open_for() -> Duration {
    Duration::from_millis(30_000)
}

fn default_scan_depth() -> u8 {
    5
}

fn default_watch_interval() -> Duration {
    Duration::from_millis(1_000)
}

fn default_heart_beat_interval() -> Duration {
    Duration::from_millis(1_000)
}

pub const CONFIG_FILE_NAME: &str = "agent.toml";

const DEFAULT_CONFIG_TEMPLATE: &str = r#"# Fleet agent configuration.
# Unique id of this agent. Generated when this file was created; keep it stable.
agent_id = "{agent_id}"

# Route to the receiver. 
# I will replace it with a system variable in the future.
receiver_url = "tls://127.0.0.1:7000"

# Connection settings. Every duration is in milliseconds.
connect_timeout = 10000
handshake_timeout = 10000
backoff_min = 1000
backoff_max = 60000
backoff_multiplier = 2
# Retries after a failed connect before the agent gives up. Leave commented out to retry forever.
# connect_max_retries = 10
breaker_failure_threshold = 5
breaker_open_for = 30000

# Filesystem overwatch: how many levels to scan and how often to rescan.
scan_depth = 5
watch_interval = 1000

# How often a heart beat with telemetry is sent to the receiver.
heart_beat_interval = 1000
"#;

/// The default config file text, with a freshly generated agent id.
pub fn default_config() -> String {
    DEFAULT_CONFIG_TEMPLATE.replace("{agent_id}", &AgentId::new().0.to_string())
}
