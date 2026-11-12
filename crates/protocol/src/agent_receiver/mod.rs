//! Protocol between an agent and the receiver.
pub mod agent_messages;
pub mod message_type;
pub mod receiver_messages;

pub use agent_messages::{
    AgentReceiverPackage, AwaitingProbationMessage, HeartBeatMessage, IntoHeartBeat, RegisteredMessage,
    RegistrationMessage,
};
pub use message_type::MessageType;
pub use receiver_messages::{
    ReceiverAgentPackage, RegistrationAcceptedMessage, RegistrationReceivedMessage, RegistrationVerifiedMessage,
};
