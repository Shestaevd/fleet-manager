use crate::domain::model::Agent;
use fleet_model::error::FsError;
use std::path::PathBuf;
use fleet_util::fs::create_dir_if_not_exist;

const AGENT_TMP_FILE_CACHE: &'static str = ".tmp_cache";
const AGENT_VERIFIED: &'static str = ".verified";
const AGENT_BANNED: &'static str = ".banned";
const AGENTS_STORE_DIR: &'static str = ".agents";

pub async fn add_agent(path: PathBuf, agent: Agent) -> Result<(), FsError> {
    let agent_path: PathBuf = path.join(&agent.agent_id.to_string());
    create_dir_if_not_exist(agent_path.clone()).await?;
    create_dir_if_not_exist(agent_path.join(AGENT_TMP_FILE_CACHE).clone()).await?;




    todo!()
}
