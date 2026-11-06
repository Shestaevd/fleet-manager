use fleet_crypto::TokenError;
use fleet_model::error::FsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error("invalid token hash: {0}")]
    TokenHash(#[from] TokenError),
}
