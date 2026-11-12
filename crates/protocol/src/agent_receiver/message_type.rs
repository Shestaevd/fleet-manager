use crate::domain::error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MessageType {
    /// Agent to receiver, no token: the first message of an agent that has no token and no `.probation` file.
    /// It asks the receiver to admit the agent; the receiver adds it on probation and answers `RegistrationReceived`.
    RegistrationStart = 0xA0,
    /// Receiver to agent: the request was received and the agent is on the probation list until a person decides
    /// (also the answer to `AwaitingProbation` while the agent is still on probation).
    /// The agent writes its `.probation` file and keeps waiting on the same connection.
    RegistrationReceived = 0xD0,
    /// Receiver to agent: the agent was approved while it was offline; the answer to `AwaitingProbation`. It carries
    /// the token hash (a `Secret`) the agent must save to `.token.txt` (and then drop its `.probation` file) and
    /// send with every later packet. The agent is authenticated by receiving it.
    RegistrationAccepted = 0xD1,
    /// Receiver to agent: answer to `Registered` whose token hash and MAC address matched. The agent is
    /// authenticated and may start its normal work on the connection.
    RegistrationVerified = 0xD2,
    /// Agent to receiver, no token: the first message of an agent that has no token but already has a `.probation`
    /// file, i.e. it asked before and is still waiting. Used instead of `RegistrationStart` after a reconnect. The
    /// receiver answers `RegistrationAccepted` if the agent was approved meanwhile, otherwise `RegistrationReceived`.
    AwaitingProbation = 0xA2,
    /// Agent to receiver, with the token hash: the first message of an agent that already has its token, sent on
    /// every new connection. The receiver checks the hash and the MAC address and answers `RegistrationVerified`;
    /// if they do not match it closes the connection.
    Registered = 0xA3,
    /// Agent to receiver, with the token hash: telemetry, sent
    /// every `heart_beat_interval` for as long as the connection lives. The receiver also reads it as a sign of life.
    HeartBeat = 0xC0,
    /// Agent to receiver, with the token hash: reserved for reporting changes in the scanned file window.
    /// Not implemented yet.
    FileMetaUpdate = 0xC1,
    /// Receiver to agent: reserved for moving the directory window the agent scans. Not implemented yet.
    MoveWindow = 0xF0,
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
            0xA0 => Ok(MessageType::RegistrationStart),
            0xD0 => Ok(MessageType::RegistrationReceived),
            0xD1 => Ok(MessageType::RegistrationAccepted),
            0xD2 => Ok(MessageType::RegistrationVerified),
            0xA2 => Ok(MessageType::AwaitingProbation),
            0xC0 => Ok(MessageType::HeartBeat),
            0xC1 => Ok(MessageType::FileMetaUpdate),
            0xF0 => Ok(MessageType::MoveWindow),
            0xA3 => Ok(MessageType::Registered),
            other => Err(ProtocolError::UnknownMessageType(other)),
        }
    }
}
