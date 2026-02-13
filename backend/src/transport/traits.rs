use async_trait::async_trait;
use super::error::TransportError;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn start(&self) -> Result<(), TransportError>;

    async fn send(&self, data: Vec<u8>) -> Result<(), TransportError>;

    async fn recv(&self) -> Result<Vec<u8>, TransportError>;

    fn is_connected(&self) -> bool;

    fn name(&self) -> &'static str;
}
