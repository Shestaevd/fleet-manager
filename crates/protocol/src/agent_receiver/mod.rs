//! Protocol between an agent and the receiver.
pub mod message;
pub mod message_type;

pub use message::HeartBeatMessage;
pub use message_type::MessageType;
