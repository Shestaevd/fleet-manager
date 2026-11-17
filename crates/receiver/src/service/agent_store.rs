use crate::domain::error::AgentStoreError;
use crate::domain::model::{AgentState, ProbationAgent, RegisteredAgent};
use crate::fs::agent_fs_io::{add_unverified_agent, read_agent, write_agent_token};
use fleet_crypto::{Secret, generate_secure_token, hash_token};
use fleet_model::{AgentId, MacAddress, Telemetry};

pub struct AgentStoreService {
    agents: Vec<AgentState>
}

impl Default for AgentStoreService {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentStoreService {
    
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    pub async fn add(&mut self, agent_id: AgentId, mac_address: MacAddress) -> Result<&AgentState, AgentStoreError> {
        if self.find_agent(&agent_id).await?.is_some() {
            return Err(AgentStoreError::AlreadyExists(agent_id));
        }

        let agent: ProbationAgent = ProbationAgent { agent_id, mac_address };
        add_unverified_agent(&agent).await?;

        self.agents.push(AgentState::Unregistered(agent));
        Ok(&self.agents[self.agents.len() - 1])
    }

    pub async fn register_new(&mut self, agent_id: AgentId) -> Result<Secret, AgentStoreError> {
        let mac_address: MacAddress = match self.find_agent(&agent_id).await? {
            Some(AgentState::Unregistered(agent)) => agent.mac_address,
            Some(AgentState::Banned(_)) => return Err(AgentStoreError::Banned(agent_id)),
            Some(AgentState::Registered(_)) => return Err(AgentStoreError::AlreadyRegistered(agent_id)),
            None => return Err(AgentStoreError::NotFound(agent_id)),
        };

        let token_hash: Secret = Secret { content: hash_token(&generate_secure_token()) };
        write_agent_token(agent_id, &token_hash).await?;

        let registered: AgentState = AgentState::Registered(RegisteredAgent {
            token: token_hash.clone(),
            agent_id,
            mac_address,
            os_meta: Telemetry::default(),
            file_meta: Vec::new(),
        });
        if let Some(state) = self.find_agent(&agent_id).await? {
            *state = registered;
        }
        Ok(token_hash)
    }

    pub async fn find_agent(&mut self, agent_id: &AgentId) -> Result<Option<&mut AgentState>, AgentStoreError> {
        let position: Option<usize> = match self.agents.iter().position(|agent| agent.agent_id() == agent_id) {
            Some(position) => Some(position),
            None => match read_agent(*agent_id).await? {
                Some(agent) => {
                    self.agents.push(agent);
                    Some(self.agents.len() - 1)
                }
                None => None
            },
        };
        Ok(position.map(|position| &mut self.agents[position]))
    }
}
