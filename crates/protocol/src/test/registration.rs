use fleet_crypto::TOKEN_HASH_BYTE_SIZE;
use fleet_crypto::Secret;
use fleet_model::{AgentId, MacAddress, RequestId};
use serde::Serialize;

use crate::agent_receiver::{
    AgentReceiverPackage, AwaitingProbationMessage, MessageType, ReceiverAgentPackage, RegisteredMessage,
    RegistrationAcceptedMessage, RegistrationMessage, RegistrationReceivedMessage,
};
use crate::protocol::{Package, UnverifiedPackage};
use crate::{ParseError, ProtocolError};

const MAC: MacAddress = MacAddress([0x00, 0x1a, 0xb2, 0x0c, 0xff, 0x07]);
const TOKEN_HASH: [u8; TOKEN_HASH_BYTE_SIZE] = [4u8; TOKEN_HASH_BYTE_SIZE];

/// What `decode` takes: the packet without its length prefix.
fn body(packet: Vec<u8>) -> Vec<u8> {
    packet[4..].to_vec()
}

fn registration() -> RegistrationMessage {
    RegistrationMessage::new(AgentId::new(), MAC)
}

fn accepted() -> RegistrationAcceptedMessage {
    RegistrationAcceptedMessage {
        request_id: RequestId::new(),
        token_hash: Secret { content: vec![5u8; TOKEN_HASH_BYTE_SIZE] },
    }
}

#[test]
fn every_token_free_message_roundtrips_through_encode_and_decode() {
    let registration = registration();
    assert_eq!(RegistrationMessage::decode(body(registration.encode().unwrap())).unwrap(), registration);

    let awaiting = AwaitingProbationMessage::new(AgentId::new(), MAC);
    assert_eq!(AwaitingProbationMessage::decode(body(awaiting.encode().unwrap())).unwrap(), awaiting);

    let probation = RegistrationReceivedMessage { request_id: RequestId::new() };
    assert_eq!(RegistrationReceivedMessage::decode(body(probation.encode().unwrap())).unwrap(), probation);

    let accepted = accepted();
    assert_eq!(RegistrationAcceptedMessage::decode(body(accepted.encode().unwrap())).unwrap(), accepted);
}

#[test]
fn a_packet_without_a_token_is_length_then_type_then_payload_and_the_mac_is_in_the_payload() {
    let message = registration();
    let packet = message.encode().unwrap();
    let payload: Vec<u8> = message.clone().into();

    assert_eq!(packet.len(), 4 + 1 + payload.len());
    assert_eq!(u32::from_be_bytes(packet[..4].try_into().unwrap()) as usize, packet.len() - 4);
    assert_eq!(packet[4], MessageType::RegistrationStart.as_byte());
    assert_eq!(&packet[5..], payload.as_slice());
    assert_eq!(RegistrationMessage::decode(body(packet)).unwrap().mac_address, MAC);
}

#[test]
fn every_agent_message_exposes_its_ids_and_its_mac_address() {
    let agent_id = AgentId::new();
    let registration = RegistrationMessage::new(agent_id, MAC);
    let awaiting = AwaitingProbationMessage::new(agent_id, MAC);
    let registered = RegisteredMessage::new(agent_id, MAC);

    assert_eq!(registration.agent_id(), agent_id);
    assert_eq!(registration.package_id(), registration.package_id);
    assert_eq!(registration.mac_address(), MAC);
    assert_eq!(awaiting.mac_address(), MAC);
    assert_eq!(registered.mac_address(), MAC);
    assert_ne!(registered.package_id, RegisteredMessage::new(agent_id, MAC).package_id);

    let probation = RegistrationReceivedMessage { request_id: RequestId::new() };
    assert_eq!(probation.request_id(), probation.request_id);
    assert_eq!(probation.encode().unwrap()[4], MessageType::RegistrationReceived.as_byte());
    assert_eq!(accepted().encode().unwrap()[4], MessageType::RegistrationAccepted.as_byte());
}

#[test]
fn decoding_rejects_broken_bodies() {
    assert!(matches!(RegistrationMessage::decode(Vec::new()), Err(ProtocolError::EmptyPacket)));

    let mut garbage = vec![MessageType::RegistrationStart.as_byte(), 0xFF, 0xFF];
    assert!(matches!(
        RegistrationMessage::decode(garbage.clone()),
        Err(ProtocolError::Parse(ParseError::Decode { .. }))
    ));
    garbage[0] = 0x00;
    assert!(matches!(RegistrationMessage::decode(garbage), Err(ProtocolError::UnexpectedMessageType { .. })));
}

#[test]
fn a_body_without_the_mac_address_in_its_message_is_rejected() {
    // a registration payload cut off before the mac address
    let full: Vec<u8> = registration().into();
    let mut packet_body = vec![MessageType::RegistrationStart.as_byte()];
    packet_body.extend_from_slice(&full[..full.len() - 6]);

    assert!(matches!(
        RegistrationMessage::decode(packet_body),
        Err(ProtocolError::Parse(ParseError::Decode { .. }))
    ));
}

#[test]
fn a_packet_of_another_message_type_is_rejected() {
    let awaiting = AwaitingProbationMessage::new(AgentId::new(), MAC).encode().unwrap();
    assert!(matches!(
        RegistrationMessage::decode(body(awaiting)),
        Err(ProtocolError::UnexpectedMessageType { expected: 0xA0, actual: 0xA2 })
    ));
}

#[test]
fn registration_accepted_with_a_token_hash_of_the_wrong_size_is_rejected() {
    #[derive(Serialize)]
    struct RawHash {
        hash: Vec<u8>,
    }
    #[derive(Serialize)]
    struct RawAccepted {
        request_id: RequestId,
        token_hash: RawHash,
    }

    let payload = postcard::to_stdvec(&RawAccepted {
        request_id: RequestId::new(),
        token_hash: RawHash { hash: vec![1, 2, 3] },
    })
    .unwrap();
    let mut packet_body = vec![MessageType::RegistrationAccepted.as_byte()];
    packet_body.extend(payload);

    assert!(matches!(
        RegistrationAcceptedMessage::decode(packet_body),
        Err(ProtocolError::Parse(ParseError::TokenHash { .. }))
    ));
}

#[test]
fn a_secret_token_hash_does_not_print_its_content() {
    let hash: Secret = Secret { content: vec![42u8; TOKEN_HASH_BYTE_SIZE] };
    assert!(!format!("{hash:?}").contains("42"));
}

#[test]
fn registered_carries_the_token_hash_in_the_header() {
    let registered = RegisteredMessage::new(AgentId::new(), MAC);
    let packet = registered.encode(&TOKEN_HASH).unwrap();
    assert_eq!(packet[4], MessageType::Registered.as_byte());
    assert_eq!(&packet[5..5 + TOKEN_HASH_BYTE_SIZE], TOKEN_HASH.as_slice());
    let decoded = RegisteredMessage::try_from(packet[5 + TOKEN_HASH_BYTE_SIZE..].to_vec()).unwrap();
    assert_eq!(decoded, registered);
    assert_eq!(decoded.mac_address, MAC);

    for size in [0, TOKEN_HASH_BYTE_SIZE - 1, TOKEN_HASH_BYTE_SIZE + 1] {
        assert!(matches!(registered.encode(&vec![0u8; size]), Err(ProtocolError::WrongTokenSize)));
    }
}

#[test]
fn a_machine_without_a_mac_address_sends_all_zeros_and_it_is_recognisable() {
    let message = AwaitingProbationMessage::new(AgentId::new(), MacAddress::UNSPECIFIED);
    let decoded = AwaitingProbationMessage::decode(body(message.encode().unwrap())).unwrap();
    assert!(decoded.mac_address().is_unspecified());
    assert!(!MAC.is_unspecified());
}

#[test]
fn every_message_type_byte_maps_back_to_itself() {
    for message_type in [
        MessageType::RegistrationStart,
        MessageType::RegistrationReceived,
        MessageType::RegistrationAccepted,
        MessageType::RegistrationVerified,
        MessageType::AwaitingProbation,
        MessageType::HeartBeat,
        MessageType::FileMetaUpdate,
        MessageType::MoveWindow,
        MessageType::Registered,
    ] {
        assert_eq!(MessageType::try_from(message_type.as_byte()).unwrap(), message_type);
    }
}

#[test]
fn token_packets_decode_into_the_token_hash_and_the_message() {
    let registered = RegisteredMessage::new(AgentId::new(), MAC);
    let (token_hash, decoded) = RegisteredMessage::decode(body(registered.encode(&TOKEN_HASH).unwrap())).unwrap();
    assert_eq!(token_hash.content.as_slice(), TOKEN_HASH.as_slice());
    assert_eq!(decoded, registered);
}

#[test]
fn a_token_packet_that_is_too_short_or_of_another_type_is_rejected() {
    let packet = RegisteredMessage::new(AgentId::new(), MAC).encode(&TOKEN_HASH).unwrap();

    let mut cut = body(packet.clone());
    cut.truncate(1 + TOKEN_HASH_BYTE_SIZE - 1);
    assert!(matches!(RegisteredMessage::decode(cut), Err(ProtocolError::TruncatedPacket)));
    assert!(matches!(RegisteredMessage::decode(Vec::new()), Err(ProtocolError::EmptyPacket)));
    assert!(matches!(
        crate::agent_receiver::HeartBeatMessage::decode(body(packet)),
        Err(ProtocolError::UnexpectedMessageType { expected, actual })
            if expected == MessageType::HeartBeat.as_byte() && actual == MessageType::Registered.as_byte()
    ));
}
