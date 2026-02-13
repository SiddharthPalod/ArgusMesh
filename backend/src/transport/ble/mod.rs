pub mod discovery;
pub mod fragment;
pub mod reassembly;
pub mod adapter;
pub mod constants;
pub mod builder;
pub mod messaging;
pub mod connection;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::mpsc;

// Inbound packet injector (Flutter → Rust)
static INJECTOR: Lazy<Mutex<Option<mpsc::Sender<Vec<u8>>>>> = Lazy::new(|| Mutex::new(None));

pub fn set_packet_injector(tx: mpsc::Sender<Vec<u8>>) {
    if let Ok(mut guard) = INJECTOR.lock() {
        *guard = Some(tx);
    }
}

pub async fn inject_packet(data: Vec<u8>) -> Result<(), String> {
    let tx = {
        let guard = INJECTOR.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    
    if let Some(sender) = tx {
        sender.send(data).await.map_err(|e| e.to_string())
    } else {
        Err("Packet injector not initialized".into())
    }
}

// Outbound packet queue (Rust → Flutter) for Android compatibility
// On Android, btleplug doesn't work, so Rust queues packets here and Flutter polls them.
static OUTBOUND_QUEUE_TX: Lazy<Mutex<Option<mpsc::Sender<Vec<u8>>>>> = Lazy::new(|| Mutex::new(None));
static OUTBOUND_QUEUE_RX: Lazy<Mutex<Option<mpsc::Receiver<Vec<u8>>>>> = Lazy::new(|| Mutex::new(None));

pub fn set_outbound_queue(tx: mpsc::Sender<Vec<u8>>, rx: mpsc::Receiver<Vec<u8>>) {
    if let Ok(mut guard_tx) = OUTBOUND_QUEUE_TX.lock() {
        *guard_tx = Some(tx);
    }
    if let Ok(mut guard_rx) = OUTBOUND_QUEUE_RX.lock() {
        *guard_rx = Some(rx);
    }
}

pub fn queue_outbound_packet(data: Vec<u8>) -> Result<(), String> {
    let tx = {
        let guard = OUTBOUND_QUEUE_TX.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    
    if let Some(sender) = tx {
        // Use try_send to avoid blocking; if queue is full, log and continue
        match sender.try_send(data) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                log::warn!("Outbound packet queue full, dropping packet");
                Ok(()) // Don't fail, just drop
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err("Outbound queue closed".into())
            }
        }
    } else {
        // On non-Android platforms, this might not be initialized (btleplug handles it)
        // So we silently succeed
        Ok(())
    }
}

pub fn try_get_outbound_packet() -> Option<Vec<u8>> {
    let mut rx_guard = OUTBOUND_QUEUE_RX.lock().ok()?;
    let rx = rx_guard.as_mut()?;
    rx.try_recv().ok()
}
