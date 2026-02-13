/// Factory for creating router instances.
/// 
/// Encapsulates router creation logic, including:
/// - Key generation
/// - Component initialization
/// - Dependency wiring

use crate::routing::router::Router;
use crate::transport::Transport;
use crate::storage::Store;
use crate::crypto::keys::NodeKeys;
use crate::crypto::encrypt::SymKey;
use rand::RngCore;
use std::sync::Arc;

/// Factory for creating router instances.
pub struct RouterFactory;

impl RouterFactory {
    /// Creates a new router with the given transport and store.
    pub fn create<T: Transport, S: Store>(
        transport: T,
        store: S,
    ) -> Router<T, S> {
        let keys = NodeKeys::generate();
        let mut sym_key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut sym_key);
        
        Router::new(Arc::new(transport), store, keys, sym_key)
    }

    /// Creates a router with default keys (for testing).
    pub fn create_with_keys<T: Transport, S: Store>(
        transport: T,
        store: S,
        keys: NodeKeys,
        sym_key: SymKey,
    ) -> Router<T, S> {
        Router::new(Arc::new(transport), store, keys, sym_key)
    }
}
