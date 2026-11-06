use rand::distr::Alphanumeric;
use rand::{RngExt, rng};
use rcgen::{CertifiedKey, KeyPair};
use sha2::{Digest, Sha256};

pub use domain::error::{CertError, TokenError};
pub use domain::model::*;

pub struct GeneratedCerts {
    pub cert_pem: Secret,
    pub key_pem: Secret,
}

pub fn generate_certs(subject_alt_names: &[String]) -> Result<GeneratedCerts, CertError> {
    if subject_alt_names.is_empty() {
        return Err(CertError::NoSubjectNames);
    }
    let certified: CertifiedKey<KeyPair> = rcgen::generate_simple_self_signed(subject_alt_names.to_vec())
        .map_err(|err| CertError::Generate {
            source: Box::new(err),
        })?;
    Ok(GeneratedCerts {
        cert_pem: Secret { content: certified.cert.pem().into_bytes() },
        key_pem: Secret { content: certified.signing_key.serialize_pem().into_bytes() },
    })
}

pub const TOKEN_BYTE_SIZE: usize = 64;

pub fn generate_secure_token() -> Vec<u8> {
    rng().sample_iter(Alphanumeric).take(TOKEN_BYTE_SIZE).collect()
}

pub fn hash_token(token: &[u8]) -> Vec<u8> {
    Sha256::digest(token).to_vec()
}

pub const TOKEN_HASH_BYTE_SIZE: usize = 32;

pub fn validate_token_hash(token_hash: &[u8]) -> Result<(), TokenError> {
    if token_hash.len() == TOKEN_HASH_BYTE_SIZE {
        Ok(())
    } else {
        Err(TokenError::WrongSize {
            expected: TOKEN_HASH_BYTE_SIZE,
            actual: token_hash.len(),
        })
    }
}

#[cfg(test)]
mod test;
mod domain;

