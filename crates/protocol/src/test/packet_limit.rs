use std::time::Duration;

use fleet_crypto::TOKEN_HASH_BYTE_SIZE;
use fleet_model::{AgentId, MacAddress, RequestId};

use crate::agent_receiver::{HeartBeatMessage, MessageType, RegisteredMessage, RegistrationMessage};
use crate::protocol::{MAX_PACKET_SIZE, Package, UnverifiedPackage};
use crate::{ParseError, ProtocolError};

fn heart_beat_with_cores(cores: usize) -> HeartBeatMessage {
    HeartBeatMessage {
        agent_id: AgentId::new(),
        package_id: RequestId::new(),
        mac_address: MacAddress([1; 6]),
        timestamp: Duration::from_secs(1),
        uptime: Duration::from_secs(1),
        cpu_cores: cores as u32,
        cpu_usage_percent: 0.0,
        cpu_core_usage_percent: vec![0.0; cores],
        load_average: [0.0; 3],
        memory_total_bytes: 0,
        memory_used_bytes: 0,
    }
}

#[test]
fn encoding_refuses_a_packet_over_the_limit_and_accepts_one_under_it() {
    let token_hash = [1u8; TOKEN_HASH_BYTE_SIZE];

    let too_big = heart_beat_with_cores(MAX_PACKET_SIZE / 4 + 1_000).encode(&token_hash);
    assert!(matches!(too_big, Err(ProtocolError::PacketTooLarge { max: MAX_PACKET_SIZE, .. })));

    let fits = heart_beat_with_cores(MAX_PACKET_SIZE / 4 - 1_000).encode(&token_hash).unwrap();
    assert!(fits.len() - 4 <= MAX_PACKET_SIZE);
}

#[test]
fn decoding_refuses_a_body_over_the_limit_before_anything_else() {
    // the length of the body is the size of the message
    let body: Vec<u8> = vec![MessageType::RegistrationStart.as_byte(); MAX_PACKET_SIZE + 1];
    assert!(matches!(
        RegistrationMessage::decode(body.clone()),
        Err(ProtocolError::PacketTooLarge { size, max: MAX_PACKET_SIZE }) if size == MAX_PACKET_SIZE + 1
    ));

    let mut token_body: Vec<u8> = body;
    token_body[0] = MessageType::Registered.as_byte();
    assert!(matches!(
        RegisteredMessage::decode(token_body),
        Err(ProtocolError::PacketTooLarge { max: MAX_PACKET_SIZE, .. })
    ));
}

#[test]
fn a_body_exactly_at_the_limit_is_not_rejected_for_its_size() {
    let mut body: Vec<u8> = vec![0xFF; MAX_PACKET_SIZE];
    body[0] = MessageType::RegistrationStart.as_byte();

    // it gets past the size check and fails later, on the garbage payload
    assert!(matches!(
        RegistrationMessage::decode(body),
        Err(ProtocolError::Parse(ParseError::Decode { .. }))
    ));
}
