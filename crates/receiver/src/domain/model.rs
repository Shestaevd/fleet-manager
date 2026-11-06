use fleet_crypto::Secret;
use fleet_model::{AgentId, FileMeta, Telemetry};


#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub token: Secret,
    pub agent_id: AgentId,
    pub os_meta: Telemetry,
    pub file_meta: Vec<FileMeta>,
}

impl Agent {
    pub fn new_empty(token: Secret, agent_id: AgentId) -> Agent {
        Agent {
            token,
            agent_id,
            os_meta: Telemetry::default(),
            file_meta: vec![],
        }
    }
}