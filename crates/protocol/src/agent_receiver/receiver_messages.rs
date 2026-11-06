use crate::agent_receiver::MessageType;
use crate::domain::error::ParseError;
use crate::domain::model::TokenHash;
use crate::protocol::{UnverifiedPackage, postcard_message};
use fleet_crypto::validate_token_hash;
use fleet_model::RequestId;
use serde::{Deserialize, Serialize};

pub trait ReceiverAgentPackage {
    fn request_id(&self) -> RequestId;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationProbationMessage {
    pub request_id: RequestId,
}

impl UnverifiedPackage for RegistrationProbationMessage {
    const MESSAGE_TYPE: u8 = MessageType::RegistrationProbation.as_byte();
}

impl ReceiverAgentPackage for RegistrationProbationMessage {
    fn request_id(&self) -> RequestId {
        self.request_id
    }
}

postcard_message!(RegistrationProbationMessage);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationAcceptedMessage {
    pub request_id: RequestId,
    pub token_hash: TokenHash,
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
        validate_token_hash(message.token_hash.as_bytes())
            .map_err(|source| ParseError::TokenHash { source })?;
        Ok(message)
    }
}

impl From<RegistrationAcceptedMessage> for Vec<u8> {
    fn from(message: RegistrationAcceptedMessage) -> Vec<u8> {
        postcard::to_stdvec(&message).expect("serializing a plain struct to a Vec can not fail")
    }
}
