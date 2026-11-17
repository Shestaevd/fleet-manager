use std::path::PathBuf;
use std::time::Duration;

use crate::domain::config::{DEFAULT_CONFIG, ReceiverConfig};

#[test]
fn the_template_parses_and_every_value_in_it_is_the_default() {
    let from_template: ReceiverConfig = toml::from_str(DEFAULT_CONFIG).unwrap();
    assert_eq!(from_template, ReceiverConfig::default());
}

#[test]
fn an_empty_file_gives_the_defaults() {
    let config: ReceiverConfig = toml::from_str("").unwrap();
    assert_eq!(config, ReceiverConfig::default());

    assert_eq!(config.agents_server_config.host, "127.0.0.1");
    assert_eq!(config.agents_server_config.port, 7000);
    assert_eq!(config.agents_server_config.max_connections, 1024);
    assert_eq!(config.agents_server_config.handshake_timeout, Duration::from_millis(10_000));
    assert_eq!(config.agents_server_config.cert_path, PathBuf::from("self_sign.pem"));
    assert_eq!(config.agents_server_config.key_path, PathBuf::from("../../../../target/debug/self_sign.key"));
    assert_eq!(config.agents_config.agent_store_path, PathBuf::from(".agents"));
    assert_eq!(config.agents_config.agent_limit, 100);
    assert_eq!(config.agents_config.agent_store_size_bytes, 1_073_741_824);
    assert_eq!(config.users_config.cached_user_num, 16);
    assert_eq!(config.users_config.refresh_token_lifetime, Duration::from_millis(86_400_000));
}

#[test]
fn a_value_that_is_set_wins_and_the_rest_keep_their_defaults() {
    let config: ReceiverConfig = toml::from_str(
        "[server]\nport = 9000\nhandshake_timeout = 2500\n\n[agents]\nagent_limit = 5\n",
    )
    .unwrap();

    assert_eq!(config.agents_server_config.port, 9000);
    assert_eq!(config.agents_server_config.handshake_timeout, Duration::from_millis(2_500));
    assert_eq!(config.agents_config.agent_limit, 5);

    assert_eq!(config.agents_server_config.host, "127.0.0.1");
    assert_eq!(config.agents_server_config.max_connections, 1024);
    assert_eq!(config.agents_config.agent_store_path, PathBuf::from(".agents"));
}

#[test]
fn a_port_above_255_and_a_wrong_type_are_handled() {
    assert!(toml::from_str::<ReceiverConfig>("[server]\nport = 65535\n").is_ok());
    assert!(toml::from_str::<ReceiverConfig>("[server]\nport = 65536\n").is_err());
    assert!(toml::from_str::<ReceiverConfig>("[agents]\nagent_limit = 256\n").is_err());
    assert!(toml::from_str::<ReceiverConfig>("[clients]\nmax_login_attempts = 256\n").is_err());
    assert!(toml::from_str::<ReceiverConfig>("[server]\nport = \"seven\"\n").is_err());
}
