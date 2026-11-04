use crate::domain::config::{AgentConfig, CONFIG_FILE_NAME, default_config};
use crate::domain::model::{FileMetaWindow, Shared};
use crate::fs::fs_io::{file_meta_of, fs_home, read_pem};
use crate::process::fs_scan_process::fs_scan_process;
use crate::process::session_process::session_process;
use crate::tcp::connector;
use fleet_config::read_config_or_create;
use fleet_crypto::Secret;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;

mod domain;
mod fs;
mod tcp;
mod process;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let (config, created): (AgentConfig, bool) =
        read_config_or_create(CONFIG_FILE_NAME, &default_config()).expect("can not load agent config");
    if created {
        tracing::warn!("{CONFIG_FILE_NAME} did not exist, created a default one next to the binary; set receiver_url");
    }
    tracing::info!("receiver: {}", config.receiver_url);

    let certificate: Vec<u8> = read_pem().expect("can not read host.pem");
    let shared: Arc<Shared> = Arc::new(Shared {
        certificate: Secret { content: certificate },
        config,
    });

    let lookup_window: Arc<RwLock<FileMetaWindow>> = Arc::new(RwLock::new(FileMetaWindow {
        search_from: file_meta_of(fs_home()).expect("can not read home directory"),
        loaded_file_meta_window: Vec::new(),
    }));
        
        let scan_handle: JoinHandle<()> = fs_scan_process(shared.clone(), lookup_window.clone());

        let connector_task_handle: JoinHandle<std::io::Result<()>> = connector::run(shared, session_process);

        match connector_task_handle.await {
            Ok(_) => {
                scan_handle.abort();
                tracing::info!("agent stopped");
            }
            Err(err) => {
                scan_handle.abort();
                tracing::error!("connector task exited unexpectedly and could not be joined due to: {}", err);
            }
        }
}
