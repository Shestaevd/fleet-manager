pub use domain::model::*;
use crate::domain::error::CertError;

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

#[cfg(test)]
mod test;
mod domain;

