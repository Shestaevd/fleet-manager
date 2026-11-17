use thiserror::Error;

#[derive(Debug, Error)]
pub enum CertError {
    #[error("at least one subject alternative name (DNS name or IP) is required")]
    NoSubjectNames,
    #[error("can not generate certificate: {source}")]
    Generate {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("can not read certificate: {source}")]
    Read{
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("token hash must be exactly {expected} bytes, got {actual}")]
    WrongSize { expected: usize, actual: usize },
}

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("can not hash the password: {0}")]
    Hash(String),
}
