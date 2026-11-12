use crate::agent_receiver::MessageType;
use crate::domain::error::ParseError;
use crate::protocol::{UnverifiedPackage, postcard_message};
use fleet_crypto::{Secret, validate_token_hash};
use fleet_model::RequestId;
use serde::{Deserialize, Serialize};

pub trait ReceiverAgentPackage {
    fn request_id(&self) -> RequestId;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationReceivedMessage {
    pub request_id: RequestId,
}

impl UnverifiedPackage for RegistrationReceivedMessage {
    const MESSAGE_TYPE: u8 = MessageType::RegistrationReceived.as_byte();
}

impl ReceiverAgentPackage for RegistrationReceivedMessage {
    fn request_id(&self) -> RequestId {
        self.request_id
    }
}

postcard_message!(RegistrationReceivedMessage);

/// Sent by the receiver when the token hash and the MAC address of a `Registered` message are the saved ones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationVerifiedMessage {
    pub request_id: RequestId,
}

impl UnverifiedPackage for RegistrationVerifiedMessage {
    const MESSAGE_TYPE: u8 = MessageType::RegistrationVerified.as_byte();
}

impl ReceiverAgentPackage for RegistrationVerifiedMessage {
    fn request_id(&self) -> RequestId {
        self.request_id
    }
}

postcard_message!(RegistrationVerifiedMessage);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationAcceptedMessage {
    pub request_id: RequestId,
    pub token_hash: Secret,
}

impl UnverifiedPackage for RegistrationAcceptedMessage {
    const MESSAGE_TYPE: u8 = MessageType::RegistrationAccepted.as_byte();
}

impl ReceiverAgentPackage for RegistrationAcceptedMessage {
    fn request_id(&self) -> RequestId {
        self.request_id
    }
}

impl TryFrom<Vec<u8>> for RegistrationAcceptedMessage {
    type Error = ParseError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let message: RegistrationAcceptedMessage = postcard::from_bytes(&bytes)
            .map_err(|source| ParseError::Decode { source })?;
        validate_token_hash(&message.token_hash.content)
            .map_err(|source| ParseError::TokenHash { source })?;
        Ok(message)
    }
}

impl From<RegistrationAcceptedMessage> for Vec<u8> {
    fn from(message: RegistrationAcceptedMessage) -> Vec<u8> {
        postcard::to_stdvec(&message).expect("serializing a plain struct to a Vec can not fail")
    }
}
