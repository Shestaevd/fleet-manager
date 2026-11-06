use crate::domain::error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MessageType {
    Registration = 0xE0,
    RegistrationProbation = 0xE1,
    RegistrationAccepted = 0xE2,
    RegistrationFinished = 0xE3,
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
            0xE0 => Ok(MessageType::Registration),
            0xE1 => Ok(MessageType::RegistrationProbation),
            0xE2 => Ok(MessageType::RegistrationAccepted),
            0xE3 => Ok(MessageType::RegistrationFinished),
            0xF0 => Ok(MessageType::HeartBeat),
            0xF1 => Ok(MessageType::FileMetaUpdate),
            0xF2 => Ok(MessageType::MoveWindow),
            other => Err(ProtocolError::UnknownMessageType(other)),
        }
    }
}
