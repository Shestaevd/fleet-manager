use std::time::Duration;

use fleet_model::{AgentId, RequestId, Telemetry};

use crate::agent_receiver::HeartBeatMessage;
use crate::domain::error::{ParseError, ProtocolError};
use fleet_crypto::TOKEN_HASH_BYTE_SIZE;
use crate::protocol::Package;

fn sample() -> HeartBeatMessage {
    HeartBeatMessage {
        agent_id: AgentId::new(),
        package_id: RequestId::new(),
        timestamp: Duration::from_secs(1_700_000_000),
        uptime: Duration::from_secs(3600),
        cpu_cores: 4,
        cpu_usage_percent: 12.5,
        cpu_core_usage_percent: vec![10.0, 15.0, 5.0, 20.0],
        load_average: [0.5, 0.4, 0.3],
        memory_total_bytes: 16 << 30,
        memory_used_bytes: 4 << 30,
    }
}

#[test]
fn heart_beat_roundtrips_through_bytes() {
    let message = sample();
    let bytes: Vec<u8> = message.clone().into();
    assert_eq!(HeartBeatMessage::try_from(bytes).unwrap(), message);
}

#[test]
fn garbage_bytes_are_a_parse_error() {
    assert!(matches!(
        HeartBeatMessage::try_from(vec![0xFF; 3]),
        Err(ParseError::Decode { .. })
    ));
}

#[test]
fn encoded_packet_is_length_then_type_then_token_hash_then_payload() {
    let message = sample();
    let token_hash = vec![7u8; TOKEN_HASH_BYTE_SIZE];
    let encoded = message.encode(&token_hash).unwrap();

    let len = u32::from_be_bytes(encoded[..4].try_into().unwrap()) as usize;
    assert_eq!(len, encoded.len() - 4);
    assert_eq!(encoded[4], 0xF0);
    assert_eq!(&encoded[5..5 + TOKEN_HASH_BYTE_SIZE], token_hash.as_slice());
    assert_eq!(HeartBeatMessage::try_from(encoded[5 + TOKEN_HASH_BYTE_SIZE..].to_vec()).unwrap(), message);
}

#[test]
fn encoding_rejects_a_token_hash_of_the_wrong_size() {
    for size in [0, TOKEN_HASH_BYTE_SIZE - 1, TOKEN_HASH_BYTE_SIZE + 1] {
        assert!(matches!(
            sample().encode(&vec![0u8; size]),
            Err(ProtocolError::WrongTokenSize)
        ));
    }
}

#[test]
fn heart_beat_converts_to_telemetry_with_the_same_measurements() {
    let message = sample();
    let telemetry: Telemetry = message.clone().into();

    assert_eq!(telemetry.timestamp, message.timestamp);
    assert_eq!(telemetry.uptime, message.uptime);
    assert_eq!(telemetry.cpu_cores, message.cpu_cores);
    assert_eq!(telemetry.cpu_core_usage_percent, message.cpu_core_usage_percent);
    assert_eq!(telemetry.load_average, message.load_average);
    assert_eq!(telemetry.memory_used_bytes, message.memory_used_bytes);
}
