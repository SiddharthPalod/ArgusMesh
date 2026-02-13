use crate::transport::Transport;
use crate::storage::Store;
use crate::core::alert_index::add_alert;

use crate::crypto::encrypt::{encrypt, decrypt, SymKey};
use crate::crypto::keys::NodeKeys;
use crate::crypto::replay::ReplayGuard;
use crate::crypto::sign::{
    sign_message, verify_message,
    signature_from_bytes, signature_to_bytes,
};

use ed25519_dalek::VerifyingKey;
use std::sync::Arc;

use super::{
    dedup::DedupCache,
    envelope::Envelope,
    priority_queue::PriorityQueue,
    propagation::should_forward,
};

pub type ReceiveSink = Box<dyn FnMut(Envelope) + Send>;

pub struct Router<T: Transport, S: Store> {
    pub transport: Arc<T>,
    pub store: S,

    dedup: DedupCache,
    queue: PriorityQueue,

    keys: NodeKeys,
    sym_key: SymKey,
    replay: ReplayGuard,

    /// When set, called instead of global add_alert (e.g. for sim per-node index).
    receive_sink: Option<ReceiveSink>,
}

impl<T: Transport, S: Store> Router<T, S> {
    pub fn new(
        transport: Arc<T>,
        store: S,
        keys: NodeKeys,
        sym_key: SymKey,
    ) -> Self {
        Self {
            transport,
            store,
            dedup: DedupCache::new(50_000),
            queue: PriorityQueue::new(),
            keys,
            sym_key,
            replay: ReplayGuard::new(50_000),
            receive_sink: None,
        }
    }

    /// Use a custom sink instead of global add_alert when receiving (e.g. sim per-node list).
    pub fn set_receive_sink(&mut self, sink: ReceiveSink) {
        self.receive_sink = Some(sink);
    }

    /// 🔐 secure local enqueue. Returns Err if encryption fails (no panic).
    pub fn enqueue_local(&mut self, mut env: Envelope) -> Result<(), String> {
        let (ct, nonce) = encrypt(&self.sym_key, &env.payload)
            .map_err(|_| "encrypt failed".to_string())?;

        env.payload = ct;
        env.nonce = nonce;
        env.sender_pubkey = self.keys.public_bytes();

        let mut sign_bytes = env.payload.clone();
        sign_bytes.extend_from_slice(&env.nonce);
        let sig = sign_message(self.keys.signing_key(), &sign_bytes);
        env.signature = signature_to_bytes(&sig).to_vec();

        self.queue.push(env);
        Ok(())
    }

    // 🔐 secure receive path
    pub fn receive(&mut self, raw: &[u8]) {
        let mut env: Envelope = match bincode::deserialize(raw) {
            Ok(e) => e,
            Err(_) => return,
        };

        // dedup first
        if self.dedup.seen(env.msg_id) {
            return;
        }

        // verify signature
        let pk = match VerifyingKey::from_bytes(&env.sender_pubkey) {
            Ok(k) => k,
            Err(_) => return,
        };

        let mut sign_bytes = env.payload.clone();
        sign_bytes.extend_from_slice(&env.nonce);

        let sig_bytes: [u8; 64] = match env.signature.as_slice().try_into() {
            Ok(b) => b,
            Err(_) => return,
        };
        let sig = signature_from_bytes(&sig_bytes);

        if !verify_message(&pk, &sign_bytes, &sig) {
            return;
        }

        // replay defense
        if !self.replay.check_and_insert(&sign_bytes) {
            return;
        }

        // decrypt payload
        let pt = match decrypt(&self.sym_key, &env.nonce, &env.payload) {
            Ok(p) => p,
            Err(_) => return,
        };

        env.payload = pt;

        if !should_forward(&env) {
            return;
        }
        if let Some(ref mut f) = self.receive_sink {
            f(env.clone());
        } else {
            add_alert(env.clone());
        }

        env.next_hop();
        self.queue.push(env);
    }

    pub fn next_outbound(&mut self) -> Option<Vec<u8>> {
        if let Some(env) = self.queue.pop() {
            bincode::serialize(&env).ok()
        } else {
            None
        }
    }

    pub async fn tick(&mut self) {
        if let Some(env) = self.queue.pop() {
            let bytes = bincode::serialize(&env).unwrap();

            if self.transport.is_connected() {
                let _ = self.transport.as_ref().send(bytes).await;
            } else {
                self.store.persist(env.msg_id.as_bytes().to_vec(), bytes);
            }
        }
    }

    pub async fn flush_store(&mut self) {
        if !self.transport.is_connected() {
            return;
        }

        for bytes in self.store.load_all() {
            let _ = self.transport.as_ref().send(bytes.clone()).await;
        }
    }
}
