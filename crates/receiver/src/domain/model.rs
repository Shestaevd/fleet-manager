use crate::domain::config::ReceiverConfig;
use crate::service::agent_store::AgentStoreService;
use fleet_crypto::Secret;
use fleet_model::{AgentId, FileMeta, MacAddress, Telemetry};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct Shared {
    pub config: ReceiverConfig,
    pub agent_manager: Mutex<AgentStoreService>,
}

trait Agent {
    fn get_agent_id(&self) -> &AgentId;
}

pub(crate) enum AgentState {
    Unregistered(ProbationAgent),
    Registered(RegisteredAgent),
    Banned(BannedAgent),
}

impl AgentState {
    pub fn agent_id(&self) -> &AgentId {
        match self {
            AgentState::Unregistered(agent) => agent.get_agent_id(),
            AgentState::Registered(agent) => agent.get_agent_id(),
            AgentState::Banned(agent) => agent.get_agent_id(),
        }
    }

    pub fn is_banned(&self) -> bool {
        matches!(self, AgentState::Banned(_))
    }
}

#[derive(Debug)]
pub struct ProbationAgent {
    pub agent_id: AgentId,
    pub mac_address: MacAddress,
}

impl Agent for ProbationAgent {
    fn get_agent_id(&self) -> &AgentId {
        &self.agent_id
    }
}

#[derive(Debug)]
pub struct RegisteredAgent {
    pub token: Secret,
    pub agent_id: AgentId,
    pub mac_address: MacAddress,
    pub os_meta: Telemetry,
    pub file_meta: Vec<FileMeta>
}

impl Agent for RegisteredAgent {
    fn get_agent_id(&self) -> &AgentId {
        &self.agent_id
    }
}

#[derive(Debug)]
pub struct BannedAgent {
    pub agent_id: AgentId,
    pub mac_address: MacAddress,
}

impl Agent for BannedAgent {
    fn get_agent_id(&self) -> &AgentId {
        &self.agent_id
    }
}
