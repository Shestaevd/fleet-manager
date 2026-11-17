use crate::domain::config::ServerConfig;
use crate::service::agent_store::AgentStoreService;
use fleet_crypto::{Secret, generate_secure_token};
use std::time::Duration;
use time::OffsetDateTime;
use fleet_model::{AgentId, FileMeta, MacAddress, Telemetry};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct Shared {
    pub config: ServerConfig,
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

/// A user of the client CLI: the login and the bytes of the password hash.
#[derive(Debug)]
pub struct User {
    pub login: String,
    pub password: Secret,
    pub session_token: Option<SessionToken>,
}

/// A token a user may present instead of the login and password until `expires_at`.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionToken {
    pub expires_at: OffsetDateTime,
    pub token: Secret,
}

impl SessionToken {
    pub fn is_active(&self, now: OffsetDateTime) -> bool {
        now < self.expires_at
    }
}

impl User {
    /// Replaces the session token with a new random one that is active for `lifetime` from now, and returns it.
    /// It is only in memory until it is written with `write_session_token`.
    pub fn refresh_session_token(&mut self, lifetime: Duration) -> &SessionToken {
        self.session_token.insert(SessionToken {
            expires_at: OffsetDateTime::now_utc() + lifetime,
            token: Secret { content: generate_secure_token() },
        })
    }
}
