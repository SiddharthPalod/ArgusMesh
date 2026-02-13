use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};

pub fn sign_message(sk: &SigningKey, msg: &[u8]) -> Signature{
    sk.sign(msg)
}

pub fn verify_message(pk: &VerifyingKey, msg: &[u8], sig: &Signature) -> bool{
    pk.verify(msg, sig).is_ok()
}

pub fn signature_to_bytes(sig: &Signature) -> [u8; 64]{
    sig.to_bytes()
}

pub fn signature_from_bytes(b: &[u8; 64]) -> Signature{
    Signature::from_bytes(b)
}