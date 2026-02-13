use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use futures::StreamExt;

pub struct PeerRegistry {
    peers: Arc<Mutex<HashMap<String, Peripheral>>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self{
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_peer(&self, id: String, peripheral: Peripheral) {
        self.peers.lock().unwrap().insert(id, peripheral);
    }

    pub fn remove_peer(&self, id: &str) {
        self.peers.lock().unwrap().remove(id);
    }

    pub fn list(&self) -> Vec<String> {
        self.peers.lock().unwrap().keys().cloned().collect()
    }

    pub fn get_peripherals(&self) -> Vec<Peripheral> {
        self.peers.lock().unwrap().values().cloned().collect()
    }
}

use tokio::sync::mpsc;

pub struct BleScanner {
    registry: Arc<PeerRegistry>,
    notification_tx: mpsc::Sender<Vec<u8>>,
}

impl BleScanner {
    pub fn new(registry: Arc<PeerRegistry>, notification_tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self { registry, notification_tx }
    }

    pub fn get_peripherals(&self) -> Vec<btleplug::platform::Peripheral> {
        self.registry.get_peripherals()
    }

    pub async fn start_scan(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("BleScanner start_scan invoked.");
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        
        log::info!("Found {} Bluetooth adapters.", adapters.len());
        if adapters.is_empty() {
             return Err("No Bluetooth adapters found".into());
        }

        let central = adapters.into_iter().nth(0).unwrap();

        // Start scanning
        use super::constants::SERVICE_UUID;
        log::info!("Starting scan for Service UUID: {:?}", SERVICE_UUID);
        
        let filter = ScanFilter {
            services: vec![SERVICE_UUID],
            ..Default::default()
        };
        central.start_scan(filter).await?;
        log::info!("Scan started successfully.");

        let registry = self.registry.clone();
        let notif_tx = self.notification_tx.clone();
        
        tokio::spawn(async move {
            let mut events = central.events().await.unwrap();
            
            while let Some(event) = events.next().await {
                match event {
                    btleplug::api::CentralEvent::DeviceDiscovered(id) => {
                        log::info!("Discovered device: {:?}", id);
                       if let Ok(peripheral) = central.peripheral(&id).await {
                           if let Ok(Some(props)) = peripheral.properties().await {
                               if let Some(local_name) = props.local_name {
                                   log::info!("Device {:?} has name: {}", id, local_name);
                                   if local_name.contains("ArgusMesh") {
                                       log::info!("MATCH! Connecting to ArgusMesh device: {:?}", id);
                                       registry.add_peer(id.to_string(), peripheral.clone());
                                       
                                       // Auto-connect and discover services
                                       let peer = peripheral.clone();
                                       let tx = notif_tx.clone();
                                       tokio::spawn(async move {
                                            use super::constants::CHARACTERISTIC_INBOX_UUID;
                                            use btleplug::api::Peripheral;
                                            
                                            log::info!("Connecting to peer {:?}", peer.id());
                                            if let Err(e) = peer.connect().await {
                                                log::error!("Failed to connect to {:?}: {:?}", peer.id(), e);
                                                return;
                                            }
                                            log::info!("Connected to peer {:?}", peer.id());
                                            
                                            // Discover services
                                            if let Err(e) = peer.discover_services().await {
                                                log::error!("Failed to discover services for {:?}: {:?}", peer.id(), e);
                                                return;
                                            }
                                            log::info!("Services discovered for {:?}", peer.id());
                                            
                                            // Find and subscribe to INBOX characteristic
                                            let services = peer.services();
                                            for service in services {
                                                for char in service.characteristics {
                                                    if char.uuid == CHARACTERISTIC_INBOX_UUID {
                                                        log::info!("Found INBOX UUID on peer {:?}", peer.id());
                                                        if let Err(e) = peer.subscribe(&char).await {
                                                            log::error!("Failed to subscribe to characteristic: {:?}", e);
                                                            return;
                                                        }
                                                        log::info!("Subscribed to INBOX on peer {:?}", peer.id());
                                                        
                                                        // Listen for notifications
                                                        let mut notification_stream = peer.notifications().await.unwrap();
                                                        while let Some(notif) = notification_stream.next().await {
                                                            if notif.uuid == CHARACTERISTIC_INBOX_UUID {
                                                                log::info!("Received notification from {:?}", peer.id());
                                                                let _ = tx.send(notif.value).await;
                                                            }
                                                        }
                                                        return;
                                                    }
                                                }
                                            }
                                       });
                                   }
                               }
                           }
                       }
                    }
                    btleplug::api::CentralEvent::DeviceDisconnected(id) => {
                        log::info!("Device Disconnected: {:?}", id);
                        registry.remove_peer(&id.to_string());
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}

pub trait BleDiscovery {
    fn advertise(&self) -> impl std::future::Future<Output = ()> + Send;
    fn scan(&self) -> impl std::future::Future<Output = ()> + Send;
}

impl BleDiscovery for BleScanner {
    async fn advertise(&self) {
        // Advertising logic to be implemented later (Rust btleplug primarily supports Central currently)
    }

    async fn scan(&self) {
        if let Err(e) = self.start_scan().await {
            eprintln!("BLE Scan failed: {:?}", e);
        }
    }
}