use std::path::PathBuf;

use fleet_crypto::TOKEN_HASH_BYTE_SIZE;

use crate::domain::error::AgentError;
use crate::fs::fs_io::{read_token_hash_from, write_token_hash_to};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("fleet-agent-{}-{name}", std::process::id()))
}

#[tokio::test]
async fn token_hash_survives_a_write_and_a_read_and_is_stored_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let path = temp_path("token.hash");
    let hash: Vec<u8> = (0..TOKEN_HASH_BYTE_SIZE as u8).collect();

    write_token_hash_to(path.clone(), &hash).await.unwrap();
    assert_eq!(read_token_hash_from(path.clone()).await.unwrap(), hash);
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn a_token_hash_of_the_wrong_size_is_rejected_when_writing_and_when_reading() {
    let path = temp_path("wrong-size.hash");

    let written = write_token_hash_to(path.clone(), &[1, 2, 3]).await;
    assert!(matches!(written, Err(AgentError::TokenHash(_))));
    assert!(!path.exists());

    std::fs::write(&path, [1u8, 2, 3]).unwrap();
    assert!(matches!(read_token_hash_from(path.clone()).await, Err(AgentError::TokenHash(_))));
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn a_missing_token_hash_file_is_a_file_system_error() {
    let result = read_token_hash_from(temp_path("does-not-exist.hash")).await;
    assert!(matches!(result, Err(AgentError::Fs(_))));
}
