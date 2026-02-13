use std::sync::Arc;
use tokio::sync::mpsc;
use super::discovery::{BleScanner, PeerRegistry};
use super::adapter::BleAdapter;

/// Create a complete BLE transport stack with scanner and adapter
pub fn create_ble_transport() -> (Arc<BleScanner>, BleAdapter) {
    let (tx, rx) = mpsc::channel(100); // Buffer 100 notification packets (inbound)
    
    // Store a clone of tx for FFI injection (Flutter -> Rust)
    super::set_packet_injector(tx.clone());

    // On Android, also set up outbound queue (Rust -> Flutter)
    #[cfg(target_os = "android")]
    {
        let (outbound_tx, outbound_rx) = mpsc::channel(100); // Buffer 100 outbound packets
        super::set_outbound_queue(outbound_tx, outbound_rx);
    }

    let registry = Arc::new(PeerRegistry::new());
    let scanner = Arc::new(BleScanner::new(registry, tx));
    let adapter = BleAdapter::new(scanner.clone(), rx);
    
    (scanner, adapter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ble_transport() {
        let (scanner, _adapter) = create_ble_transport();
        assert_eq!(scanner.get_peripherals().len(), 0);
    }
}
