use lru::LruCache;
use std:: num::NonZeroUsize;
use uuid::Uuid;

pub struct DedupCache {
    cache: LruCache<Uuid, ()>,
}

impl DedupCache{

    pub fn new(cap: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(cap).unwrap()),
        }
    }

    pub fn seen(&mut self, id: Uuid) -> bool {
        if self.cache.contains(&id) {
            return true;
        }
        else{
            self.cache.put(id, ());
            false
        }
    }
}