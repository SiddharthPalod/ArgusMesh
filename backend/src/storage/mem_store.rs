use std::collections::HashMap;
use std::sync::Mutex;

use super::Store;

/// In-memory store for simulation (no persistence).
pub struct MemStore {
    inner: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MemStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for MemStore {
    fn persist(&self, key: Vec<u8>, val: Vec<u8>) {
        let _ = self.inner.lock().map(|mut m| m.insert(key, val));
    }

    fn load_all(&self) -> Vec<Vec<u8>> {
        self.inner
            .lock()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    fn remove(&self, key: Vec<u8>) {
        let _ = self.inner.lock().map(|mut m| m.remove(&key));
    }
}
