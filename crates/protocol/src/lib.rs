//! Wire protocols of Fleet. The two links use separate message type maps:
//! - `agent_receiver`: between an agent and the receiver
//! - `receiver_client`: between the receiver and an operator client
pub mod agent_receiver;
pub mod protocol;
pub mod receiver_client;

#[cfg(test)]
mod test;
mod domain;
