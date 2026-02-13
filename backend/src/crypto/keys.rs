use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;

pub struct NodeKeys {
    signing: SigningKey,
    verify: VerifyingKey,
}

impl NodeKeys {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let signing = SigningKey::from_bytes(&bytes);
        let verify = signing.verifying_key();
        Self{
            signing,
            verify,
        }
    }

    pub fn signing_key(&self) -> &SigningKey{
        &self.signing
    }

    pub fn verifying_key(&self) -> &VerifyingKey{
        &self.verify
    }

    pub fn public_bytes(&self) -> [u8; 32]{
        self.verify.to_bytes()
    }

    /// Secret key bytes for secure persistence. Caller must zeroize when done.
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing.to_bytes()
    }

    /// Load node keys from persisted secret bytes (e.g. from secure storage).
    pub fn from_secret_bytes(bytes: &[u8; 32]) -> Self {
        let signing = SigningKey::from_bytes(bytes);
        let verify = signing.verifying_key();
        Self { signing, verify }
    }

    /// Key rotation: generate a new keypair. Caller should persist via
    /// `crypto::keystore::persist_keys` and adopt the new keys in the router.
    pub fn rotate() -> Self {
        Self::generate()
    }
}