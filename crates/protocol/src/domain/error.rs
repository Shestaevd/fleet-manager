use fleet_crypto::TokenError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("packet is empty, expected a message type byte")]
    EmptyPacket,
    #[error("token hash has the wrong size")]
    WrongTokenSize,
    #[error("packet is larger than the 4-byte length prefix can describe")]
    PacketTooLarge,
    #[error("unknown message type byte 0x{0:02X}")]
    UnknownMessageType(u8),
    #[error("packet is shorter than its 4-byte length prefix")]
    TruncatedPacket,
    #[error("packet declares {declared} bytes after the length prefix but has {actual}")]
    LengthMismatch { declared: usize, actual: usize },
    #[error("expected message type 0x{expected:02X}, got 0x{actual:02X}")]
    UnexpectedMessageType { expected: u8, actual: u8 },
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("can not decode message: {source}")]
    Decode {
        #[source]
        source: postcard::Error,
    },
    #[error("invalid token hash: {source}")]
    TokenHash {
        #[source]
        source: TokenError,
    },
}