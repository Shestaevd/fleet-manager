use std::path::PathBuf;

use fleet_model::AgentId;
use fleet_model::error::FsError;
use fleet_protocol::ProtocolError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentStoreError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error("agent {0} is banned")]
    Banned(AgentId),
    #[error("agent {0} already exists")]
    AlreadyExists(AgentId),
    #[error("agent {0} is not known")]
    NotFound(AgentId),
    #[error("agent {0} is already registered")]
    AlreadyRegistered(AgentId),
    #[error("file {} of an agent is corrupt: {reason}", path.display())]
    Corrupt { path: PathBuf, reason: String },
}

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error("invalid login: letters, digits, '_', '-' and '.' only, 1 to 64 characters, not starting with '.'")]
    InvalidLogin,
    #[error("user already exists")]
    AlreadyExists,
    #[error("no such user")]
    NoSuchUser,
    #[error("wrong login or password")]
    InvalidCredentials,
    #[error("the session token is wrong or expired")]
    InvalidToken,
    #[error(transparent)]
    Password(#[from] fleet_crypto::PasswordError),
    #[error("background task failed: {0}")]
    Task(String),
    #[error("file {} of a user is corrupt: {reason}", path.display())]
    Corrupt { path: PathBuf, reason: String },
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error(transparent)]
    Store(#[from] AgentStoreError),
    #[error("agent {0} was not verified")]
    Rejected(AgentId),
}
