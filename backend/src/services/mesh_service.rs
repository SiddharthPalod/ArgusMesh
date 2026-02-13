/// Service for managing mesh node lifecycle and operations.
///
/// Coordinates between:
/// - Runtime state management
/// - Transport layer
/// - Storage layer
/// - Routing layer
use crate::core::runtime;
use crate::core::node::Node;
use crate::crypto::encrypt::SymKey;
use crate::crypto::keys::NodeKeys;
use crate::error::{MeshError, MeshResult};
use crate::routing::router::Router;
use crate::storage::mem_store::MemStore;
use crate::transport::ble::adapter::BleAdapter;
use crate::transport::ble::builder::create_ble_transport;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use tokio::task;

/// Concrete router type used by the on-device BLE mesh runtime.
type MeshRouter = Router<BleAdapter, MemStore>;

/// Shared symmetric key used by all nodes on the mesh.
///
/// NOTE: This is intentionally a fixed key for the current prototype so that all
/// devices can decrypt each other's messages. In a production system this
/// should be replaced by a proper key management / distribution mechanism.
const SHARED_SYM_KEY: SymKey = [0x42; 32];

/// Global handle to the running router so that other services (e.g. alerts)
/// can enqueue messages for transmission.
static ROUTER_HANDLE: Lazy<Mutex<Option<Arc<Mutex<MeshRouter>>>>> =
    Lazy::new(|| Mutex::new(None));

fn set_router_handle(router: Arc<Mutex<MeshRouter>>) {
    if let Ok(mut guard) = ROUTER_HANDLE.lock() {
        *guard = Some(router);
    }
}

/// Get a clone of the global router handle, if the mesh runtime is running.
pub fn get_router_handle() -> Option<Arc<Mutex<MeshRouter>>> {
    ROUTER_HANDLE
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

/// High-level service for mesh node operations.
pub struct MeshService;

impl MeshService {
    /// Initializes the mesh node with a given tag.
    pub fn initialize(tag: String) -> MeshResult<()> {
        runtime::set_runtime_tag(tag.clone());
        crate::core::alert_index::set_persisted_runtime_tag(&tag);
        
        // Load persisted alerts from DB into memory cache
        crate::core::alert_index::load_alerts_from_db();
        let alert_count = crate::core::alert_index::list_alerts().len();
        log::info!("Loaded {} persisted alerts on startup", alert_count);
        
        Ok(())
    }

    /// Starts the BLE-backed mesh node runtime.
    ///
    /// This is idempotent – if the runtime is already running, this is a no-op.
    /// The actual async tasks (BLE transport + routing loop) are spawned onto
    /// the Tokio runtime and run in the background.
    pub fn start_mesh_node(tag: String) -> MeshResult<()> {
        if runtime::has_mesh_runtime() {
            return Ok(());
        }

        // Record runtime state for diagnostics and UI.
        runtime::set_runtime_tag(tag);
        runtime::mark_mesh_started();

        // Spawn the mesh runtime in the background so this call remains fast
        // and fits the sync FRB API surface.
        task::spawn(async move {
            if let Err(e) = Self::spawn_runtime().await {
                log::error!("Mesh runtime failed to start: {:?}", e);
            }
        });

        Ok(())
    }

    /// Internal helper that wires together:
    /// - BLE transport (scanner + adapter)
    /// - in-memory store
    /// - router
    /// - node run loop
    async fn spawn_runtime() -> MeshResult<()> {
        // Create BLE transport stack and packet injector (for Flutter → Rust).
        let (_scanner, adapter) = create_ble_transport();
        let adapter_arc = Arc::new(adapter);

        // In-memory store for opportunistic forwarding; can be swapped later.
        let store = MemStore::new();

        // Per-node keypair, shared symmetric key for the current prototype.
        let keys = NodeKeys::generate();
        let sym_key: SymKey = SHARED_SYM_KEY;

        // Build router - Router now stores Arc<T> so we can share it with Node.
        let router = Router::new(adapter_arc.clone(), store, keys, sym_key);
        let router_arc = Arc::new(Mutex::new(router));
        set_router_handle(router_arc.clone());

        // Build node wrapper and start BLE + routing loop.
        let node = Node::new(router_arc, adapter_arc.clone());

        // Start BLE scanning / connections first.
        use crate::transport::Transport;
        if let Err(e) = adapter_arc.as_ref().start().await {
            return Err(MeshError::Transport(format!(
                "Failed to start BLE transport: {:?}",
                e
            )));
        }

        // Run the node loop forever; errors are logged from inside.
        node.run().await;
        Ok(())
    }

    /// Gets the current runtime tag/status.
    pub fn get_status() -> String {
        runtime::get_runtime_tag()
            .unwrap_or_else(|| "not-initialized".to_string())
    }

    /// Checks if the mesh runtime is active.
    pub fn is_running() -> bool {
        runtime::has_mesh_runtime()
    }
}

