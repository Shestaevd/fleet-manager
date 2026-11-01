use std::sync::Arc;
use std::time::Duration;

use fleet_crypto::{Secret, TOKEN_BYTE_SIZE, generate_secure_token};
use fleet_protocol::agent_receiver::HeartBeatMessage;
use tokio::io::{AsyncReadExt, duplex};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::domain::config::AgentConfig;
use crate::domain::model::Shared;
use crate::process::session_process::heart_beat_process;

fn shared() -> Arc<Shared> {
    let config: AgentConfig = toml::from_str(
        "agent_id = \"9d3b8f9e-6f2a-4c1e-9a55-2c7f0a1b3d44\"\nreceiver_url = \"tls://127.0.0.1:7000\"\nheart_beat_interval_ms = 30\n",
    )
    .unwrap();
    Arc::new(Shared {
        certificate: Secret { content: Vec::new() },
        token: Secret { content: generate_secure_token() },
        config,
    })
}

#[tokio::test]
async fn sends_length_prefixed_heart_beats_and_stops_when_the_stream_breaks() {
    let shared = shared();
    let (writer, mut reader) = duplex(64 * 1024);
    let handle = heart_beat_process(shared.clone(), Arc::new(Mutex::new(writer)));

    let mut len = [0u8; 4];
    timeout(Duration::from_secs(2), reader.read_exact(&mut len)).await.unwrap().unwrap();
    let mut body = vec![0u8; u32::from_be_bytes(len) as usize];
    reader.read_exact(&mut body).await.unwrap();

    assert_eq!(body[0], 0xF0);
    assert_eq!(&body[1..1 + TOKEN_BYTE_SIZE], shared.token.content.as_slice());
    let message = HeartBeatMessage::try_from(body[1 + TOKEN_BYTE_SIZE..].to_vec()).unwrap();
    assert_eq!(message.agent_id, shared.config.agent_id);
    assert!(message.cpu_cores > 0);

    drop(reader);
    timeout(Duration::from_secs(2), handle).await.unwrap().unwrap();
}
