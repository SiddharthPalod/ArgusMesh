/// Service layer providing high-level abstractions for core operations.
/// 
/// This module implements the Service Layer pattern, providing a clean interface
/// between the API layer and the domain logic. Services coordinate between
/// multiple repositories and domain objects.

pub mod alert_service;
pub mod mesh_service;
pub mod storage_service;

pub use alert_service::AlertService;
pub use mesh_service::MeshService;
pub use storage_service::StorageService;
