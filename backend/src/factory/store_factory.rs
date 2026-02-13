/// Factory for creating store instances.
/// 
/// Encapsulates store creation logic and provides a clean interface
/// for creating different store types.

use crate::storage::Store;
use crate::storage::mem_store::MemStore;
use crate::error::MeshResult;

/// Store type enumeration.
#[derive(Debug, Clone, Copy)]
pub enum StoreType {
    Memory,
    // Future: Sled, Sqlite, etc.
}

/// Factory for creating store instances.
pub struct StoreFactory;

impl StoreFactory {
    /// Creates a store instance of the specified type.
    pub fn create(store_type: StoreType) -> MeshResult<Box<dyn Store>> {
        match store_type {
            StoreType::Memory => {
                Ok(Box::new(MemStore::new()) as Box<dyn Store>)
            }
        }
    }

    /// Creates the default store (in-memory).
    pub fn create_default() -> MeshResult<Box<dyn Store>> {
        Self::create(StoreType::Memory)
    }
}
