use zeroize::Zeroize;

use crate::crypto::keys::NodeKeys;

pub const NODE_KEYPAIR_KEY: &str = "node_keypair";

pub struct StoredKeypair {
    pub secret: Vec<u8>,
    pub public: Vec<u8>,
}

impl StoredKeypair {
    pub fn new(secret: Vec<u8>, public: Vec<u8>) -> Self {
        Self { secret, public }
    }
}

impl Drop for StoredKeypair {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

/// Persist node keypair to sled DB. Overwrites existing key.
pub fn persist_keys(db: &sled::Db, keys: &NodeKeys) -> Result<(), ()> {
    let mut buf = [0u8; 64];
    buf[0..32].copy_from_slice(&keys.secret_bytes());
    buf[32..64].copy_from_slice(&keys.public_bytes());
    db.insert(NODE_KEYPAIR_KEY, buf.as_slice()).map_err(|_| ())?;
    buf.zeroize();
    Ok(())
}

/// Load node keypair from sled DB. Returns None if not found or invalid.
pub fn load_keys(db: &sled::Db) -> Option<NodeKeys> {
    let buf = db.get(NODE_KEYPAIR_KEY).ok()??;
    let bytes: [u8; 64] = buf.as_ref().try_into().ok()?;
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&bytes[0..32]);
    let keys = NodeKeys::from_secret_bytes(&secret);
    secret.zeroize();
    Some(keys)
}

