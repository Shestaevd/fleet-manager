use crate::{TOKEN_BYTE_SIZE, TOKEN_HASH_BYTE_SIZE, generate_secure_token, hash_token, validate_token_hash};

#[test]
fn tokens_are_always_the_same_length_ascii_and_different_every_time() {
    let first = generate_secure_token();
    assert_eq!(first.len(), TOKEN_BYTE_SIZE);
    assert!(first.iter().all(u8::is_ascii_alphanumeric));
    for _ in 0..1000 {
        assert_eq!(generate_secure_token().len(), TOKEN_BYTE_SIZE);
    }
    assert_ne!(first, generate_secure_token());
}

#[test]
fn a_token_hash_is_always_token_hash_byte_size_long_and_valid() {
    for token in [generate_secure_token(), Vec::new(), vec![0u8; 1], vec![7u8; 10_000]] {
        let hash = hash_token(&token);
        assert_eq!(hash.len(), TOKEN_HASH_BYTE_SIZE);
        validate_token_hash(&hash).unwrap();
    }
}

#[test]
fn hashing_is_deterministic_and_matches_the_sha256_test_vector() {
    assert_eq!(hash_token(b"abc"), hash_token(b"abc"));
    assert_ne!(hash_token(b"abc"), hash_token(b"abd"));

    let expected: [u8; 32] = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
        0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    ];
    assert_eq!(hash_token(b"abc"), expected);
}
