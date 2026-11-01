use crate::domain::error::CertError;
use crate::generate_certs;

#[test]
fn generates_pem_cert_and_key() {
    let certs = generate_certs(&["receiver.lan".into(), "10.0.0.5".into()]).unwrap();
    assert!(certs.cert_pem.content.starts_with(b"-----BEGIN CERTIFICATE-----"));
    assert!(certs.key_pem.content.starts_with(b"-----BEGIN PRIVATE KEY-----"));
}

#[test]
fn every_call_produces_a_new_key() {
    let names = ["receiver.lan".to_string()];
    let first = generate_certs(&names).unwrap();
    let second = generate_certs(&names).unwrap();
    assert_ne!(first.key_pem.content, second.key_pem.content);
}

#[test]
fn rejects_empty_subject_names() {
    assert!(matches!(generate_certs(&[]), Err(CertError::NoSubjectNames)));
}
