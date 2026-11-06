use fleet_crypto::{TokenError, validate_token_hash};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

/// The hash of an agent's token, as the receiver hands it to the agent to be saved.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenHash {
    hash: Vec<u8>,
}

impl TokenHash {
    /// Fails unless `hash` is exactly `TOKEN_HASH_BYTE_SIZE` bytes.
    pub fn new(hash: Vec<u8>) -> Result<Self, TokenError> {
        validate_token_hash(&hash)?;
        Ok(Self { hash })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.hash
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.hash
    }
}

impl Debug for TokenHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenHash(...)")
    }
}
