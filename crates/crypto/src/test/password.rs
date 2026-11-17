use crate::{constant_time_eq, hash_password, verify_password};

#[test]
fn a_password_matches_only_its_own_hash_and_every_hash_has_its_own_salt() {
    let hash: Vec<u8> = hash_password(b"correct horse").unwrap();
    assert!(hash.starts_with(b"$argon2id$"));
    assert!(verify_password(b"correct horse", &hash));
    assert!(!verify_password(b"wrong", &hash));
    assert!(!verify_password(b"correct horse", b"not a hash"));
    assert!(!verify_password(b"correct horse", &[0xff, 0xfe]));
    assert_ne!(hash, hash_password(b"correct horse").unwrap());
}

#[test]
fn constant_time_eq_compares_content_and_length() {
    assert!(constant_time_eq(b"abc", b"abc"));
    assert!(constant_time_eq(b"", b""));
    assert!(!constant_time_eq(b"abc", b"abd"));
    assert!(!constant_time_eq(b"abc", b"ab"));
}
