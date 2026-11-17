use std::path::PathBuf;

use fleet_crypto::Secret;

use crate::domain::error::UserStoreError;
use crate::domain::model::User;
use std::time::Duration;
use time::OffsetDateTime;
use crate::fs::user_fs_io::{read_user_from, write_session_token_to, write_user_to};

fn store(name: &str) -> PathBuf {
    let path: PathBuf = std::env::temp_dir().join(format!("fleet-receiver-users-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[tokio::test]
async fn a_user_is_written_as_hex_and_read_back() {
    let store: PathBuf = store("roundtrip");
    assert!(read_user_from(store.clone(), "alice").await.unwrap().is_none());

    let user: User = User { login: "alice".to_owned(), password: Secret { content: vec![0xde, 0xad, 0xbe, 0xef] }, session_token: None };
    write_user_to(store.clone(), &user).await.unwrap();
    assert_eq!(std::fs::read_to_string(store.join("alice").join(".pwd")).unwrap(), "deadbeef");

    let read: User = read_user_from(store.clone(), "alice").await.unwrap().unwrap();
    assert_eq!(read.login, "alice");
    assert_eq!(read.password.content, user.password.content);

    let changed: User = User { login: "alice".to_owned(), password: Secret { content: vec![1, 2] }, session_token: None };
    write_user_to(store.clone(), &changed).await.unwrap();
    assert_eq!(read_user_from(store.clone(), "alice").await.unwrap().unwrap().password.content, vec![1, 2]);

    std::fs::write(store.join("alice").join(".pwd"), "not hex").unwrap();
    assert!(matches!(read_user_from(store.clone(), "alice").await, Err(UserStoreError::Corrupt { .. })));

    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test]
async fn a_login_can_not_leave_the_store() {
    let store: PathBuf = store("login");
    for login in ["", ".", "..", "../evil", "a/b", ".hidden", "with space", &"x".repeat(65)] {
        let user: User = User { login: login.to_owned(), password: Secret { content: vec![1] }, session_token: None };
        assert!(matches!(write_user_to(store.clone(), &user).await, Err(UserStoreError::InvalidLogin)), "{login:?}");
        assert!(matches!(read_user_from(store.clone(), login).await, Err(UserStoreError::InvalidLogin)), "{login:?}");
    }
    assert!(!store.exists());
}

#[tokio::test]
async fn a_session_token_is_refreshed_written_as_date_colon_token_and_read_back() {
    let store: PathBuf = store("session");
    let mut user: User = User { login: "bob".to_owned(), password: Secret { content: vec![1] }, session_token: None };
    write_user_to(store.clone(), &user).await.unwrap();
    assert!(read_user_from(store.clone(), "bob").await.unwrap().unwrap().session_token.is_none());

    let first = user.refresh_session_token(Duration::from_secs(3600)).clone();
    assert_eq!(first.token.content.len(), fleet_crypto::TOKEN_BYTE_SIZE);
    assert!(first.is_active(OffsetDateTime::now_utc()));
    assert!(!first.is_active(first.expires_at));
    assert_ne!(user.refresh_session_token(Duration::from_secs(3600)).token, first.token);

    let token = user.session_token.clone().unwrap();
    write_session_token_to(store.clone(), "bob", &token).await.unwrap();

    let text: String = std::fs::read_to_string(store.join("bob").join("session_token")).unwrap();
    let (date, written): (&str, &str) = text.rsplit_once(':').unwrap();
    assert!(date.ends_with('Z') && date.contains('T'));
    assert_eq!(written.as_bytes(), token.token.content.as_slice());

    let read: User = read_user_from(store.clone(), "bob").await.unwrap().unwrap();
    assert_eq!(read.session_token, Some(token));

    for broken in ["no colon here", "yesterday:abc", "2030-01-01T00:00:00Z:", ""] {
        std::fs::write(store.join("bob").join("session_token"), broken).unwrap();
        let read: User = read_user_from(store.clone(), "bob").await.unwrap().unwrap();
        assert!(read.session_token.is_none(), "{broken:?}");
        assert_eq!(read.password.content, vec![1]);
    }

    let _ = std::fs::remove_dir_all(&store);
}
