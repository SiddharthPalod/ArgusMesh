/// Trait defining the repository pattern interface.
/// 
/// Repositories abstract data access operations, making it easy to:
/// - Swap storage implementations
/// - Test with mock repositories
/// - Maintain consistent data access patterns

use crate::error::MeshResult;

/// Generic repository trait for CRUD operations.
pub trait Repository<T, ID> {
    /// Saves an entity.
    fn save(&self, entity: &T) -> MeshResult<()>;

    /// Finds an entity by ID.
    fn find_by_id(&self, id: &ID) -> MeshResult<Option<T>>;

    /// Lists all entities.
    fn list_all(&self) -> MeshResult<Vec<T>>;

    /// Removes an entity by ID.
    fn remove(&self, id: &ID) -> MeshResult<()>;

    /// Clears all entities.
    fn clear(&self) -> MeshResult<()>;
}
