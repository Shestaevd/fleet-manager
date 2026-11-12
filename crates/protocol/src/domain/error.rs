use fleet_crypto::TokenError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("packet is empty, expected a message type byte")]
    EmptyPacket,
    #[error("token hash has the wrong size")]
    WrongTokenSize,
    #[error("packet body of {size} bytes is over the {max} byte limit")]
    PacketTooLarge { size: usize, max: usize },
    #[error("unknown message type byte 0x{0:02X}")]
    UnknownMessageType(u8),
    #[error("packet is too short")]
    TruncatedPacket,
    #[error("expected message type 0x{expected:02X}, got 0x{actual:02X}")]
    UnexpectedMessageType { expected: u8, actual: u8 },
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("Stream ended unexpectedly {}", source)]
    StreamBroken {
        #[source]
        source: std::io::Error,
    },
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
