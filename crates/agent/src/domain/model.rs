use std::fmt::Debug;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::domain::config::AgentConfig;
use fleet_crypto::Secret;
use fleet_model::{AgentId, RequestId};
use fleet_protocol::agent_receiver::HeartBeatMessage;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    File,
    Dir,
    Symlink,
    Other,
}

pub struct Shared {
    pub certificate: Secret,
    pub config: AgentConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    pub path: PathBuf,
    pub kind: FileKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub readonly: bool,
    pub hidden: bool,
    pub symlink_target: Option<PathBuf>,
}

#[derive(Debug)]
pub struct FileMetaWindow {
    pub search_from: FileMeta,
    pub loaded_file_meta_window: Vec<FileMeta>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Telemetry {
    pub timestamp_secs: u64,
    pub uptime_secs: u64,
    pub cpu_cores: u32,
    pub cpu_usage_percent: f32,
    pub cpu_core_usage_percent: Vec<f32>,
    pub load_average: [f64; 3],
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
}

impl Telemetry {
    pub fn into_heart_beat(self, agent_id: AgentId) -> HeartBeatMessage {
        HeartBeatMessage {
            agent_id,
            package_id: RequestId::new(),
            timestamp_secs: self.timestamp_secs,
            uptime_secs: self.uptime_secs,
            cpu_cores: self.cpu_cores,
            cpu_usage_percent: self.cpu_usage_percent,
            cpu_core_usage_percent: self.cpu_core_usage_percent,
            load_average: self.load_average,
            memory_total_bytes: self.memory_total_bytes,
            memory_used_bytes: self.memory_used_bytes,
        }
    }
}
