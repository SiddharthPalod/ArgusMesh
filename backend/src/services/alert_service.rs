/// Service for managing alerts in the mesh network.
///
/// Implements the Service Layer pattern, coordinating between:
/// - Alert repository (storage)
/// - Routing layer (propagation)
/// - Crypto layer (signing/encryption)
use crate::core::alert_index;
use crate::routing::envelope::{Envelope, Priority};
use crate::services::mesh_service;
use crate::error::MeshResult;

/// High-level service for alert operations.
pub struct AlertService;

impl AlertService {
    /// Creates a new alert and adds it to the system, and enqueues it for mesh
    /// propagation when the mesh runtime is running.
    pub fn create_alert(
        sender: String,
        priority: Priority,
        payload: Vec<u8>,
    ) -> MeshResult<uuid::Uuid> {
        let envelope = Envelope::new(sender, priority, payload);
        let msg_id = envelope.msg_id;

        // Persist locally so the alert survives restarts.
        alert_index::add_alert(envelope.clone());

        // If the mesh runtime is active, enqueue for transmission via the
        // global router handle. Any encryption/signing is handled by
        // Router::enqueue_local.
        if let Some(router_handle) = mesh_service::get_router_handle() {
            if let Ok(mut router) = router_handle.lock() {
                let _ = router.enqueue_local(envelope);
            }
        }

        Ok(msg_id)
    }

    /// Lists all known alerts.
    pub fn list_alerts() -> Vec<Envelope> {
        alert_index::list_alerts()
    }

    /// Clears all alerts (useful for testing).
    pub fn clear_alerts() {
        alert_index::clear_alerts();
    }

    /// Gets a specific alert by message ID.
    pub fn get_alert(msg_id: uuid::Uuid) -> Option<Envelope> {
        alert_index::list_alerts()
            .into_iter()
            .find(|env| env.msg_id == msg_id)
    }
}

