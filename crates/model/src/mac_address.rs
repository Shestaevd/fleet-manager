use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

/// A hardware address, written `aa:bb:cc:dd:ee:ff`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// All zeros: what a machine without a physical network card reports.
    pub const UNSPECIFIED: MacAddress = MacAddress([0; 6]);

    pub fn is_unspecified(&self) -> bool {
        *self == Self::UNSPECIFIED
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let [a, b, c, d, e, g] = self.0;
        write!(f, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{g:02x}")
    }
}

impl Debug for MacAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MacAddress({self})")
    }
}
