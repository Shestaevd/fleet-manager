use std::path::PathBuf;

use fleet_crypto::Secret;
use fleet_model::{AgentId, MacAddress, Telemetry};

use crate::domain::error::AgentStoreError;
use crate::domain::model::AgentState;
use crate::domain::model::ProbationAgent;
use crate::fs::fs_io::{add_unverified_agent_to, clear_agent_from, read_agent_from, write_agent_token_to};

fn store(name: &str) -> PathBuf {
    let path: PathBuf = std::env::temp_dir().join(format!("fleet-receiver-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn agent_dir(store: &PathBuf, agent_id: AgentId) -> PathBuf {
    let dir: PathBuf = store.join(agent_id.to_string());
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn an_agent_is_read_from_its_directory() {
    let store: PathBuf = store("read");
    let agent_id: AgentId = AgentId::new();

    assert!(read_agent_from(store.clone(), agent_id).await.unwrap().is_none());

    let dir: PathBuf = agent_dir(&store, agent_id);
    std::fs::write(dir.join("mac"), [1, 2, 3, 4, 5, 6]).unwrap();
    let Some(AgentState::Unregistered(probation)) = read_agent_from(store.clone(), agent_id).await.unwrap() else {
        panic!("an agent without a token is on probation");
    };
    assert_eq!(probation.mac_address, MacAddress([1, 2, 3, 4, 5, 6]));

    std::fs::write(dir.join(".token"), [9u8; 32]).unwrap();
    let status: Telemetry = Telemetry { cpu_cores: 4, ..Telemetry::default() };
    std::fs::write(dir.join("status"), toml::to_string(&status).unwrap()).unwrap();
    let Some(AgentState::Registered(agent)) = read_agent_from(store.clone(), agent_id).await.unwrap() else {
        panic!("an agent with a token is registered");
    };
    assert_eq!(agent.token.content, vec![9u8; 32]);
    assert_eq!(agent.os_meta, status);
    assert!(agent.file_meta.is_empty());

    std::fs::write(dir.join(".banned"), []).unwrap();
    let Some(banned) = read_agent_from(store.clone(), agent_id).await.unwrap() else {
        panic!("a banned agent is still read");
    };
    assert!(banned.is_banned());

    std::fs::remove_file(dir.join(".banned")).unwrap();
    std::fs::write(dir.join("mac"), [1, 2]).unwrap();
    assert!(matches!(read_agent_from(store.clone(), agent_id).await, Err(AgentStoreError::Corrupt { .. })));


    clear_agent_from(store.clone(), agent_id).await.unwrap();
    assert!(!dir.exists());
    assert!(read_agent_from(store.clone(), agent_id).await.unwrap().is_none());
    clear_agent_from(store.clone(), agent_id).await.unwrap();

    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test]
async fn an_unverified_agent_is_added_once_and_read_back_on_probation() {
    let store: PathBuf = store("add");
    let agent_id: AgentId = AgentId::new();
    let agent: ProbationAgent = ProbationAgent { agent_id, mac_address: MacAddress([6, 5, 4, 3, 2, 1]) };

    add_unverified_agent_to(store.clone(), &agent).await.unwrap();
    let Some(AgentState::Unregistered(read)) = read_agent_from(store.clone(), agent_id).await.unwrap() else {
        panic!("an added agent is on probation");
    };
    assert_eq!(read.mac_address, agent.mac_address);

    assert!(matches!(
        add_unverified_agent_to(store.clone(), &agent).await,
        Err(AgentStoreError::AlreadyExists(id)) if id == agent_id
    ));

    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test]
async fn a_saved_token_turns_a_probation_agent_into_a_registered_one() {
    let store: PathBuf = store("token");
    let agent_id: AgentId = AgentId::new();
    let agent: ProbationAgent = ProbationAgent { agent_id, mac_address: MacAddress([1; 6]) };
    add_unverified_agent_to(store.clone(), &agent).await.unwrap();

    let token: Secret = Secret { content: vec![3u8; 32] };
    write_agent_token_to(store.clone(), agent_id, &token).await.unwrap();

    let Some(AgentState::Registered(read)) = read_agent_from(store.clone(), agent_id).await.unwrap() else {
        panic!("an agent with a token is registered");
    };
    assert_eq!(read.token, token);
    assert_eq!(read.mac_address, agent.mac_address);

    let _ = std::fs::remove_dir_all(&store);
}
