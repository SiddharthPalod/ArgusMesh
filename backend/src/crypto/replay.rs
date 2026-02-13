use lru::LruCache;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;

pub struct ReplayGuard{
    seen: LruCache<[u8; 32], ()>,
}

impl ReplayGuard{
    pub fn new(cap: usize) -> Self{
        Self{
            seen: LruCache::new(NonZeroUsize::new(cap).unwrap()),
        }
    }

    pub fn digest(data: &[u8]) -> [u8; 32]{
        let mut h = Sha256::new();
        h.update(data);
        h.finalize().into()
    }

    pub fn check_and_insert(&mut self, data: &[u8]) -> bool{
        let id = Self::digest(data);
        if self.seen.contains(&id) {
            false
        } else{
            self.seen.put(id, ());
            true
        }
    }
}