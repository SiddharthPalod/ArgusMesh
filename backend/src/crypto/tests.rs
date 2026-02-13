use crate::crypto::encrypt::{decrypt, encrypt};
use crate::crypto::keys::NodeKeys;
use crate::crypto::replay::ReplayGuard;
use crate::crypto::sign::*;

#[test]
fn sign_and_verify_ok(){
    let keys = NodeKeys::generate();
    let msg = b"mesh-alert";
    let sig = sign_message(keys.signing_key(), msg);
    assert!(verify_message(keys.verifying_key(), msg, &sig));
}

#[test]
fn tampered_message_rejected(){
    let keys = NodeKeys::generate();
    let msg = b"mesh-alert";
    let sig = sign_message(keys.signing_key(), msg);
    let bad = b"mesh-alert-modified";
    assert!(!verify_message(keys.verifying_key(), bad, &sig));
}

#[test]
fn encryption_roundtrip() {
    let key = [9u8; 32];
    let msg = b"secret payload";

    let (ct, nonce) = encrypt(&key, msg).unwrap();
    let pt = decrypt(&key, &nonce, &ct).unwrap();

    assert_eq!(pt, msg);
}

#[test]
fn invalid_ciphertext_rejected() {
    let key = [3u8; 32];
    let msg = b"secret";

    let (mut ct, nonce) = encrypt(&key, msg).unwrap();
    ct[0] ^= 1;

    assert!(decrypt(&key, &nonce, &ct).is_err());
}

#[test]
fn replay_attack_detected() {
    let mut guard = ReplayGuard::new(100);
    let msg = b"packet";

    assert!(guard.check_and_insert(msg));
    assert!(!guard.check_and_insert(msg));
}