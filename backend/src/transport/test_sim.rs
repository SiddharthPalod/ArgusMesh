use tokio::sync::mpsc;
use crate::transport::Transport;
use crate::transport::error::TransportError;
use rand::Rng;
use std::time::Duration;
use async_trait::async_trait;

pub struct SimConfig {
    pub latency_ms: u64,
    pub drop_rate: f32,
}

pub struct SimTransport {
    tx: mpsc::Sender<Vec<u8>>,
    #[allow(dead_code)]
    rx: tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>,
    cfg: SimConfig,
}

impl SimTransport {
    pub fn new(tx: mpsc::Sender<Vec<u8>>, rx: mpsc::Receiver<Vec<u8>>, cfg: SimConfig) -> Self {
        Self { tx, rx: tokio::sync::Mutex::new(rx), cfg }
    }
}

#[async_trait]
impl Transport for SimTransport {
    async fn start(&self) -> Result<(), TransportError> {
        Ok(())
    }

    async fn send(&self, data: Vec<u8>) -> Result<(), TransportError> {
        // Drop logic
        if rand::thread_rng().r#gen::<f32>() < self.cfg.drop_rate {
            return Ok(());
        }

        // Latency logic
        tokio::time::sleep(Duration::from_millis(self.cfg.latency_ms)).await;

        self.tx.send(data).await.map_err(|_| TransportError::Internal("Channel closed".into()))?;
        Ok(())
    }

    async fn recv(&self) -> Result<Vec<u8>, TransportError> {
        let mut rx = self.rx.lock().await;
        rx.recv().await.ok_or(TransportError::Internal("Channel closed".into()))
    }

    fn is_connected(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "sim_transport"
    }
}
