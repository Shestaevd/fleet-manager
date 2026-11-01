use rand::distr::Alphanumeric;
use rand::{RngExt, rng};
pub use domain::model::*;
use crate::domain::error::{CertError, TokenError};

pub struct GeneratedCerts {
    pub cert_pem: Secret,
    pub key_pem: Secret,
}

pub fn generate_certs(subject_alt_names: &[String]) -> Result<GeneratedCerts, CertError> {
    if subject_alt_names.is_empty() {
        return Err(CertError::NoSubjectNames);
    }
    let certified = rcgen::generate_simple_self_signed(subject_alt_names.to_vec())
        .map_err(|err| CertError::Generate {
            source: Box::new(err),
        })?;
    Ok(GeneratedCerts {
        cert_pem: Secret { content: certified.cert.pem().into_bytes() },
        key_pem: Secret { content: certified.signing_key.serialize_pem().into_bytes() },
    })
}

pub const TOKEN_BYTE_SIZE: usize = 1366;

pub fn generate_secure_token() -> Vec<u8> {
    rng().sample_iter(Alphanumeric).take(TOKEN_BYTE_SIZE).collect()
}

pub fn validate_token(token: &[u8]) -> Result<(), TokenError> {
    if token.len() == TOKEN_BYTE_SIZE {
        Ok(())
    } else {
        Err(TokenError::WrongSize {
            expected: TOKEN_BYTE_SIZE,
            actual: token.len(),
        })
    }
}

#[cfg(test)]
mod test;
mod domain;

