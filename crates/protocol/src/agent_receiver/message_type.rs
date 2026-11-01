use crate::domain::error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MessageType {
    HeartBeat = 0xF0,
    FileMetaUpdate = 0xF1,
    MoveWindow = 0xF2,
}

impl MessageType {
    pub const fn as_byte(self) -> u8 {
        self as u8
    }
}

impl From<MessageType> for u8 {
    fn from(message_type: MessageType) -> u8 {
        message_type.as_byte()
    }
}

impl TryFrom<u8> for MessageType {
    type Error = ProtocolError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0xF0 => Ok(MessageType::HeartBeat),
            0xF1 => Ok(MessageType::FileMetaUpdate),
            0xF2 => Ok(MessageType::MoveWindow),
            other => Err(ProtocolError::UnknownMessageType(other)),
        }
    }
}
