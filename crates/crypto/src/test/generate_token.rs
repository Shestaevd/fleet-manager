use crate::domain::error::TokenError;
use crate::{TOKEN_BYTE_SIZE, generate_secure_token, validate_token};

#[test]
fn token_is_always_the_same_length() {
    for _ in 0..1000 {
        let token = generate_secure_token();
        assert_eq!(token.len(), TOKEN_BYTE_SIZE);
        validate_token(&token).unwrap();
    }
}

#[test]
fn token_is_ascii_letters_and_digits_and_every_token_is_different() {
    let token = generate_secure_token();
    assert!(token.iter().all(u8::is_ascii_alphanumeric));
    assert_ne!(token, generate_secure_token());
}

#[test]
fn validation_rejects_every_other_length() {
    for size in [0, 1, TOKEN_BYTE_SIZE - 1, TOKEN_BYTE_SIZE + 1, TOKEN_BYTE_SIZE * 2] {
        assert!(matches!(
            validate_token(&vec![b'a'; size]),
            Err(TokenError::WrongSize { expected, actual }) if expected == TOKEN_BYTE_SIZE && actual == size
        ));
    }
}
