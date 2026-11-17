use std::sync::Arc;

use fleet_config::read_config_or_create;
use tokio::sync::Mutex;
use tokio::task::{JoinError, JoinHandle};

use crate::domain::config::{CONFIG_FILE_NAME, DEFAULT_CONFIG, ReceiverConfig};
use crate::domain::model::Shared;
use crate::process::session_process::session_process;
use crate::service::agent_store::AgentStoreService;
use crate::tcp::server::run;

mod domain;
mod fs;
mod tcp;

#[cfg(test)]
mod test;
mod process;
mod service;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let (config, created): (ReceiverConfig, bool) =
        read_config_or_create(CONFIG_FILE_NAME, DEFAULT_CONFIG).expect("can not load receiver config");
    if created {
        tracing::warn!("{CONFIG_FILE_NAME} did not exist, created a default one next to the binary");
    }

    let shared: Arc<Shared> = Arc::new(Shared {
        config: config.agents_server_config.clone(),
        agent_manager: Mutex::new(AgentStoreService::new()),
    });

    let agent_server: JoinHandle<std::io::Result<()>> = run(config.agents_server_config.clone(), move |reader, writer| {
        let shared = Arc::clone(&shared);
        async move { session_process(shared, reader, writer).await }
    });

    let client_server: JoinHandle<std::io::Result<()>> = run(config.client_server_config.clone(), move |reader, writer| {
        async move { todo!() }
    });

    let (agents, clients) = tokio::join!(agent_server, client_server);
    log_server_end("agent server", agents);
    log_server_end("client server", clients);
    tracing::info!("the receiver finished its work");
}

/// Says in the log how a server task ended: normally, with an error, by a panic or by being cancelled.
fn log_server_end(name: &str, ended: Result<std::io::Result<()>, JoinError>) {
    match ended {
        Ok(Ok(())) => tracing::info!("the {name} stopped"),
        Ok(Err(err)) => tracing::error!("the {name} failed: {err}"),
        Err(err) if err.is_panic() => tracing::error!("the {name} panicked: {err}"),
        Err(err) => tracing::warn!("the {name} was cancelled: {err}"),
    }
}
