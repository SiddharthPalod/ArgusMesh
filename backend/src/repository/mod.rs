/// Repository pattern implementation for data access.
/// 
/// Repositories provide a clean abstraction over storage implementations,
/// allowing easy swapping between in-memory, file-based, or database storage.

pub mod alert_repository;
pub mod traits;

pub use alert_repository::AlertRepository;
pub use traits::Repository;
