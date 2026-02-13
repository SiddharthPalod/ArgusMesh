/// Factory for creating transport instances.
/// 
/// Encapsulates transport creation logic and provides a clean interface
/// for creating different transport types.

use crate::transport::Transport;
use crate::transport::ble::builder::create_ble_transport;
use crate::transport::test_sim::{SimTransport, SimConfig};
use crate::error::MeshResult;
use tokio::sync::mpsc;

/// Transport type enumeration.
#[derive(Debug, Clone, Copy)]
pub enum TransportType {
    Ble,
    TestSim,
    // Future: WifiDirect, Udp, etc.
}

/// Factory for creating transport instances.
pub struct TransportFactory;

impl TransportFactory {
    /// Creates a transport instance of the specified type.
    pub async fn create(
        transport_type: TransportType,
    ) -> MeshResult<Box<dyn Transport>> {
        match transport_type {
            TransportType::Ble => {
                let (_scanner, adapter) = create_ble_transport();
                Ok(Box::new(adapter) as Box<dyn Transport>)
            }
            TransportType::TestSim => {
                let (tx, rx) = mpsc::channel(100);
                let config = SimConfig {
                    latency_ms: 10,
                    drop_rate: 0.0,
                };
                Ok(Box::new(SimTransport::new(tx, rx, config)) as Box<dyn Transport>)
            }
        }
    }

    /// Creates the default transport (BLE).
    pub async fn create_default() -> MeshResult<Box<dyn Transport>> {
        Self::create(TransportType::Ble).await
    }
}
