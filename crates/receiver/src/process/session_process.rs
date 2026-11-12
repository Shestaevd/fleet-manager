use std::sync::Arc;

use fleet_crypto::Secret;
use fleet_model::{AgentId, RequestId};
use fleet_protocol::ProtocolError;
use fleet_protocol::agent_receiver::{
    AgentReceiverPackage, AwaitingProbationMessage, MessageType, RegisteredMessage, RegistrationAcceptedMessage,
    RegistrationMessage, RegistrationReceivedMessage, RegistrationVerifiedMessage,
};
use fleet_protocol::protocol::{Package, UnverifiedPackage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::{JoinError, JoinHandle};

use crate::domain::error::{AgentStoreError, VerificationError};
use crate::domain::model::{AgentState, Shared};
use crate::service::agent_store::AgentStoreService;
use crate::tcp::server::{Reader, Writer};

pub async fn session_process(shared: Arc<Shared>, reader: Reader, writer: Writer) -> Result<(), JoinError> {
    match verification_process(shared, reader, writer).await? {
        Ok(verified) => listening_process(verified).await,
        Err(err) => {
            tracing::warn!("verification failed: {err}, closing connection");
            Ok(())
        }
    }
}

fn listening_process(verified: Verified) -> JoinHandle<()> {
    tokio::spawn(async move {
        let Verified { agent_id, mut reader, writer } = verified;
        let _writer: Writer = writer;
        tracing::info!("agent {agent_id} verified");

        let mut buffer: [u8; 1024] = [0u8; 1024];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!("connection of agent {agent_id} broke: {err}");
                    break;
                }
            }
        }
        tracing::debug!("agent {agent_id} disconnected");
    })
}

enum Verdict {
    /// The agent proved itself: answer `RegistrationVerified` and finish.
    Verified { agent_id: AgentId, request_id: RequestId },
    /// The agent was approved while offline: answer `RegistrationAccepted` with its token and finish.
    Accepted { agent_id: AgentId, request_id: RequestId, token: Secret },
    /// The agent is on probation: answer `RegistrationReceived` and keep reading.
    Probation { request_id: RequestId },
    /// The agent is unknown, banned, already registered or does not match what is stored: close the connection.
    Rejected(AgentId),
}

pub struct Verified {
    pub agent_id: AgentId,
    pub reader: Reader,
    pub writer: Writer,
}

/// Starts the task that authenticates a freshly connected agent from its first messages and resolves to its id
/// together with the reader and writer. On an error the halves are dropped, which closes the connection.
///
/// Each message looks its agent up in the store:
/// - `RegistrationStart` adds an unknown agent on probation (a known one on probation stays there) and answers
///   `RegistrationReceived`;
/// - `AwaitingProbation` answers `RegistrationAccepted` with the saved token if the agent was approved while it
///   was offline, and finishes; if it is still on probation it answers `RegistrationReceived`;
/// - `Registered` is checked against the saved token and MAC address and answered with `RegistrationVerified`.
///
/// The task ends with an error when a message can not be decoded, an unexpected message arrives, the agent is
/// rejected or the stream breaks. Packets with an unknown message type are skipped.
pub fn verification_process(
    shared: Arc<Shared>,
    reader: Reader,
    writer: Writer,
) -> JoinHandle<Result<Verified, VerificationError>> {
    tokio::spawn(async move { verify(shared, reader, writer).await })
}

async fn verify(shared: Arc<Shared>, mut reader: Reader, mut writer: Writer) -> Result<Verified, VerificationError> {
    loop {
        let body: Vec<u8> = read_packet(&mut reader).await?;

        let message_type: Result<MessageType, ProtocolError> = body
            .first()
            .copied()
            .ok_or(ProtocolError::EmptyPacket)
            .and_then(MessageType::try_from);
        let Ok(message_type) = message_type else {
            tracing::warn!("expecting an authentication message");
            continue;
        };

        let verdict: Verdict = match message_type {
            MessageType::Registered => {
                let (token, message): (Secret, RegisteredMessage) = RegisteredMessage::decode(body)?;
                let mut manager = shared.agent_manager.lock().await;
                check_registration(&token, &message, &mut manager).await?
            }
            MessageType::RegistrationStart => {
                let message: RegistrationMessage = RegistrationMessage::decode(body)?;
                let mut manager = shared.agent_manager.lock().await;
                registration_start(&message, &mut manager).await?
            }
            MessageType::AwaitingProbation => {
                let message: AwaitingProbationMessage = AwaitingProbationMessage::decode(body)?;
                let mut manager = shared.agent_manager.lock().await;
                check_probation(&message, &mut manager).await?
            }
            actual => {
                tracing::warn!("unhandled message type {actual:?} in authentication stage, closing stream");
                return Err(ProtocolError::UnknownMessageType(actual.into()).into());
            }
        };

        match verdict {
            Verdict::Verified { agent_id, request_id } => {
                send(&mut writer, RegistrationVerifiedMessage { request_id }.encode()?).await?;
                return Ok(Verified { agent_id, reader, writer });
            }
            Verdict::Accepted { agent_id, request_id, token } => {
                let accepted: RegistrationAcceptedMessage =
                    RegistrationAcceptedMessage { request_id, token_hash: token };
                send(&mut writer, accepted.encode()?).await?;
                return Ok(Verified { agent_id, reader, writer });
            }
            Verdict::Probation { request_id } => {
                send(&mut writer, RegistrationReceivedMessage { request_id }.encode()?).await?;
            }
            Verdict::Rejected(agent_id) => {
                tracing::warn!("agent {agent_id} was rejected, closing stream");
                return Err(VerificationError::Rejected(agent_id));
            }
        }
    }
}

async fn registration_start(
    message: &RegistrationMessage,
    manager: &mut AgentStoreService,
) -> Result<Verdict, AgentStoreError> {
    let agent_id: AgentId = message.agent_id();
    let request_id: RequestId = message.package_id();

    let on_probation: Option<bool> = manager
        .find_agent(&agent_id)
        .await?
        .map(|agent| matches!(agent, AgentState::Unregistered(_)));
    match on_probation {
        None => {
            manager.add(agent_id, message.mac_address()).await?;
            Ok(Verdict::Probation { request_id })
        }
        Some(true) => Ok(Verdict::Probation { request_id }),
        Some(false) => Ok(Verdict::Rejected(agent_id)),
    }
}

/// The agent must be registered and send the saved token and MAC address.
async fn check_registration(
    token: &Secret,
    message: &RegisteredMessage,
    manager: &mut AgentStoreService,
) -> Result<Verdict, AgentStoreError> {
    let agent_id: AgentId = message.agent_id();

    Ok(match manager.find_agent(&agent_id).await? {
        Some(AgentState::Registered(agent))
            if agent.token == *token && agent.mac_address == message.mac_address() =>
        {
            Verdict::Verified { agent_id, request_id: message.package_id() }
        }
        _ => Verdict::Rejected(agent_id),
    })
}

async fn check_probation(
    message: &AwaitingProbationMessage,
    manager: &mut AgentStoreService,
) -> Result<Verdict, AgentStoreError> {
    let agent_id: AgentId = message.agent_id();
    let request_id: RequestId = message.package_id();

    Ok(match manager.find_agent(&agent_id).await? {
        Some(AgentState::Registered(agent)) if agent.mac_address == message.mac_address() => {
            Verdict::Accepted { agent_id, request_id, token: agent.token.clone() }
        }
        Some(AgentState::Unregistered(agent)) if agent.mac_address == message.mac_address() => {
            Verdict::Probation { request_id }
        }
        _ => Verdict::Rejected(agent_id),
    })
}

/// One packet body: the 4 length bytes are read and dropped.
async fn read_packet(reader: &mut Reader) -> Result<Vec<u8>, ProtocolError> {
    let mut len_bytes: [u8; 4] = [0u8; 4];
    reader
        .read_exact(&mut len_bytes)
        .await
        .map_err(|source| ProtocolError::StreamBroken { source })?;
    let len: usize = u32::from_be_bytes(len_bytes) as usize;

    let mut body: Vec<u8> = Vec::with_capacity(len);
    reader
        .take(len as u64)
        .read_to_end(&mut body)
        .await
        .map_err(|source| ProtocolError::StreamBroken { source })?;
    Ok(body)
}

async fn send(writer: &mut Writer, packet: Vec<u8>) -> Result<(), ProtocolError> {
    writer
        .write_all(&packet)
        .await
        .map_err(|source| ProtocolError::StreamBroken { source })?;
    writer.flush().await.map_err(|source| ProtocolError::StreamBroken { source })
}
