/// Repository implementation for alert storage.
/// 
/// Provides a clean abstraction over alert storage, currently using
/// the alert_index module but could be swapped for different implementations.

use crate::core::alert_index;
use crate::routing::envelope::Envelope;
use crate::repository::traits::Repository;
use crate::error::{MeshError, MeshResult};
use flutter_rust_bridge::frb;
use uuid::Uuid;

/// Repository for alert operations.
/// Marked as opaque since it's a unit struct (not supported by flutter_rust_bridge).
#[frb(opaque)]
pub struct AlertRepository;

impl AlertRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AlertRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository<Envelope, Uuid> for AlertRepository {
    fn save(&self, entity: &Envelope) -> MeshResult<()> {
        alert_index::add_alert(entity.clone());
        Ok(())
    }

    fn find_by_id(&self, id: &Uuid) -> MeshResult<Option<Envelope>> {
        let alerts = alert_index::list_alerts();
        Ok(alerts.into_iter().find(|env| env.msg_id == *id))
    }

    fn list_all(&self) -> MeshResult<Vec<Envelope>> {
        Ok(alert_index::list_alerts())
    }

    fn remove(&self, _id: &Uuid) -> MeshResult<()> {
        // Current implementation doesn't support removal
        // Could be enhanced to add removal to alert_index
        Err(MeshError::Storage("Alert removal not implemented".to_string()))
    }

    fn clear(&self) -> MeshResult<()> {
        alert_index::clear_alerts();
        Ok(())
    }
}
