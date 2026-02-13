/// Builder for constructing Node instances.
/// 
/// Provides a fluent interface for configuring and creating Node instances
/// with various options.

use crate::core::node::Node;
use crate::routing::router::Router;
use crate::transport::Transport;
use crate::storage::Store;
use std::sync::Arc;
use std::time::Duration;

/// Builder for creating Node instances.
pub struct NodeBuilder<T: Transport + 'static, S: Store + 'static> {
    router: Option<Arc<std::sync::Mutex<Router<T, S>>>>,
    transport: Option<Arc<T>>,
    tick_interval: Option<Duration>,
}

impl<T: Transport + 'static, S: Store + 'static> NodeBuilder<T, S> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            router: None,
            transport: None,
            tick_interval: None,
        }
    }

    /// Sets the router for the node.
    pub fn with_router(mut self, router: Arc<std::sync::Mutex<Router<T, S>>>) -> Self {
        self.router = Some(router);
        self
    }

    /// Sets the transport for the node.
    pub fn with_transport(mut self, transport: Arc<T>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Sets the tick interval (currently not used but reserved for future use).
    pub fn with_tick_interval(mut self, interval: Duration) -> Self {
        self.tick_interval = Some(interval);
        self
    }

    /// Builds the Node instance.
    pub fn build(self) -> Result<Node<T, S>, String> {
        let router = self.router.ok_or("Router is required")?;
        let transport = self.transport.ok_or("Transport is required")?;
        
        Ok(Node::new(router, transport))
    }
}

impl<T: Transport + 'static, S: Store + 'static> Default for NodeBuilder<T, S> {
    fn default() -> Self {
        Self::new()
    }
}
