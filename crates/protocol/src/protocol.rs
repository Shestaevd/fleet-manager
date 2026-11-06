use crate::domain::error::{ParseError, ProtocolError};
use fleet_crypto::validate_token_hash;

// Packets carrying a token hash (agent to receiver, after registration):
// [ 4 bytes length ][ 1 byte message type ][ token hash: TOKEN_HASH_BYTE_SIZE ][ payload ]
//
// Packets without a token (registration, and everything the receiver sends):
// [ 4 bytes length ][ 1 byte message type ][ payload ]
//
// The length counts everything after it.

const LENGTH_BYTES: usize = 4;

/// Writes `[length][message type][token hash, if any][payload]`.
fn frame(message_type: u8, token_hash: Option<&[u8]>, payload: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let token_hash: &[u8] = token_hash.unwrap_or(&[]);
    let body_len: usize = 1 + token_hash.len() + payload.len();
    let len: u32 = u32::try_from(body_len).map_err(|_| ProtocolError::PacketTooLarge)?;

    let mut bytes: Vec<u8> = Vec::with_capacity(LENGTH_BYTES + body_len);
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.push(message_type);
    bytes.extend_from_slice(token_hash);
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

pub trait Package
where Self: TryFrom<Vec<u8>> + Into<Vec<u8>> + Clone + Sized {
    fn message_type(&self) -> u8;

    fn encode(&self, token_hash: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        validate_token_hash(token_hash).map_err(|_| ProtocolError::WrongTokenSize)?;
        let serialized: Vec<u8> = self.clone().into();
        frame(self.message_type(), Some(token_hash), &serialized)
    }
}

pub trait UnverifiedPackage
where Self: TryFrom<Vec<u8>, Error = ParseError> + Into<Vec<u8>> + Clone + Sized {
    const MESSAGE_TYPE: u8;

    fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let serialized: Vec<u8> = self.clone().into();
        frame(Self::MESSAGE_TYPE, None, &serialized)
    }

    fn decode(packet: &[u8]) -> Result<Self, ProtocolError> {
        let Some((len_bytes, body)) = packet.split_first_chunk::<LENGTH_BYTES>() else {
            return Err(ProtocolError::TruncatedPacket);
        };
        let declared: usize = u32::from_be_bytes(*len_bytes) as usize;
        if declared != body.len() {
            return Err(ProtocolError::LengthMismatch { declared, actual: body.len() });
        }
        let Some((&actual, payload)) = body.split_first() else {
            return Err(ProtocolError::EmptyPacket);
        };
        if actual != Self::MESSAGE_TYPE {
            return Err(ProtocolError::UnexpectedMessageType { expected: Self::MESSAGE_TYPE, actual });
        }
        Ok(Self::try_from(payload.to_vec())?)
    }
}

/// Implements the `postcard` conversions every plain message needs.
macro_rules! postcard_message {
    ($message:ty) => {
        impl TryFrom<Vec<u8>> for $message {
            type Error = $crate::domain::error::ParseError;

            fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
                postcard::from_bytes::<$message>(&bytes)
                    .map_err(|source| $crate::domain::error::ParseError::Decode { source })
            }
        }

        impl From<$message> for Vec<u8> {
            fn from(message: $message) -> Vec<u8> {
                postcard::to_stdvec(&message).expect("serializing a plain struct to a Vec can not fail")
            }
        }
    };
}
pub(crate) use postcard_message;
