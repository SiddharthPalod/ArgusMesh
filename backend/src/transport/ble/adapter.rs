use async_trait::async_trait;
use crate::transport::{Transport, error::TransportError};
use super::discovery::BleScanner;
use super::reassembly::Reassembler;
use super::messaging::{AckTracker, is_ack_packet, decode_ack};
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

pub struct BleAdapter {
    scanner: Arc<BleScanner>,
    reassembler: Arc<Mutex<Reassembler>>,
    ack_tracker: Arc<Mutex<AckTracker>>,
    notification_rx: tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>,
}

impl BleAdapter {
    pub fn new(scanner: Arc<BleScanner>, notification_rx: mpsc::Receiver<Vec<u8>>) -> Self {
        Self { 
            scanner,
            reassembler: Arc::new(Mutex::new(Reassembler::new())),
            ack_tracker: Arc::new(Mutex::new(AckTracker::new(30))),
            notification_rx: tokio::sync::Mutex::new(notification_rx),
        }
    }

    /// Get acknowledgement stats: (pending, acked).
    pub fn ack_stats(&self) -> (usize, usize) {
        self.ack_tracker.lock().unwrap().stats()
    }
}

use uuid::Uuid;
use super::fragment;

#[async_trait]
impl Transport for BleAdapter {
    async fn start(&self) -> Result<(), TransportError> {
        self.ack_tracker.lock().unwrap().prune_expired();
        
        // On Android, btleplug doesn't work - Flutter handles BLE scanning/connections.
        // So we skip the btleplug scan here. The scanner.start_scan() would fail anyway.
        // On other platforms (Linux/macOS/Windows), btleplug works and we can scan.
        #[cfg(target_os = "android")]
        {
            log::info!("BleAdapter::start() skipped on Android (Flutter handles BLE)");
            return Ok(());
        }
        
        #[cfg(not(target_os = "android"))]
        {
            self.scanner.start_scan().await.map_err(|e| TransportError::Internal(e.to_string()))
        }
    }

    async fn send(&self, data: Vec<u8>) -> Result<(), TransportError> {
        // On Android, queue packets for Flutter to send via its BLE connections.
        // On other platforms, use btleplug directly.
        #[cfg(target_os = "android")]
        {
            // Fragment the packet with UUID headers (same format as non-Android path)
            // so the receiving side's reassembler can handle it correctly.
            let msg_id = Uuid::new_v4();
            let fragments = fragment::fragment(msg_id, &data);
            
            // Queue each fragment separately - Flutter will send them one by one
            // and the receiving side will reassemble them.
            for frag in fragments {
                if let Err(e) = super::queue_outbound_packet(frag) {
                    log::warn!("Failed to queue fragment: {}", e);
                    // Continue with other fragments even if one fails
                }
            }
            Ok(())
        }
        
        #[cfg(not(target_os = "android"))]
        {
            let peers = self.scanner.get_peripherals();
            if peers.is_empty() {
                 // In a broadcast mesh, no peers might just mean no one hears it, 
                 // but let's log or debug print.
                 return Ok(());
            }

            let msg_id = Uuid::new_v4();

            // Register with ack tracker for delivery tracking
            if let Ok(mut tracker) = self.ack_tracker.lock() {
                tracker.register(msg_id);
            }

            let fragments = fragment::fragment(msg_id, &data);

            use btleplug::api::{Peripheral, WriteType};
            use super::constants::CHARACTERISTIC_INBOX_UUID;

            for peer in peers {
                 // Find characteristic
                 // Optimization: Cache characteristics in PeerRegistry or similar
                 let mut char_found = None;
                 if let Ok(Some(_props)) = peer.properties().await {
                     // Services should be discovered
                     let services = peer.services();
                     for service in services {
                         for char in service.characteristics {
                             if char.uuid == CHARACTERISTIC_INBOX_UUID {
                                 char_found = Some(char);
                                 break;
                             }
                         }
                         if char_found.is_some() { break; }
                     }
                 }

                 if let Some(c) = char_found {
                     for frag in &fragments {
                         if let Err(e) = peer.write(&c, frag, WriteType::WithoutResponse).await {
                             eprintln!("Failed to write frag to peer: {:?}", e);
                         }
                     }
                 }
            }
            Ok(())
        }
    }

    async fn recv(&self) -> Result<Vec<u8>, TransportError> {
        loop {
            let mut rx = self.notification_rx.lock().await;
            let packet = rx.recv().await.ok_or(TransportError::Internal("Notification channel closed".into()))?;
            drop(rx); // Release lock

            // Check if this is an ack packet (exactly 17 bytes)
            if is_ack_packet(&packet) {
                if let Some(ack) = decode_ack(&packet) {
                    if let Ok(mut tracker) = self.ack_tracker.lock() {
                        tracker.receive_ack(&ack);
                    }
                }
                // Don't return ack packets as messages; continue to next packet
                continue;
            }
            
            // Try to reassemble
            let mut reassembler = self.reassembler.lock().unwrap();
            if let Some(full_msg) = reassembler.push(packet) {
                return Ok(full_msg);
            }
            // If None, fragment was stored but not complete yet; continue loop
        }
    }

    fn is_connected(&self) -> bool {
        true 
    }

    fn name(&self) -> &'static str {
        "ble_adapter"
    }
}
