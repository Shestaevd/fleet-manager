use fleet_config::read_config_from;
use fleet_model::ConfigReadingError;

use crate::domain::config::AgentConfig;

fn write(name: &str, body: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("fleet-agent-{}-{name}", std::process::id()));
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn parses_receiver_url_with_hostname() {
    let p = write("host.toml", "agent_id = \"9d3b8f9e-6f2a-4c1e-9a55-2c7f0a1b3d44\"\nreceiver_url = \"tls://receiver.lan:7000\"\n");
    let cfg: AgentConfig = read_config_from(&p).unwrap();
    assert_eq!(cfg.receiver_url.host_str(), Some("receiver.lan"));
    assert_eq!(cfg.receiver_url.port(), Some(7000));
}

#[test]
fn invalid_receiver_url_is_parse_error() {
    let p = write("bad.toml", "agent_id = \"9d3b8f9e-6f2a-4c1e-9a55-2c7f0a1b3d44\"\nreceiver_url = \"not-an-address\"\n");
    let r = read_config_from::<AgentConfig>(&p);
    assert!(matches!(r, Err(ConfigReadingError::Parse { .. })));
}

#[test]
fn connection_timeouts_default_and_can_be_overridden() {
    let p = write("defaults.toml", "agent_id = \"9d3b8f9e-6f2a-4c1e-9a55-2c7f0a1b3d44\"\nreceiver_url = \"tls://receiver.lan:7000\"\n");
    let cfg: AgentConfig = read_config_from(&p).unwrap();
    assert_eq!(cfg.connect_timeout().as_secs(), 10);
    assert_eq!(cfg.handshake_timeout().as_secs(), 10);
    assert_eq!(cfg.backoff_min().as_secs(), 1);
    assert_eq!(cfg.backoff_max().as_secs(), 60);
    assert_eq!(cfg.backoff_multiplier, 2);
    assert_eq!(cfg.connect_max_retries, None);
    assert_eq!(cfg.breaker_failure_threshold, 5);
    assert_eq!(cfg.breaker_open_for().as_secs(), 30);
    assert_eq!(cfg.scan_depth, 5);
    assert_eq!(cfg.watch_interval().as_millis(), 1000);
    assert_eq!(cfg.heart_beat_interval().as_millis(), 1000);

    let p = write(
        "override.toml",
        "agent_id = \"9d3b8f9e-6f2a-4c1e-9a55-2c7f0a1b3d44\"\nreceiver_url = \"tls://receiver.lan:7000\"\nconnect_timeout_secs = 3\nbackoff_max_secs = 5\nconnect_max_retries = 7\n",
    );
    let cfg: AgentConfig = read_config_from(&p).unwrap();
    assert_eq!(cfg.connect_timeout().as_secs(), 3);
    assert_eq!(cfg.backoff_max().as_secs(), 5);
    assert_eq!(cfg.connect_max_retries, Some(7));
}

#[test]
fn default_config_parses_matches_field_defaults_and_gets_a_unique_agent_id() {
    use crate::domain::config::default_config;

    let full: AgentConfig = toml::from_str(&default_config()).unwrap();
    let minimal: AgentConfig = toml::from_str(&format!(
        "agent_id = \"{}\"\nreceiver_url = \"tls://127.0.0.1:7000\"\n",
        full.agent_id.0
    ))
    .unwrap();
    assert_eq!(full, minimal);

    let other: AgentConfig = toml::from_str(&default_config()).unwrap();
    assert_ne!(full.agent_id, other.agent_id);
}
