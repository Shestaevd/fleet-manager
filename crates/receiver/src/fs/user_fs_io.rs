use std::io::ErrorKind;
use std::path::PathBuf;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use fleet_crypto::Secret;
use fleet_model::error::FsError;
use fleet_util::fs::{create_dir_if_not_exist, executable_dir, read_file, write_secret_file};

use crate::domain::error::UserStoreError;
use crate::domain::model::{SessionToken, User};
use crate::fs::fs_io::{read_optional, remove_dir_if_exists};

const USERS_STORE_DIR: &str = "users";
const USER_PASSWORD: &str = ".pwd";
const USER_SESSION_TOKEN: &str = "session_token";
const MAX_LOGIN_LENGTH: usize = 64;

pub fn user_store_dir() -> Result<PathBuf, UserStoreError> {
    Ok(executable_dir()?.join(USERS_STORE_DIR))
}

pub async fn read_user(login: &str) -> Result<Option<User>, UserStoreError> {
    read_user_from(executable_dir()?.join(USERS_STORE_DIR), login).await
}

pub async fn read_user_from(store: PathBuf, login: &str) -> Result<Option<User>, UserStoreError> {
    validate_login(login)?;
    let path: PathBuf = store.join(login).join(USER_PASSWORD);

    let text: Vec<u8> = match read_file(path.clone()).await {
        Ok(text) => text,
        Err(FsError::CanNotRead { source, .. }) if source.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    let hash: Vec<u8> = hex::decode(text.trim_ascii())
        .map_err(|err| UserStoreError::Corrupt { path: path.clone(), reason: err.to_string() })?;
    
    if hash.is_empty() {
        return Err(UserStoreError::Corrupt { path, reason: "the password hash is empty".to_owned() });
    }
    let session_token: Option<SessionToken> =
        read_session_token(store.join(login).join(USER_SESSION_TOKEN)).await.unwrap_or_else(|err| {
            tracing::warn!("ignoring the session token of {login}: {err}");
            None
        });
    Ok(Some(User { login: login.to_owned(), password: Secret { content: hash }, session_token }))
}

pub async fn write_user(user: &User) -> Result<(), UserStoreError> {
    write_user_to(executable_dir()?.join(USERS_STORE_DIR), user).await
}

pub async fn write_user_to(store: PathBuf, user: &User) -> Result<(), UserStoreError> {
    validate_login(&user.login)?;
    let dir: PathBuf = store.join(&user.login);
    create_dir_if_not_exist(dir.clone()).await?;
    write_secret_file(dir.join(USER_PASSWORD), hex::encode(&user.password.content).as_bytes()).await?;
    Ok(())
}

pub async fn write_session_token(login: &str, session_token: &SessionToken) -> Result<(), UserStoreError> {
    write_session_token_to(executable_dir()?.join(USERS_STORE_DIR), login, session_token).await
}

pub async fn write_session_token_to(
    store: PathBuf,
    login: &str,
    session_token: &SessionToken,
) -> Result<(), UserStoreError> {
    validate_login(login)?;
    let path: PathBuf = store.join(login).join(USER_SESSION_TOKEN);
    let expires_at: String = session_token
        .expires_at
        .format(&Rfc3339)
        .map_err(|err| UserStoreError::Corrupt { path: path.clone(), reason: err.to_string() })?;

    let mut content: Vec<u8> = format!("{expires_at}:").into_bytes();
    content.extend_from_slice(&session_token.token.content);
    write_secret_file(path, &content).await?;
    Ok(())
}

async fn read_session_token(path: PathBuf) -> Result<Option<SessionToken>, UserStoreError> {
    let Some(bytes) = read_optional(path.clone()).await? else {
        return Ok(None);
    };
    
    let corrupt = |reason: String| UserStoreError::Corrupt { path: path.clone(), reason };

    let text: &str = std::str::from_utf8(bytes.trim_ascii()).map_err(|err| corrupt(err.to_string()))?;
    let (date, token): (&str, &str) = text.rsplit_once(':').ok_or_else(|| corrupt("expected date:token".to_owned()))?;
    if token.is_empty() {
        return Err(corrupt("the session token is empty".to_owned()));
    }
    let expires_at: OffsetDateTime = OffsetDateTime::parse(date, &Rfc3339).map_err(|err| corrupt(err.to_string()))?;
    Ok(Some(SessionToken { expires_at, token: Secret { content: token.as_bytes().to_vec() } }))
}

pub async fn remove_user_from(store: PathBuf, login: &str) -> Result<(), UserStoreError> {
    validate_login(login)?;
    let dir: PathBuf = store.join(login);
    Ok(remove_dir_if_exists(dir).await?)
}

fn validate_login(login: &str) -> Result<(), UserStoreError> {
    let allowed: bool = login.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if login.is_empty() || login.len() > MAX_LOGIN_LENGTH || login.starts_with('.') || !allowed {
        return Err(UserStoreError::InvalidLogin);
    }
    Ok(())
}
