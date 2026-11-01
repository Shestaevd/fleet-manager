use std::path::{Path, PathBuf};

use crate::read_config_from;
use fleet_model::ConfigReadingError;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    port: u16,
}

fn write(name: &str, body: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("fleet-config-{}-{name}", std::process::id()));
    std::fs::write(&p, body).unwrap();
    p
}

#[test]
fn parses_valid() {
    let p = write("ok.toml", "name = \"a\"\nport = 1\n");
    let v: TestConfig = read_config_from(&p).unwrap();
    assert_eq!(
        v,
        TestConfig {
            name: "a".into(),
            port: 1
        }
    );
}

#[test]
fn missing_file() {
    let r = read_config_from::<TestConfig>(Path::new("/nonexistent/fleet.toml"));
    assert!(matches!(r, Err(ConfigReadingError::Read { .. })));
}

#[test]
fn bad_toml() {
    let p = write("bad.toml", "name = ");
    let r = read_config_from::<TestConfig>(&p);
    assert!(matches!(r, Err(ConfigReadingError::Parse { .. })));
}

#[test]
fn creates_default_file_once_and_keeps_existing() {
    use crate::read_config_or_create_at;

    let path = std::env::temp_dir().join(format!("fleet-config-{}-create.toml", std::process::id()));
    let _ = std::fs::remove_file(&path);

    let (first, created): (TestConfig, bool) =
        read_config_or_create_at(&path, "name = \"a\"\nport = 1\n").unwrap();
    assert!(created);
    assert_eq!(first, TestConfig { name: "a".into(), port: 1 });

    let (second, created): (TestConfig, bool) =
        read_config_or_create_at(&path, "name = \"other\"\nport = 2\n").unwrap();
    assert!(!created);
    assert_eq!(second, first);
    let _ = std::fs::remove_file(&path);
}
