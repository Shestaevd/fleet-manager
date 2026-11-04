use fleet_model::{AgentId, FileMeta, Telemetry};


#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub agent_id: AgentId,
    pub os_meta: Telemetry,
    pub file_meta: Vec<FileMeta>,
}

pub struct Probation {
    pub vec: Vec<Agent>
}