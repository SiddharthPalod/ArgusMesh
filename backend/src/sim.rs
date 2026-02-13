//! Multi-node mesh simulation: propagation goes through router.receive()
//! (verify → replay → decrypt → dedup) so sim matches real network behavior.

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::routing::envelope::{Envelope, Priority};
use crate::routing::router::Router;
use crate::storage::mem_store::MemStore;
use crate::transport::{Transport, error::TransportError};
use crate::crypto::keys::NodeKeys;
use crate::crypto::encrypt::SymKey;
use crate::core::api::AlertSummary;
use rand::RngCore;

// ---- Dummy transport for sim: we never call tick/recv; only next_outbound + receive ----

struct SimDummyTransport {
    _tx: mpsc::Sender<Vec<u8>>,
    rx: tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>,
}

impl SimDummyTransport {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(64);
        Self {
            _tx: tx,
            rx: tokio::sync::Mutex::new(rx),
        }
    }
}

#[async_trait]
impl Transport for SimDummyTransport {
    async fn start(&self) -> Result<(), TransportError> {
        Ok(())
    }
    async fn send(&self, _data: Vec<u8>) -> Result<(), TransportError> {
        Ok(())
    }
    async fn recv(&self) -> Result<Vec<u8>, TransportError> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| TransportError::Internal("sim transport closed".into()))
    }
    fn is_connected(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        "sim_dummy"
    }
}

// ---- Sim node: router + inbox + per-node alert list (filled by receive path) ----

struct SimNode {
    router: Mutex<Router<SimDummyTransport, MemStore>>,
    inbox: Mutex<Vec<Vec<u8>>>,
    alerts: Arc<Mutex<Vec<Envelope>>>,
}

// ---- Mesh state ----

static MESH: Lazy<Mutex<Option<SimMesh>>> = Lazy::new(|| Mutex::new(None));

struct SimMesh {
    nodes: Vec<SimNode>,
}

/// Start a simulation with n nodes. Each node has its own Router; propagation uses receive().
pub fn sim_start(n: usize) -> Result<(), String> {
    if n == 0 || n > 64 {
        return Err("n must be 1..64".into());
    }
    let mut sym_key: SymKey = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut sym_key);

    let mut nodes = Vec::with_capacity(n);
    for _ in 0..n {
        let alerts = Arc::new(Mutex::new(Vec::new()));
        let transport = SimDummyTransport::new();
        let store = MemStore::new();
        let keys = NodeKeys::generate();
        let mut router = Router::new(std::sync::Arc::new(transport), store, keys, sym_key);
        let alerts_clone = Arc::clone(&alerts);
        router.set_receive_sink(Box::new(move |env| {
            let _ = alerts_clone.lock().map(|mut v| v.push(env));
        }));
        nodes.push(SimNode {
            router: Mutex::new(router),
            inbox: Mutex::new(Vec::new()),
            alerts,
        });
    }
    *MESH.lock().map_err(|e| e.to_string())? = Some(SimMesh { nodes });
    Ok(())
}

/// Create an alert on the given node via router.enqueue_local (encrypt + sign).
pub fn sim_create_alert(
    node_id: usize,
    sender: String,
    priority: Priority,
    payload: String,
) -> Result<String, String> {
    let env = Envelope::new(sender, priority, payload.into_bytes());
    let id = env.msg_id.to_string();

    let mut guard = MESH.lock().map_err(|e| e.to_string())?;
    let mesh = guard.as_mut().ok_or("sim not started")?;
    if node_id >= mesh.nodes.len() {
        return Err("invalid node_id".into());
    }
    let mut router = mesh.nodes[node_id].router.lock().map_err(|e| e.to_string())?;
    router.enqueue_local(env)?;
    Ok(id)
}

/// One propagation step: serialize from each router (next_outbound), deliver to neighbor inboxes, then receive() so each node runs verify → replay → decrypt → dedup.
pub fn sim_propagate() -> Result<(), String> {
    let mut guard = MESH.lock().map_err(|e| e.to_string())?;
    let mesh = guard.as_mut().ok_or("sim not started")?;
    let n = mesh.nodes.len();
    if n == 0 {
        return Ok(());
    }

    // 1. Each node: pop outbound, push to prev/next inbox
    for i in 0..n {
        let mut router = mesh.nodes[i].router.lock().map_err(|e| e.to_string())?;
        while let Some(bytes) = router.next_outbound() {
            let prev = (i + n - 1) % n;
            let next = (i + 1) % n;
            mesh.nodes[prev].inbox.lock().map_err(|e| e.to_string())?.push(bytes.clone());
            if next != prev {
                mesh.nodes[next].inbox.lock().map_err(|e| e.to_string())?.push(bytes);
            }
        }
    }

    // 2. Each node: drain inbox and run receive() (verify → replay → decrypt → dedup)
    for i in 0..n {
        let to_deliver: Vec<Vec<u8>> = {
            let mut inbox = mesh.nodes[i].inbox.lock().map_err(|e| e.to_string())?;
            std::mem::take(&mut *inbox)
        };
        let mut router = mesh.nodes[i].router.lock().map_err(|e| e.to_string())?;
        for bytes in to_deliver {
            router.receive(&bytes);
        }
    }
    Ok(())
}

/// Get alerts for a node (filled by receive path via sink).
pub fn sim_get_alerts(node_id: usize) -> Result<String, String> {
    let guard = MESH.lock().map_err(|e| e.to_string())?;
    let mesh = guard.as_ref().ok_or("sim not started")?;
    if node_id >= mesh.nodes.len() {
        return Err("invalid node_id".into());
    }
    let alerts = mesh.nodes[node_id]
        .alerts
        .lock()
        .map_err(|e| e.to_string())?;
    let summaries: Vec<AlertSummary> = alerts.iter().map(AlertSummary::from).collect();
    serde_json::to_string(&summaries).map_err(|e| e.to_string())
}

pub fn sim_node_count() -> usize {
    MESH
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|m| m.nodes.len()))
        .unwrap_or(0)
}

pub fn sim_stop() {
    let _ = MESH.lock().map(|mut g| *g = None);
}
