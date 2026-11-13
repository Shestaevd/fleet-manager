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
        config: config.clone(),
        agent_manager: Mutex::new(AgentStoreService::new()),
    });

    let server: JoinHandle<std::io::Result<()>> = run(Arc::new(config), move |config, reader, writer| {
        let shared: Arc<Shared> = Arc::clone(&shared);
        async move { session_process(shared, reader, writer).await }
    });

    tokio::select! {
        ended = server => match ended {
            Ok(Ok(())) => tracing::info!("the server stopped"),
            Ok(Err(err)) => tracing::error!("the server failed: {err}"),
            Err(err) => tracing::error!("the server task ended unexpectedly: {err}"),
        },
        _ = tokio::signal::ctrl_c() => tracing::info!("interrupted, shutting down"),
    }
}
