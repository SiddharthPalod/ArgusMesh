/// Factory pattern implementations for creating complex objects.
/// 
/// Factories encapsulate object creation logic, making it easier to:
/// - Create objects with complex initialization
/// - Swap implementations
/// - Provide default configurations
/// - Centralize creation logic

pub mod transport_factory;
pub mod store_factory;
pub mod router_factory;

pub use transport_factory::TransportFactory;
pub use store_factory::StoreFactory;
pub use router_factory::RouterFactory;
