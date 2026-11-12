use std::io::ErrorKind;
use std::path::PathBuf;

use fleet_crypto::Secret;
use fleet_model::error::FsError;
use fleet_model::{AgentId, MacAddress, Telemetry};
use fleet_util::fs::{create_dir_if_not_exist, executable_dir, read_file, write_file, write_secret_file};

use crate::domain::error::AgentStoreError;
use crate::domain::model::{AgentState, BannedAgent, ProbationAgent, RegisteredAgent};

const AGENT_TMP_FILE_CACHE: &str = ".tmp_cache";
const AGENT_BANNED: &str = ".banned";
const AGENT_TOKEN: &str = ".token";
const AGENT_STATUS: &str = "status";
const AGENT_MAC: &str = "mac";
const AGENTS_STORE_DIR: &str = "agents";


pub async fn read_agent(agent_id: AgentId) -> Result<Option<AgentState>, AgentStoreError> {
    read_agent_from(executable_dir()?.join(AGENTS_STORE_DIR), agent_id).await
}


/// - `.banned` present: `Banned` (before the token is looked at);
/// - `.token` present: `Registered`, with the last telemetry from `status` (TOML, default if missing);
/// - no `.token`: `Unregistered` (on probation);
/// - `mac`: 6 raw bytes, all zeros if missing.
pub async fn read_agent_from(store: PathBuf, agent_id: AgentId) -> Result<Option<AgentState>, AgentStoreError> {
    let dir: PathBuf = store.join(agent_id.to_string());
    match tokio::fs::metadata(&dir).await {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => return Err(FsError::NotADirectory { path: dir }.into()),
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(FsError::CanNotReadMetadata { path: dir, source }.into()),
    }

    let mac_address: MacAddress = read_mac(dir.join(AGENT_MAC)).await?;

    if exists(dir.join(AGENT_BANNED)).await? {
        return Ok(Some(AgentState::Banned(BannedAgent { agent_id, mac_address })));
    }

    let Some(token) = read_optional(dir.join(AGENT_TOKEN)).await? else {
        return Ok(Some(AgentState::Unregistered(ProbationAgent { agent_id, mac_address })));
    };
    
    if token.is_empty() {
        return Err(corrupt(dir.join(AGENT_TOKEN), "the token is empty"));
    }

    let os_meta: Telemetry = read_status(dir.join(AGENT_STATUS)).await?;

    Ok(Some(AgentState::Registered(RegisteredAgent {
        token: Secret { content: token },
        agent_id,
        mac_address,
        os_meta,
        file_meta: Vec::new(),
    })))
}

pub async fn add_unverified_agent(agent: &ProbationAgent) -> Result<(), AgentStoreError> {
    add_unverified_agent_to(executable_dir()?.join(AGENTS_STORE_DIR), agent).await
}

pub async fn add_unverified_agent_to(store: PathBuf, agent: &ProbationAgent) -> Result<(), AgentStoreError> {
    create_dir_if_not_exist(store.clone()).await?;
    let dir: PathBuf = store.join(agent.agent_id.to_string());
    match tokio::fs::create_dir(&dir).await {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::AlreadyExists => return Err(AgentStoreError::AlreadyExists(agent.agent_id)),
        Err(source) => return Err(FsError::CanNotWrite { path: dir, source }.into()),
    }
    write_file(dir.join(AGENT_MAC), &agent.mac_address.0).await?;
    Ok(())
}

/// Saves the token of an agent to `<executable dir>/agents/<agent_id>/.token` (mode 0600).
pub async fn write_agent_token(agent_id: AgentId, token: &Secret) -> Result<(), AgentStoreError> {
    write_agent_token_to(executable_dir()?.join(AGENTS_STORE_DIR), agent_id, token).await
}

pub async fn write_agent_token_to(store: PathBuf, agent_id: AgentId, token: &Secret) -> Result<(), AgentStoreError> {
    write_secret_file(store.join(agent_id.to_string()).join(AGENT_TOKEN), &token.content).await?;
    Ok(())
}

pub async fn clear_agent(agent_id: AgentId) -> Result<(), AgentStoreError> {
    clear_agent_from(executable_dir()?.join(AGENTS_STORE_DIR), agent_id).await
}

pub async fn clear_agent_from(store: PathBuf, agent_id: AgentId) -> Result<(), AgentStoreError> {
    let dir: PathBuf = store.join(agent_id.to_string());
    match tokio::fs::remove_dir_all(&dir).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(source) => Err(FsError::CanNotDelete { path: dir, source }.into()),
    }
}

async fn read_mac(path: PathBuf) -> Result<MacAddress, AgentStoreError> {
    let Some(bytes) = read_optional(path.clone()).await? else {
        return Ok(MacAddress::UNSPECIFIED);
    };
    let mac: [u8; 6] = bytes
        .try_into()
        .map_err(|bytes: Vec<u8>| corrupt(path, format!("expected 6 bytes, found {}", bytes.len())))?;
    Ok(MacAddress(mac))
}

async fn read_status(path: PathBuf) -> Result<Telemetry, AgentStoreError> {
    let Some(bytes) = read_optional(path.clone()).await? else {
        return Ok(Telemetry::default());
    };
    let text: &str = std::str::from_utf8(&bytes).map_err(|err| corrupt(path.clone(), err.to_string()))?;
    toml::from_str(text).map_err(|err| corrupt(path, err.to_string()))
}

async fn read_optional(path: PathBuf) -> Result<Option<Vec<u8>>, FsError> {
    match read_file(path).await {
        Ok(content) => Ok(Some(content)),
        Err(FsError::CanNotRead { source, .. }) if source.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

async fn exists(path: PathBuf) -> Result<bool, FsError> {
    tokio::fs::try_exists(&path)
        .await
        .map_err(|source| FsError::CanNotReadMetadata { path, source })
}

fn corrupt(path: PathBuf, reason: impl Into<String>) -> AgentStoreError {
    AgentStoreError::Corrupt { path, reason: reason.into() }
}
