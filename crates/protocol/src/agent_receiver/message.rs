use serde::{Deserialize, Serialize};
use uuid::Uuid;
use fleet_model::{AgentId, RequestId};
use crate::agent_receiver::MessageType;
use crate::domain::error::ParseError;
use crate::protocol::Package;

trait AgentReceiverPackage : Package {
    fn agent_id(&self) -> AgentId;
    fn package_id(&self) -> RequestId;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartBeatMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
    pub timestamp_secs: u64,
    pub uptime_secs: u64,
    pub cpu_cores: u32,
    pub cpu_usage_percent: f32,
    pub cpu_core_usage_percent: Vec<f32>,
    pub load_average: [f64; 3],
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
}

impl Package for HeartBeatMessage {
    fn message_type(&self) -> u8 {
        MessageType::HeartBeat.into()
    }
}

impl AgentReceiverPackage for HeartBeatMessage {
    fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    fn package_id(&self) -> RequestId {
        self.package_id
    }
}

impl TryFrom<Vec<u8>> for HeartBeatMessage {
    type Error = ParseError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        postcard::from_bytes::<HeartBeatMessage>(&bytes)
            .map_err(|source| ParseError::Decode { source })
    }
}

impl From<HeartBeatMessage> for Vec<u8> {
    fn from(message: HeartBeatMessage) -> Vec<u8> {
        postcard::to_stdvec(&message).expect("serializing a plain struct to a Vec can not fail")
    }
}
