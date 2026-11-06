use fleet_crypto::TOKEN_HASH_BYTE_SIZE;
use fleet_model::{AgentId, RequestId};
use serde::Serialize;

use crate::agent_receiver::{
    AgentReceiverPackage, MessageType, ReceiverAgentPackage, RegistrationAcceptedMessage,
    RegistrationFinishedMessage, RegistrationMessage, RegistrationProbationMessage,
};
use crate::protocol::{Package, UnverifiedPackage};
use crate::{ParseError, ProtocolError, TokenHash};

fn registration() -> RegistrationMessage {
    RegistrationMessage::new(AgentId::new())
}

fn accepted() -> RegistrationAcceptedMessage {
    RegistrationAcceptedMessage {
        request_id: RequestId::new(),
        token_hash: TokenHash::new(vec![5u8; TOKEN_HASH_BYTE_SIZE]).unwrap(),
    }
}

#[test]
fn every_registration_message_roundtrips_through_encode_and_decode() {
    let registration = registration();
    assert_eq!(RegistrationMessage::decode(&registration.encode().unwrap()).unwrap(), registration);

    let probation = RegistrationProbationMessage { request_id: RequestId::new() };
    assert_eq!(RegistrationProbationMessage::decode(&probation.encode().unwrap()).unwrap(), probation);

    let accepted = accepted();
    assert_eq!(RegistrationAcceptedMessage::decode(&accepted.encode().unwrap()).unwrap(), accepted);

}

#[test]
fn a_packet_without_a_token_is_length_then_type_then_payload() {
    let message = registration();
    let packet = message.encode().unwrap();
    let payload: Vec<u8> = message.clone().into();

    assert_eq!(packet.len(), 4 + 1 + payload.len());
    assert_eq!(u32::from_be_bytes(packet[..4].try_into().unwrap()) as usize, packet.len() - 4);
    assert_eq!(packet[4], MessageType::Registration.as_byte());
    assert_eq!(&packet[5..], payload.as_slice());
}

#[test]
fn each_message_uses_its_own_type_byte_and_exposes_its_ids() {
    let probation = RegistrationProbationMessage { request_id: RequestId::new() };
    assert_eq!(probation.encode().unwrap()[4], MessageType::RegistrationProbation.as_byte());
    assert_eq!(accepted().encode().unwrap()[4], MessageType::RegistrationAccepted.as_byte());

    let finished = RegistrationFinishedMessage::new(AgentId::new());
    assert_eq!(
        finished.encode(&[1u8; TOKEN_HASH_BYTE_SIZE]).unwrap()[4],
        MessageType::RegistrationFinished.as_byte()
    );
    assert_eq!(finished.agent_id(), finished.agent_id);
    assert_eq!(finished.package_id(), finished.package_id);
    assert_ne!(finished.package_id, RegistrationFinishedMessage::new(finished.agent_id).package_id);

    let registration = registration();
    assert_eq!(registration.agent_id(), registration.agent_id);
    assert_eq!(registration.package_id(), registration.package_id);
    assert_eq!(probation.request_id(), probation.request_id);
}

#[test]
fn decoding_rejects_broken_packets() {
    let packet = registration().encode().unwrap();

    assert!(matches!(RegistrationMessage::decode(&[]), Err(ProtocolError::TruncatedPacket)));
    assert!(matches!(RegistrationMessage::decode(&packet[..3]), Err(ProtocolError::TruncatedPacket)));
    assert!(matches!(
        RegistrationMessage::decode(&packet[..packet.len() - 1]),
        Err(ProtocolError::LengthMismatch { .. })
    ));
    let mut with_extra = packet.clone();
    with_extra.push(0);
    assert!(matches!(RegistrationMessage::decode(&with_extra), Err(ProtocolError::LengthMismatch { .. })));
    assert!(matches!(RegistrationMessage::decode(&[0, 0, 0, 0]), Err(ProtocolError::EmptyPacket)));

    let mut garbage = vec![0, 0, 0, 3, MessageType::Registration.as_byte(), 0xFF, 0xFF];
    assert!(matches!(
        RegistrationMessage::decode(&garbage),
        Err(ProtocolError::Parse(ParseError::Decode { .. }))
    ));
    garbage[4] = 0x00;
    assert!(matches!(RegistrationMessage::decode(&garbage), Err(ProtocolError::UnexpectedMessageType { .. })));
}

#[test]
fn a_packet_of_another_message_type_is_rejected() {
    let probation = RegistrationProbationMessage { request_id: RequestId::new() }.encode().unwrap();
    assert!(matches!(
        RegistrationMessage::decode(&probation),
        Err(ProtocolError::UnexpectedMessageType { expected: 0xE0, actual: 0xE1 })
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
    let mut packet = ((payload.len() + 1) as u32).to_be_bytes().to_vec();
    packet.push(MessageType::RegistrationAccepted.as_byte());
    packet.extend(payload);

    assert!(matches!(
        RegistrationAcceptedMessage::decode(&packet),
        Err(ProtocolError::Parse(ParseError::TokenHash { .. }))
    ));
}

#[test]
fn a_token_hash_checks_its_size_and_does_not_print_its_content() {
    assert!(TokenHash::new(vec![0u8; TOKEN_HASH_BYTE_SIZE - 1]).is_err());
    let hash = TokenHash::new(vec![42u8; TOKEN_HASH_BYTE_SIZE]).unwrap();
    assert_eq!(hash.as_bytes(), vec![42u8; TOKEN_HASH_BYTE_SIZE].as_slice());
    assert!(!format!("{hash:?}").contains("42"));
    assert_eq!(hash.into_bytes().len(), TOKEN_HASH_BYTE_SIZE);
}

#[test]
fn registration_finished_carries_the_token_hash_as_proof() {
    let finished = RegistrationFinishedMessage::new(AgentId::new());
    let token_hash = vec![9u8; TOKEN_HASH_BYTE_SIZE];
    let packet = finished.encode(&token_hash).unwrap();

    assert_eq!(u32::from_be_bytes(packet[..4].try_into().unwrap()) as usize, packet.len() - 4);
    assert_eq!(packet[4], MessageType::RegistrationFinished.as_byte());
    assert_eq!(&packet[5..5 + TOKEN_HASH_BYTE_SIZE], token_hash.as_slice());
    assert_eq!(
        RegistrationFinishedMessage::try_from(packet[5 + TOKEN_HASH_BYTE_SIZE..].to_vec()).unwrap(),
        finished
    );

    for size in [0, TOKEN_HASH_BYTE_SIZE - 1, TOKEN_HASH_BYTE_SIZE + 1] {
        assert!(matches!(finished.encode(&vec![0u8; size]), Err(ProtocolError::WrongTokenSize)));
    }
}
