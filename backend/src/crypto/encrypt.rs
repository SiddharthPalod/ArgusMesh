use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};

use rand::RngCore;

pub type SymKey = [u8; 32];
pub type NonceBytes = [u8; 12];

pub fn encrypt(key: &SymKey, plaintext: &[u8]) -> Result<(Vec<u8>, NonceBytes), ()>{
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));

    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);

    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|_| ())?;
    Ok((ciphertext, nonce))
}

pub fn decrypt(key: &SymKey, nonce: &NonceBytes, ciphertext: &[u8]) -> Result<Vec<u8>, ()>{
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| ())
}