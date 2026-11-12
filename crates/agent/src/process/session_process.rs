use std::sync::Arc;
use std::time::Duration;

use crate::domain::config::AgentConfig;
use crate::domain::error::AgentError;
use crate::domain::model::Shared;
use crate::fs::fs_io::read_is_in_probation;
use crate::fs::fs_io::{
    delete_in_probation, read_token_hash, write_in_probation, write_token_hash,
};
use crate::process::telemetry::collect_telemetry;
use fleet_crypto::Secret;
use fleet_model::MacAddress;
use fleet_model::error::FsError;
use fleet_protocol::agent_receiver::HeartBeatMessage;
use fleet_protocol::agent_receiver::IntoHeartBeat;
use fleet_protocol::agent_receiver::{
    AwaitingProbationMessage, MessageType, RegisteredMessage, RegistrationAcceptedMessage,
    RegistrationMessage, RegistrationReceivedMessage, RegistrationVerifiedMessage,
};
use fleet_protocol::protocol::Package;
use fleet_protocol::protocol::UnverifiedPackage;
use sysinfo::System;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time::{Instant, Interval, interval_at};

/// Starts the task that sends a heart beat to `writer` every `heart_beat_interval`: it collects telemetry,
/// builds the message with the agent id and sends it together with the token hash. The first beat goes out at
/// once and its CPU figures are 0, because usage is measured between two samples. A message that can not be
/// encoded is skipped. When the stream breaks the task ends, which lets the connector reconnect.
pub fn heart_beat_process<W>(shared: Arc<Shared>, writer: Arc<Mutex<W>>) -> JoinHandle<()>
where
    W: AsyncWrite + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let period: Duration = shared.config.heart_beat_interval;
        let mut system: System = System::new();
        let mut ticker: Interval = interval_at(Instant::now(), period);
        loop {
            ticker.tick().await;

            let heart_beat: HeartBeatMessage =
                collect_telemetry(&mut system).into_heart_beat(shared.config.agent_id, shared.mac_address);
            let bytes: Vec<u8> = match heart_beat.encode(&shared.token_hash.content) {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::error!("can not encode heart beat, continuing: {err}");
                    continue;
                }
            };

            let mut writer: MutexGuard<W> = writer.lock().await;
            if let Err(err) = writer.write_all(&bytes).await {
                tracing::warn!("stream is broken, stopping heart beat: {err}");
                return;
            }
            if let Err(err) = writer.flush().await {
                tracing::warn!("stream is broken, stopping heart beat: {err}");
                return;
            }
        }
    })
}

pub fn session_process<R, W>(
    shared: Arc<Shared>,
    reader: Arc<Mutex<R>>,
    writer: Arc<Mutex<W>>,
) -> JoinHandle<()>
where
    W: AsyncWrite + Unpin + Send + 'static,
    R: AsyncRead + Unpin + Send + 'static,
{
    let shared = shared.clone();
    let writer_clone = writer.clone();
    let reader_clone = reader.clone();
    heart_beat_process(shared, writer_clone)
}

/// Starts the task that registers the agent on a new connection and returns the `Shared` it ends up with.
/// `mac_address` is the machine's address, read once at start-up; it goes into every message the agent builds.
///
/// It first says where the agent stands: `Registered` (sent with the token hash) if a token is saved,
/// `AwaitingProbation` if only the `.probation` file exists, `RegistrationStart` otherwise. Then it waits for
/// the receiver: `RegistrationVerified` (answer to `Registered`) returns; `RegistrationReceived` creates
/// `.probation`; `RegistrationAccepted` saves the token hash, removes `.probation` and returns, the receiver
/// having authenticated the agent by sending it. Any other packet is skipped.
///
/// The task panics, so the awaited handle is an `Err`, when the saved token is invalid, when a file can not be
/// read or written, or when the stream closes before registration finishes; the connector then reconnects.
pub fn registration_process<R, W>(
    config: AgentConfig,
    mac_address: MacAddress,
    reader: Arc<Mutex<R>>,
    writer: Arc<Mutex<W>>,
) -> JoinHandle<Shared>
where
    W: AsyncWrite + Unpin + Send + 'static,
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let token_hash: Option<Vec<u8>> = match read_token_hash().await {
            Ok(token_hash) => Some(token_hash),
            Err(AgentError::Fs(FsError::CanNotRead { source, .. }))
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                None
            }
            Err(err) => panic!("can not use the saved token hash: {err}"),
        };

        let first_packet: Vec<u8> = match &token_hash {
            Some(token_hash) => RegisteredMessage::new(config.agent_id, mac_address)
                .encode(token_hash)
                .expect("can not encode the registered message"),
            None => {
                let in_probation: bool = read_is_in_probation()
                    .await
                    .expect("can not check for the probation file");
                if in_probation {
                    AwaitingProbationMessage::new(config.agent_id, mac_address)
                        .encode()
                        .expect("can not encode the awaiting probation message")
                } else {
                    RegistrationMessage::new(config.agent_id, mac_address)
                        .encode()
                        .expect("can not encode the registration message")
                }
            }
        };
        {
            let mut writer: MutexGuard<W> = writer.lock().await;
            writer
                .write_all(&first_packet)
                .await
                .expect("can not send the first registration message");
            writer
                .flush()
                .await
                .expect("can not send the first registration message");
        }

        let mut token_hash: Option<Vec<u8>> = token_hash;
        let mut reader: MutexGuard<R> = reader.lock().await;

        loop {
            let mut len_bytes: [u8; 4] = [0u8; 4];
            reader
                .read_exact(&mut len_bytes)
                .await
                .expect("the stream closed before the registration finished");
            let len: usize = u32::from_be_bytes(len_bytes) as usize;

            let mut body: Vec<u8> = Vec::with_capacity(len);
            (&mut *reader)
                .take(len as u64)
                .read_to_end(&mut body)
                .await
                .expect("the stream broke before the registration finished");

            let message_type = body
                .first()
                .copied()
                .ok_or(fleet_protocol::ProtocolError::EmptyPacket)
                .and_then(MessageType::try_from);
            
            match message_type {
                Ok(MessageType::RegistrationReceived) => {
                    match RegistrationReceivedMessage::decode(body) {
                        Ok(_) => {
                            write_in_probation()
                                .await
                                .expect("can not write the probation file");
                            tracing::info!("the receiver put this agent on probation");
                        }
                        Err(err) => tracing::warn!("ignoring a broken probation message: {err}"),
                    }
                }
                Ok(MessageType::RegistrationAccepted) => {
                    match RegistrationAcceptedMessage::decode(body) {
                        Ok(message) => {
                            let token_hash: Vec<u8> = message.token_hash.content;
                            write_token_hash(&token_hash)
                                .await
                                .expect("can not save the token hash");
                            tracing::info!("registration accepted, token hash saved");

                            match delete_in_probation(std::path::PathBuf::new()).await {
                                Ok(()) => {}
                                Err(FsError::CanNotDelete { source, .. })
                                    if source.kind() == std::io::ErrorKind::NotFound => {}
                                Err(err) => {
                                    tracing::warn!("can not remove the probation file: {err}")
                                }
                            }

                            let shared: Shared = Shared {
                                token_hash: Secret {
                                    content: token_hash,
                                },
                                mac_address,
                                config,
                            };
                            return shared;
                        }
                        Err(err) => {
                            tracing::warn!("ignoring a broken registration accepted message: {err}")
                        }
                    }
                }
                Ok(MessageType::RegistrationVerified) => match RegistrationVerifiedMessage::decode(body) {
                    Ok(_) => match token_hash.take() {
                        Some(token_hash) => {
                            tracing::info!("the receiver verified this agent");
                            return Shared {
                                token_hash: Secret { content: token_hash },
                                mac_address,
                                config,
                            };
                        }
                        None => tracing::warn!("ignoring a verification: this agent sent no token"),
                    },
                    Err(err) => tracing::warn!("ignoring a broken verified message: {err}"),
                },
                Ok(other) => {
                    tracing::warn!("ignoring unexpected {other:?} message while registering")
                }
                Err(err) => tracing::warn!("ignoring a packet while registering: {err}"),
            }
        }
    })
}
