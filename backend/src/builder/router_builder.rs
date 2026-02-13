/// Builder for constructing Router instances.
/// 
/// Provides a fluent interface for configuring Router instances with
/// various options like dedup cache size, replay guard size, etc.

use crate::routing::router::Router;
use crate::transport::Transport;
use crate::storage::Store;
use crate::crypto::keys::NodeKeys;
use crate::crypto::encrypt::SymKey;
use std::sync::Arc;

/// Builder for creating Router instances.
pub struct RouterBuilder<T: Transport, S: Store> {
    transport: Option<T>,
    store: Option<S>,
    keys: Option<NodeKeys>,
    sym_key: Option<SymKey>,
    dedup_size: Option<usize>,
    replay_size: Option<usize>,
}

impl<T: Transport, S: Store> RouterBuilder<T, S> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            transport: None,
            store: None,
            keys: None,
            sym_key: None,
            dedup_size: None,
            replay_size: None,
        }
    }

    /// Sets the transport.
    pub fn with_transport(mut self, transport: T) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Sets the store.
    pub fn with_store(mut self, store: S) -> Self {
        self.store = Some(store);
        self
    }

    /// Sets the node keys.
    pub fn with_keys(mut self, keys: NodeKeys) -> Self {
        self.keys = Some(keys);
        self
    }

    /// Sets the symmetric key.
    pub fn with_sym_key(mut self, sym_key: SymKey) -> Self {
        self.sym_key = Some(sym_key);
        self
    }

    /// Sets the deduplication cache size.
    pub fn with_dedup_size(mut self, size: usize) -> Self {
        self.dedup_size = Some(size);
        self
    }

    /// Sets the replay guard size.
    pub fn with_replay_size(mut self, size: usize) -> Self {
        self.replay_size = Some(size);
        self
    }

    /// Builds the Router instance.
    pub fn build(self) -> Result<Router<T, S>, String> {
        use rand::RngCore;
        
        let transport = self.transport.ok_or("Transport is required")?;
        let store = self.store.ok_or("Store is required")?;
        let keys = self.keys.unwrap_or_else(NodeKeys::generate);
        let sym_key = self.sym_key.unwrap_or_else(|| {
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            key
        });
        
        // Note: Router::new doesn't currently accept these parameters,
        // but this builder pattern allows for future extensibility
        let mut router = Router::new(Arc::new(transport), store, keys, sym_key);
        
        // Future: Apply dedup_size and replay_size if Router supports it
        
        Ok(router)
    }
}

impl<T: Transport, S: Store> Default for RouterBuilder<T, S> {
    fn default() -> Self {
        Self::new()
    }
}
