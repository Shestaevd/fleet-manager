use std::path::PathBuf;
use std::time::Duration;

use fleet_crypto::Secret;

use crate::domain::config::UserConfig;
use crate::domain::error::UserStoreError;
use crate::service::user_store::UserStoreService;

fn service(name: &str, cached_user_num: usize, lifetime: Duration) -> (UserStoreService, PathBuf) {
    let store: PathBuf = std::env::temp_dir().join(format!("fleet-receiver-service-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    let config: UserConfig = UserConfig { cached_user_num, refresh_token_lifetime: lifetime };
    (UserStoreService::with_store(config, store.clone()).unwrap(), store)
}

fn secret(text: &str) -> Secret {
    Secret { content: text.as_bytes().to_vec() }
}

#[tokio::test]
async fn a_user_is_added_hashed_and_authenticated_with_a_session_token() {
    let (mut users, store) = service("auth", 4, Duration::from_secs(3600));

    users.add_user("alice", &secret("hunter2")).await.unwrap();
    assert!(matches!(users.add_user("alice", &secret("other")).await, Err(UserStoreError::AlreadyExists)));
    assert!(matches!(users.add_user("../evil", &secret("x")).await, Err(UserStoreError::InvalidLogin)));

    let pwd: String = std::fs::read_to_string(store.join("alice").join(".pwd")).unwrap();
    assert!(!pwd.contains("hunter2"));
    assert!(hex::decode(&pwd).unwrap().starts_with(b"$argon2id$"));

    assert!(matches!(users.authenticate("alice", &secret("wrong")).await, Err(UserStoreError::InvalidCredentials)));
    assert!(matches!(users.authenticate("nobody", &secret("hunter2")).await, Err(UserStoreError::NoSuchUser)));

    let token = users.authenticate("alice", &secret("hunter2")).await.unwrap();
    assert!(store.join("alice").join("session_token").exists());
    assert_eq!(token.token.content.len(), fleet_crypto::TOKEN_BYTE_SIZE);

    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test]
async fn a_token_is_validated_once_and_replaced_by_a_new_one() {
    let (mut users, store) = service("token", 4, Duration::from_secs(3600));
    users.add_user("bob", &secret("pw")).await.unwrap();
    let first = users.authenticate("bob", &secret("pw")).await.unwrap();

    assert!(matches!(users.validate_token("bob", &secret("guess")).await, Err(UserStoreError::InvalidToken)));
    assert!(matches!(users.validate_token("nobody", &first.token).await, Err(UserStoreError::InvalidToken)));

    let second = users.validate_token("bob", &first.token).await.unwrap();
    assert_ne!(second.token, first.token);
    assert!(matches!(users.validate_token("bob", &first.token).await, Err(UserStoreError::InvalidToken)));
    users.validate_token("bob", &second.token).await.unwrap();

    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test]
async fn an_expired_token_is_refused_and_a_token_survives_a_restart() {
    let (mut users, store) = service("expiry", 4, Duration::from_millis(1));
    users.add_user("carol", &secret("pw")).await.unwrap();
    let token = users.authenticate("carol", &secret("pw")).await.unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(matches!(users.validate_token("carol", &token.token).await, Err(UserStoreError::InvalidToken)));

    let config: UserConfig = UserConfig { cached_user_num: 4, refresh_token_lifetime: Duration::from_secs(3600) };
    let mut users: UserStoreService = UserStoreService::with_store(config.clone(), store.clone()).unwrap();
    let token = users.authenticate("carol", &secret("pw")).await.unwrap();
    let mut restarted: UserStoreService = UserStoreService::with_store(config, store.clone()).unwrap();
    restarted.validate_token("carol", &token.token).await.unwrap();

    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test]
async fn the_cache_holds_at_most_the_configured_number_of_users_and_removing_a_user_deletes_it() {
    let (mut users, store) = service("cache", 2, Duration::from_secs(3600));
    for login in ["u1", "u2", "u3"] {
        users.add_user(login, &secret("pw")).await.unwrap();
    }
    for login in ["u1", "u2", "u3"] {
        users.authenticate(login, &secret("pw")).await.unwrap();
    }
    assert_eq!(users.cached(), 2);

    users.remove_user("u3").await.unwrap();
    assert_eq!(users.cached(), 1);
    assert!(!store.join("u3").exists());
    assert!(matches!(users.authenticate("u3", &secret("pw")).await, Err(UserStoreError::NoSuchUser)));
    users.remove_user("u3").await.unwrap();
    assert!(matches!(users.remove_user("../evil").await, Err(UserStoreError::InvalidLogin)));

    let _ = std::fs::remove_dir_all(&store);
}
