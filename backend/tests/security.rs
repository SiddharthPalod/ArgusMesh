//! Phase 4.4 security integration tests: tampered packet, replay, invalid sender.

use backend::crypto::encrypt::encrypt;
use backend::crypto::keys::NodeKeys;
use backend::crypto::sign::{sign_message, signature_to_bytes};
use backend::routing::envelope::{Envelope, Priority};
use backend::routing::router::Router;
use backend::storage::Store;
use backend::transport::test_sim::{SimConfig, SimTransport};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

struct MemStore(Arc<Mutex<Vec<Vec<u8>>>>);

impl Store for MemStore {
    fn persist(&self, _k: Vec<u8>, v: Vec<u8>) {
        self.0.lock().unwrap().push(v);
    }
    fn load_all(&self) -> Vec<Vec<u8>> {
        self.0.lock().unwrap().clone()
    }
    fn remove(&self, _k: Vec<u8>) {}
}

fn make_router() -> (
    Router<SimTransport, MemStore>,
    mpsc::Receiver<Vec<u8>>,
) {
    let (tx, rx) = mpsc::channel(100);
    let (_tx2, rx2) = mpsc::channel(100);
    let cfg = SimConfig { latency_ms: 0, drop_rate: 0.0 };
    let transport = SimTransport::new(tx, rx2, cfg);
    let store = MemStore(Arc::new(Mutex::new(vec![])));
    let keys = NodeKeys::generate();
    let sym_key = [5u8; 32];
    let router = Router::new(transport, store, keys, sym_key);
    (router, rx)
}

/// Build raw bytes for a valid envelope (encrypted + signed) with the given keys/sym_key.
fn build_valid_raw(
    keys: &NodeKeys,
    sym_key: &[u8; 32],
    payload: Vec<u8>,
) -> Vec<u8> {
    let mut env = Envelope::new("node_a".into(), Priority::Normal, payload);
    let (ct, nonce) = encrypt(sym_key, &env.payload).unwrap();
    env.payload = ct;
    env.nonce = nonce;
    env.sender_pubkey = keys.public_bytes();
    let mut sign_bytes = env.payload.clone();
    sign_bytes.extend_from_slice(&env.nonce);
    let sig = sign_message(keys.signing_key(), &sign_bytes);
    env.signature = signature_to_bytes(&sig).to_vec();
    bincode::serialize(&env).unwrap()
}

#[test]
fn tampered_packet_rejected() {
    let (mut router, _rx) = make_router();
    let keys = NodeKeys::generate();
    let sym_key = [5u8; 32];
    let valid_raw = build_valid_raw(&keys, &sym_key, b"hello".to_vec());

    // Tamper payload (ciphertext) so decrypt will fail
    let mut env: Envelope = bincode::deserialize(&valid_raw).unwrap();
    if !env.payload.is_empty() {
        env.payload[0] = env.payload[0].wrapping_add(1);
    }
    let tampered_raw = bincode::serialize(&env).unwrap();

    router.receive(&tampered_raw);
    // Message should not be forwarded (decrypt fails) -> queue stays empty
    assert!(router.next_outbound().is_none());
}

#[test]
fn replay_attack_rejected() {
    let (mut router, _rx) = make_router();
    let keys = NodeKeys::generate();
    let sym_key = [5u8; 32];
    let valid_raw = build_valid_raw(&keys, &sym_key, b"replay-me".to_vec());

    router.receive(&valid_raw);
    router.receive(&valid_raw); // replay

    // Only one message should be queued (replay dropped)
    let first = router.next_outbound();
    let second = router.next_outbound();
    assert!(first.is_some());
    assert!(second.is_none());
}

#[test]
fn invalid_sender_rejected() {
    let (mut router, _rx) = make_router();
    let signer_keys = NodeKeys::generate();
    let fake_sender_pubkey = NodeKeys::generate(); // different key
    let sym_key = [5u8; 32];

    let mut env = Envelope::new("attacker".into(), Priority::High, b"evil".to_vec());
    let (ct, nonce) = encrypt(&sym_key, &env.payload).unwrap();
    env.payload = ct;
    env.nonce = nonce;
    env.sender_pubkey = fake_sender_pubkey.public_bytes(); // claim to be fake_sender
    let mut sign_bytes = env.payload.clone();
    sign_bytes.extend_from_slice(&env.nonce);
    let sig = sign_message(signer_keys.signing_key(), &sign_bytes); // but sign with signer_keys
    env.signature = signature_to_bytes(&sig).to_vec();

    let raw = bincode::serialize(&env).unwrap();
    router.receive(&raw);

    // Verification fails (signature from signer_keys, pubkey from fake_sender_pubkey)
    assert!(router.next_outbound().is_none());
}

#[test]
fn valid_message_accepted_and_forwarded() {
    let (mut router, _rx) = make_router();
    let keys = NodeKeys::generate();
    let sym_key = [5u8; 32];
    let valid_raw = build_valid_raw(&keys, &sym_key, b"valid".to_vec());

    router.receive(&valid_raw);
    let out = router.next_outbound();
    assert!(out.is_some());
    let env: Envelope = bincode::deserialize(out.unwrap().as_slice()).unwrap();
    // Router forwards with plaintext payload after verify+decrypt
    assert_eq!(env.payload.as_slice(), b"valid");
}
