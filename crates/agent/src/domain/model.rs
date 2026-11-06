use std::fmt::Debug;

use crate::domain::config::AgentConfig;
use fleet_crypto::Secret;
use fleet_model::FileMeta;


pub struct Shared {
    pub token_hash: Secret,
    pub config: AgentConfig,
}

#[derive(Debug)]
pub struct FileMetaWindow {
    pub search_from: FileMeta,
    pub loaded_file_meta_window: Vec<FileMeta>,
}
