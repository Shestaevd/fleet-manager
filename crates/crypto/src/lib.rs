use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::distr::Alphanumeric;
use rand::{RngExt, rng};
use rcgen::{CertifiedKey, KeyPair};
use sha2::{Digest, Sha256};

pub use domain::error::{CertError, PasswordError, TokenError};
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

/// Hashes a password with Argon2id and a fresh random salt. The result is the PHC string
/// (`$argon2id$v=19$m=...,t=...,p=...$salt$hash`) as bytes, so it carries its own salt and parameters.
pub fn hash_password(password: &[u8]) -> Result<Vec<u8>, PasswordError> {
    let mut salt_bytes: [u8; 16] = [0u8; 16];
    rng().fill(&mut salt_bytes);
    let salt: SaltString = SaltString::encode_b64(&salt_bytes).map_err(|err| PasswordError::Hash(err.to_string()))?;
    let hash = Argon2::default()
        .hash_password(password, &salt)
        .map_err(|err| PasswordError::Hash(err.to_string()))?;
    Ok(hash.to_string().into_bytes())
}

/// Whether `password` matches `hash`, a value made by `hash_password`. A hash that can not be read is no match.
pub fn verify_password(password: &[u8], hash: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(hash) else {
        return false;
    };
    let Ok(parsed) = PasswordHash::new(text) else {
        return false;
    };
    Argon2::default().verify_password(password, &parsed).is_ok()
}

/// Compares two byte strings in time that depends only on their length.
pub fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len() && left.iter().zip(right).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
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

