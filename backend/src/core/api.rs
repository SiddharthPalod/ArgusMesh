use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;
use crate::routing::envelope::{Envelope, Priority};
use crate::services::{AlertService, MeshService, StorageService};

#[frb]
#[derive(Serialize, Deserialize)]
pub struct AlertInput{
    pub sender: String,
    pub priority: Priority,
    pub payload: String,
}

#[frb]
#[derive(Serialize)]
pub struct NodeState{
    pub status: String,
}

/// JSON-serializable alert summary (Envelope contains SystemTime which doesn't serialize).
#[frb]
#[derive(Serialize)]
pub struct AlertSummary {
    pub msg_id: String,
    pub sender_id: String,
    pub priority: Priority,
    pub hop_count: u8,
    pub created_secs: u64,
}

impl From<&Envelope> for AlertSummary {
    fn from(env: &Envelope) -> Self {
        let created_secs = env
            .created_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            msg_id: env.msg_id.to_string(),
            sender_id: env.sender_id.clone(),
            priority: env.priority,
            hop_count: env.hop_count,
            created_secs,
        }
    }
}

/// Configure a platform-appropriate persistent storage path for the alert index.
///
/// On mobile/desktop, Flutter can call this early in startup with an
/// application-specific directory. The Rust core will then append the
/// `argus_alert_index.db` filename and use that path for sled.
#[frb]
pub fn configure_storage_base_dir(base_dir: String) -> Result<(), String> {
    StorageService::configure_base_dir(base_dir)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn init_node() -> Result<(), String> {
    let tag = crate::core::alert_index::get_persisted_runtime_tag()
        .unwrap_or_else(|| "argus-node".into());
    MeshService::initialize(tag)
        .map_err(|e| e.to_string())
}

/// Start the BLE-backed mesh node on this device (idempotent).
///
/// This wires together:
/// - BLE transport (scanner + adapter)
/// - in-memory store
/// - router
/// - node run loop
///
/// and spawns the async tasks onto the existing Tokio runtime managed by
/// flutter_rust_bridge.
#[frb]
pub async fn start_mesh_node(tag: String) -> Result<(), String> {
    // Initialise Android logcat integration on first call.
    crate::ensure_logging();
    log::info!("start_mesh_node called with tag={}", tag);

    // Use MeshService to start the node
    MeshService::start_mesh_node(tag)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn create_alert(input: AlertInput) -> Result<String, String> {
    AlertService::create_alert(
        input.sender,
        input.priority,
        input.payload.into_bytes(),
    )
    .map(|id| id.to_string())
    .map_err(|e| e.to_string())
}

#[frb]
pub fn get_known_alerts() -> Result<String, String> {
    let alerts: Vec<AlertSummary> = AlertService::list_alerts()
        .iter()
        .map(AlertSummary::from)
        .collect();
    serde_json::to_string(&alerts).map_err(|e| e.to_string())
}

#[frb]
pub fn get_node_state() -> Result<String, String> {
    let status = MeshService::get_status();
    let state = NodeState { status };
    serde_json::to_string(&state).map_err(|e| e.to_string())
}

#[frb]
pub async fn receive_ble_packet(data: Vec<u8>) -> Result<(), String> {
    crate::transport::ble::inject_packet(data).await
}

/// Get the next outbound packet queued by the Rust mesh core for transmission.
/// 
/// On Android, btleplug doesn't work, so Rust queues packets here and Flutter
/// polls this function and sends them via its BLE connections.
/// Returns None if no packets are queued.
#[frb]
pub fn get_next_outbound_packet() -> Result<Option<Vec<u8>>, String> {
    Ok(crate::transport::ble::try_get_outbound_packet())
}