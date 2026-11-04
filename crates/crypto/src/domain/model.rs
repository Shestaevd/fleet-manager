use std::fmt::{Debug, Display, Formatter};

/// Secret bytes (a certificate or a key)
pub struct Secret {
    pub content: Vec<u8>,
}

impl Display for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "...")
    }
}

impl Debug for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "...")
    }
}
