use crate::domain::error::TokenError;
use crate::{TOKEN_HASH_BYTE_SIZE, validate_token_hash};

#[test]
fn only_the_exact_size_is_a_valid_token_hash() {
    validate_token_hash(&[0u8; TOKEN_HASH_BYTE_SIZE]).unwrap();

    for size in [0, 1, TOKEN_HASH_BYTE_SIZE - 1, TOKEN_HASH_BYTE_SIZE + 1, TOKEN_HASH_BYTE_SIZE * 2] {
        assert!(matches!(
            validate_token_hash(&vec![0u8; size]),
            Err(TokenError::WrongSize { expected, actual }) if expected == TOKEN_HASH_BYTE_SIZE && actual == size
        ));
    }
}
