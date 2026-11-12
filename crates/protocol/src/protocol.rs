use crate::domain::error::{ParseError, ProtocolError};
use fleet_crypto::{Secret, TOKEN_HASH_BYTE_SIZE, validate_token_hash};

// Packets carrying a token hash (agent to receiver, after registration):
// [ 4 bytes length ][ 1 byte message type ][ token hash: TOKEN_HASH_BYTE_SIZE ][ payload ]
//
// Packets without a token (registration, and everything the receiver sends):
// [ 4 bytes length ][ 1 byte message type ][ payload ]
//
// The length counts everything after it. The agent's MAC address is a field of its messages, not part of the header.

const LENGTH_BYTES: usize = 4;
pub const MAX_PACKET_SIZE: usize = 64 * 1024;

/// Writes `[length][message type][token hash, if any][payload]`.
fn frame(message_type: u8, token_hash: Option<&[u8]>, payload: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let token_hash: &[u8] = token_hash.unwrap_or(&[]);
    let body_len: usize = 1 + token_hash.len() + payload.len();
    if body_len > MAX_PACKET_SIZE {
        return Err(ProtocolError::PacketTooLarge { size: body_len, max: MAX_PACKET_SIZE });
    }
    let len: u32 = body_len as u32;

    let mut bytes: Vec<u8> = Vec::with_capacity(LENGTH_BYTES + body_len);
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.push(message_type);
    bytes.extend_from_slice(token_hash);
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

fn open(mut body: Vec<u8>, expected: u8) -> Result<Vec<u8>, ProtocolError> {
    if body.len() > MAX_PACKET_SIZE {
        return Err(ProtocolError::PacketTooLarge { size: body.len(), max: MAX_PACKET_SIZE });
    }
    let Some(&actual) = body.first() else {
        return Err(ProtocolError::EmptyPacket);
    };
    if actual != expected {
        return Err(ProtocolError::UnexpectedMessageType { expected, actual });
    }
    body.drain(..1);
    Ok(body)
}

/// A package containing a token, providing identity check for a message
pub trait Package
where Self: TryFrom<Vec<u8>, Error = ParseError> + Into<Vec<u8>> + Clone + Sized {
    const MESSAGE_TYPE: u8;

    fn encode(&self, token_hash: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        validate_token_hash(token_hash).map_err(|_| ProtocolError::WrongTokenSize)?;
        let serialized: Vec<u8> = self.clone().into();
        frame(Self::MESSAGE_TYPE, Some(token_hash), &serialized)
    }

    fn decode(body: Vec<u8>) -> Result<(Secret, Self), ProtocolError> {
        let mut rest: Vec<u8> = open(body, Self::MESSAGE_TYPE)?;
        if rest.len() < TOKEN_HASH_BYTE_SIZE {
            return Err(ProtocolError::TruncatedPacket);
        }
        let payload: Vec<u8> = rest.split_off(TOKEN_HASH_BYTE_SIZE);
        validate_token_hash(&rest).map_err(|_| ProtocolError::WrongTokenSize)?;
        let token_hash: Secret = Secret { content: rest };
        Ok((token_hash, Self::try_from(payload)?))
    }
}

/// A package without a token, can be sent by anyone
pub trait UnverifiedPackage
where Self: TryFrom<Vec<u8>, Error = ParseError> + Into<Vec<u8>> + Clone + Sized {
    const MESSAGE_TYPE: u8;

    fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let serialized: Vec<u8> = self.clone().into();
        frame(Self::MESSAGE_TYPE, None, &serialized)
    }

    fn decode(body: Vec<u8>) -> Result<Self, ProtocolError> {
        Ok(Self::try_from(open(body, Self::MESSAGE_TYPE)?)?)
    }
}

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
