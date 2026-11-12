use std::time::Duration;

use crate::agent_receiver::MessageType;
use crate::protocol::{Package, UnverifiedPackage, postcard_message};
use fleet_model::{AgentId, MacAddress, RequestId, Telemetry};
use serde::{Deserialize, Serialize};

pub trait AgentReceiverPackage {
    fn agent_id(&self) -> AgentId;
    fn package_id(&self) -> RequestId;
    /// The MAC address of the machine the agent runs on; all zeros if it has no physical network card.
    fn mac_address(&self) -> MacAddress;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
    pub mac_address: MacAddress,
}

impl RegistrationMessage {
    pub fn new(agent_id: AgentId, mac_address: MacAddress) -> Self {
        Self {
            agent_id,
            package_id: RequestId::new(),
            mac_address,
        }
    }
}

impl UnverifiedPackage for RegistrationMessage {
    const MESSAGE_TYPE: u8 = MessageType::RegistrationStart.as_byte();
}

impl AgentReceiverPackage for RegistrationMessage {
    fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    fn package_id(&self) -> RequestId {
        self.package_id
    }

    fn mac_address(&self) -> MacAddress {
        self.mac_address
    }
}

postcard_message!(RegistrationMessage);

/// Sent first by an agent that is on probation and has no token yet: it already asked and is still waiting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwaitingProbationMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
    pub mac_address: MacAddress,
}

impl AwaitingProbationMessage {
    /// A message from `agent_id` with a freshly generated package id.
    pub fn new(agent_id: AgentId, mac_address: MacAddress) -> Self {
        Self {
            agent_id,
            package_id: RequestId::new(),
            mac_address,
        }
    }
}

impl UnverifiedPackage for AwaitingProbationMessage {
    const MESSAGE_TYPE: u8 = MessageType::AwaitingProbation.as_byte();
}

impl AgentReceiverPackage for AwaitingProbationMessage {
    fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    fn package_id(&self) -> RequestId {
        self.package_id
    }

    fn mac_address(&self) -> MacAddress {
        self.mac_address
    }
}

postcard_message!(AwaitingProbationMessage);

/// Sent first by an agent that already has its token. It is sent as a `Package`, so the token hash
/// travels in the packet and the receiver can tell the agent is who it says it is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisteredMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
    pub mac_address: MacAddress,
}

impl RegisteredMessage {
    /// A message from `agent_id` with a freshly generated package id.
    pub fn new(agent_id: AgentId, mac_address: MacAddress) -> Self {
        Self {
            agent_id,
            package_id: RequestId::new(),
            mac_address,
        }
    }
}

impl Package for RegisteredMessage {
    const MESSAGE_TYPE: u8 = MessageType::Registered.as_byte();
}

impl AgentReceiverPackage for RegisteredMessage {
    fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    fn package_id(&self) -> RequestId {
        self.package_id
    }

    fn mac_address(&self) -> MacAddress {
        self.mac_address
    }
}

postcard_message!(RegisteredMessage);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartBeatMessage {
    pub agent_id: AgentId,
    pub package_id: RequestId,
    pub mac_address: MacAddress,
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
    const MESSAGE_TYPE: u8 = MessageType::HeartBeat.as_byte();
}

impl AgentReceiverPackage for HeartBeatMessage {
    fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    fn package_id(&self) -> RequestId {
        self.package_id
    }

    fn mac_address(&self) -> MacAddress {
        self.mac_address
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
    fn into_heart_beat(self, agent_id: AgentId, mac_address: MacAddress) -> HeartBeatMessage;
}

impl IntoHeartBeat for Telemetry {
    fn into_heart_beat(self, agent_id: AgentId, mac_address: MacAddress) -> HeartBeatMessage {
        HeartBeatMessage {
            agent_id,
            package_id: RequestId::new(),
            mac_address,
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
