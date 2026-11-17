use std::collections::VecDeque;
use std::path::PathBuf;

use fleet_crypto::{Secret, constant_time_eq, hash_password, verify_password};
use time::OffsetDateTime;

use crate::domain::config::UserConfig;
use crate::domain::error::UserStoreError;
use crate::domain::model::{SessionToken, User};
use crate::fs::user_fs_io::{read_user_from, remove_user_from, user_store_dir, write_session_token_to, write_user_to};


pub struct UserStoreService {
    store: PathBuf,
    config: UserConfig,
    /// The least recently used user is in front, the most recent at the back.
    users: VecDeque<User>,
}

impl UserStoreService {
    /// A service over `<executable dir>/users`.
    pub fn new(config: UserConfig) -> Result<Self, UserStoreError> {
        Self::with_store(config, user_store_dir()?)
    }

    pub fn with_store(config: UserConfig, store: PathBuf) -> Result<Self, UserStoreError> {
        Ok(Self { store, config, users: VecDeque::new() })
    }

    pub async fn add_user(&mut self, login: &str, password: &Secret) -> Result<(), UserStoreError> {
        match self.find_user(login).await {
            Ok(_) => return Err(UserStoreError::AlreadyExists),
            Err(UserStoreError::NoSuchUser) => {}
            Err(err) => return Err(err),
        }
        let hash: Vec<u8> = hash_blocking(password).await?;
        let user: User = User { login: login.to_owned(), password: Secret { content: hash }, session_token: None };
        write_user_to(self.store.clone(), &user).await
    }

    pub async fn authenticate(&mut self, login: &str, password: &Secret) -> Result<SessionToken, UserStoreError> {
        let hash: Vec<u8> = self.find_user(login).await?.password.content.clone();

        let matches: bool = verify_blocking(password, hash).await?;
        if !matches {
            return Err(UserStoreError::InvalidCredentials);
        }
        self.issue_token(login).await
    }

    /// Checks the session token of `login`, which must be the current one and not expired, and replaces it with a
    /// new one that is active for a full lifetime from now. The old token stops working.
    pub async fn validate_token(&mut self, login: &str, token: &Secret) -> Result<SessionToken, UserStoreError> {
        let now: OffsetDateTime = OffsetDateTime::now_utc();
        let valid: bool = match self.find_user(login).await {
            Ok(user) => user
                .session_token
                .as_ref()
                .is_some_and(|session| session.is_active(now) && constant_time_eq(&session.token.content, &token.content)),
            Err(UserStoreError::NoSuchUser) => false,
            Err(err) => return Err(err),
        };
        if !valid {
            return Err(UserStoreError::InvalidToken);
        }
        self.issue_token(login).await
    }

    /// Deletes the user's directory and drops the user from the cache; succeeds if there is no such user.
    pub async fn remove_user(&mut self, login: &str) -> Result<(), UserStoreError> {
        remove_user_from(self.store.clone(), login).await?;
        self.users.retain(|user| user.login != login);
        Ok(())
    }

    #[cfg(test)]
    pub fn cached(&self) -> usize {
        self.users.len()
    }

    /// The user from the cache, or read from disk into the cache (dropping the least recently used user when the
    /// cache is full). `NoSuchUser` if there is no such user.
    async fn find_user(&mut self, login: &str) -> Result<&mut User, UserStoreError> {
        match self.users.iter().position(|user| user.login == login) {
            Some(position) => {
                if let Some(user) = self.users.remove(position) {
                    self.users.push_back(user);
                }
            }
            None => {
                let user: User = read_user_from(self.store.clone(), login).await?.ok_or(UserStoreError::NoSuchUser)?;
                self.users.push_back(user);
                while self.users.len() > self.config.cached_user_num.max(1) {
                    self.users.pop_front();
                }
            }
        }
        self.users.back_mut().ok_or(UserStoreError::NoSuchUser)
    }

    async fn issue_token(&mut self, login: &str) -> Result<SessionToken, UserStoreError> {
        let lifetime = self.config.refresh_token_lifetime;
        let store: PathBuf = self.store.clone();
        let user: &mut User = self.find_user(login).await?;

        let previous: Option<SessionToken> = user.session_token.clone();
        let token: SessionToken = user.refresh_session_token(lifetime).clone();
        if let Err(err) = write_session_token_to(store, login, &token).await {
            user.session_token = previous;
            return Err(err);
        }
        Ok(token)
    }
}

async fn hash_blocking(password: &Secret) -> Result<Vec<u8>, UserStoreError> {
    let password: Vec<u8> = password.content.clone();
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|err| UserStoreError::Task(err.to_string()))?
        .map_err(Into::into)
}

async fn verify_blocking(password: &Secret, hash: Vec<u8>) -> Result<bool, UserStoreError> {
    let password: Vec<u8> = password.content.clone();
    tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|err| UserStoreError::Task(err.to_string()))
}
