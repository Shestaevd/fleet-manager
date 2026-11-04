use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("packet is empty, expected a message type byte")]
    EmptyPacket,
    #[error("packet is larger than the 4-byte length prefix can describe")]
    PacketTooLarge,
    #[error("unknown message type byte 0x{0:02X}")]
    UnknownMessageType(u8),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("can not decode message: {source}")]
    Decode {
        #[source]
        source: postcard::Error,
    },
}