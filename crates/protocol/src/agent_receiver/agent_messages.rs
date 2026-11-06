use std::time::Duration;

use crate::agent_receiver::MessageType;
use crate::protocol::{Package, UnverifiedPackage, postcard_message};
use fleet_model::{AgentId, RequestId, Telemetry};
use serde::{Deserialize, Serialize};

pub trait AgentReceiverPackage {
    fn agent_id(&self) -> AgentId;
    fn package_id(&self) -> RequestId;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
}

impl RegistrationMessage {
    /// A registration from `agent_id` with a freshly generated package id.
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            package_id: RequestId::new(),
        }
    }
}

impl UnverifiedPackage for RegistrationMessage {
    const MESSAGE_TYPE: u8 = MessageType::Registration.as_byte();
}

impl AgentReceiverPackage for RegistrationMessage {
    fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    fn package_id(&self) -> RequestId {
        self.package_id
    }
}

postcard_message!(RegistrationMessage);

/// Sent by an agent once it has saved the token hash from `RegistrationAccepted`. It is sent as a `Package`,
/// so the saved token hash travels in the packet and proves the agent really holds it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationFinishedMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
}

impl RegistrationFinishedMessage {
    /// A confirmation from `agent_id` with a freshly generated package id.
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            package_id: RequestId::new(),
        }
    }
}

impl Package for RegistrationFinishedMessage {
    fn message_type(&self) -> u8 {
        MessageType::RegistrationFinished.into()
    }
}

impl AgentReceiverPackage for RegistrationFinishedMessage {
    fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    fn package_id(&self) -> RequestId {
        self.package_id
    }
}

postcard_message!(RegistrationFinishedMessage);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartBeatMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
    pub timestamp: Duration,
    pub uptime: Duration,
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

impl From<HeartBeatMessage> for Telemetry {
    fn from(message: HeartBeatMessage) -> Self {
        Telemetry {
            timestamp: message.timestamp,
            uptime: message.uptime,
            cpu_cores: message.cpu_cores,
            cpu_usage_percent: message.cpu_usage_percent,
            cpu_core_usage_percent: message.cpu_core_usage_percent,
            load_average: message.load_average,
            memory_total_bytes: message.memory_total_bytes,
            memory_used_bytes: message.memory_used_bytes,
        }
    }
}

postcard_message!(HeartBeatMessage);

pub trait IntoHeartBeat {
    fn into_heart_beat(self, agent_id: AgentId) -> HeartBeatMessage;
}

impl IntoHeartBeat for Telemetry {
    fn into_heart_beat(self, agent_id: AgentId) -> HeartBeatMessage {
        HeartBeatMessage {
            agent_id,
            package_id: RequestId::new(),
            timestamp: self.timestamp,
            uptime: self.uptime,
            cpu_cores: self.cpu_cores,
            cpu_usage_percent: self.cpu_usage_percent,
            cpu_core_usage_percent: self.cpu_core_usage_percent,
            load_average: self.load_average,
            memory_total_bytes: self.memory_total_bytes,
            memory_used_bytes: self.memory_used_bytes,
        }
    }
}
