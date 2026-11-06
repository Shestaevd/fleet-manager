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
use fleet_model::error::FsError;
use fleet_protocol::agent_receiver::HeartBeatMessage;
use fleet_protocol::agent_receiver::IntoHeartBeat;
use fleet_protocol::agent_receiver::{
    MessageType, RegistrationAcceptedMessage, RegistrationFinishedMessage, RegistrationMessage,
    RegistrationProbationMessage,
};
use fleet_protocol::protocol::Package;
use fleet_protocol::protocol::UnverifiedPackage;
use sysinfo::System;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time::{Instant, Interval, interval_at};

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
                collect_telemetry(&mut system).into_heart_beat(shared.config.agent_id);
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

pub fn registration_process<R, W>(
    config: AgentConfig,
    reader: Arc<Mutex<R>>,
    writer: Arc<Mutex<W>>,
) -> JoinHandle<Shared>
where
    W: AsyncWrite + Unpin + Send + 'static,
    R: AsyncRead + Unpin + Send + 'static,
{
    const MAX_PACKET_BYTES: usize = 64 * 1024;

    tokio::spawn(async move {
        let mut reader: MutexGuard<R> = reader.lock().await;

        match read_token_hash().await {
            Ok(token_hash) => {
                let shared: Shared = Shared {
                    token_hash: Secret {
                        content: token_hash,
                    },
                    config,
                };
                return shared;
            }
            Err(AgentError::Fs(FsError::CanNotRead { source, .. }))
                if source.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => panic!("can not use the saved token hash: {err}"),
        }

        let in_probation: bool = read_is_in_probation()
            .await
            .expect("can not check for the probation file");
        if !in_probation {
            let registration: RegistrationMessage = RegistrationMessage::new(config.agent_id);
            let packet: Vec<u8> = registration
                .encode()
                .expect("can not encode the registration message");
            let mut writer: MutexGuard<W> = writer.lock().await;
            writer
                .write_all(&packet)
                .await
                .expect("can not send the registration message");
            writer
                .flush()
                .await
                .expect("can not send the registration message");
        }

        loop {
            let mut len_bytes: [u8; 4] = [0u8; 4];
            reader
                .read_exact(&mut len_bytes)
                .await
                .expect("the stream closed before the registration finished");
            let len: usize = u32::from_be_bytes(len_bytes) as usize;
            if len == 0 || len > MAX_PACKET_BYTES {
                panic!("the receiver sent a registration packet of {len} bytes");
            }

            let mut packet: Vec<u8> = Vec::with_capacity(4 + len);
            packet.extend_from_slice(&len_bytes);
            packet.resize(4 + len, 0);
            reader
                .read_exact(&mut packet[4..])
                .await
                .expect("the stream closed before the registration finished");

            match MessageType::try_from(packet[4]) {
                Ok(MessageType::RegistrationProbation) => {
                    match RegistrationProbationMessage::decode(&packet) {
                        Ok(_) => {
                            write_in_probation()
                                .await
                                .expect("can not write the probation file");
                            tracing::info!("the receiver put this agent on probation");
                        }
                        Err(err) =>
                            tracing::warn!("ignoring a broken probation message: {err}"),
                    }
                }
                Ok(MessageType::RegistrationAccepted) => {
                    match RegistrationAcceptedMessage::decode(&packet) {
                        Ok(message) => {
                            let token_hash: Vec<u8> = message.token_hash.into_bytes();
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


                            let finished: Vec<u8> = RegistrationFinishedMessage::new(config.agent_id)
                                .encode(&token_hash)
                                .expect("can not encode the registration finished message");
                            let mut writer: MutexGuard<W> = writer.lock().await;
                            let sent: std::io::Result<()> = async {
                                writer.write_all(&finished).await?;
                                writer.flush().await
                            }
                            .await;
                            if let Err(err) = sent {
                                tracing::warn!("can not send the registration finished message: {err}");
                            }

                            let shared: Shared = Shared {
                                token_hash: Secret {
                                    content: token_hash,
                                },
                                config,
                            };
                            return shared;
                        }
                        Err(err) => {
                            tracing::warn!("ignoring a broken registration accepted message: {err}")
                        }
                    }
                }
                Ok(other) => {
                    tracing::warn!("ignoring unexpected {other:?} message while registering")
                }
                Err(err) => tracing::warn!("ignoring a packet while registering: {err}"),
            }
        }
    })
}
