use serde::{Serialize, Deserialize};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::core::types::{NodeId, AlertType, Severity};

pub const DEFAULT_TTL: u8 = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_id: Uuid,
    pub origin_node: NodeId,
    pub alert_type: AlertType,
    pub severity: Severity,
    pub lattitude: f64,
    pub longitude: f64,
    pub created_at: OffsetDateTime,
    pub ttl: u8,
}

impl Alert {
    pub fn new(origin_node: NodeId, alert_type: AlertType, severity: Severity, lattitude: f64, longitude: f64) -> Result<Self, AlertError> {
        
        validate_coords(lattitude, longitude)?;

        Ok(Self {
            alert_id: Uuid::new_v4(),
            origin_node,
            alert_type,
            severity,
            lattitude,
            longitude,
            created_at: OffsetDateTime::now_utc(),
            ttl: DEFAULT_TTL,
        })
    }

    pub fn decrement_ttl(&mut self) {
        if self.ttl > 0 {
            self.ttl -= 1;
        }
    }

    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AlertError {
    #[error("Invalid coordinates: latitude must be between -90 and 90, longitude must be between -180 and 180")]
    InvalidCoordinates,
}

fn validate_coords(lattitude: f64, longitude: f64) -> Result<(), AlertError> {
    if lattitude.abs() > 90.0 || longitude.abs() > 180.0 {
        Err(AlertError::InvalidCoordinates)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::*;

    #[test]
    fn alert_creation_ok(){
        let node = NodeId::generate_ephemeral();
        let alert = Alert::new(node, AlertType::Intrusion, Severity::High, 12.0, 77.0).unwrap();

        assert_eq!(alert.ttl, DEFAULT_TTL);
    }

    #[test]
    fn invalid_coords_fail(){
        let node = NodeId::generate_ephemeral();
        let alert = Alert::new(node, AlertType::Drone, Severity::Low, 999.0, 0.0);
        assert!(alert.is_err());
    }

    #[test]
    fn ttl_decrements(){
        let node = NodeId::generate_ephemeral();
        let mut alert = Alert::new(node, AlertType::Movement, Severity::Medium, 10.0, 10.0).unwrap();
        alert.decrement_ttl();
        assert_eq!(alert.ttl, DEFAULT_TTL - 1);
    }
}