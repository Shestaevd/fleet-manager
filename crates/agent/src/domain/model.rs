use std::fmt::Debug;

use crate::domain::config::AgentConfig;
use fleet_crypto::Secret;
use fleet_model::{AgentId, FileMeta, RequestId, Telemetry};
use fleet_protocol::agent_receiver::HeartBeatMessage;


pub struct Shared {
    pub certificate: Secret,
    pub config: AgentConfig,
}

#[derive(Debug)]
pub struct FileMetaWindow {
    pub search_from: FileMeta,
    pub loaded_file_meta_window: Vec<FileMeta>,
}

/// Turns a `Telemetry` snapshot into the heartbeat message sent to the receiver.
pub trait IntoHeartBeat {
    /// Builds the heartbeat with a freshly generated request id.
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
