/// Builder pattern implementations for complex object construction.
/// 
/// Builders provide a fluent interface for constructing complex objects
/// with many optional parameters, improving readability and maintainability.

pub mod node_builder;
pub mod router_builder;

pub use node_builder::NodeBuilder;
pub use router_builder::RouterBuilder;
