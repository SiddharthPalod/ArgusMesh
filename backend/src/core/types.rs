use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn generate_ephemeral() -> Self {
        NodeId(Uuid::new_v4())
    }
}

#[derive(Debug, Clone,Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertType {
    Intrusion,
    Drone,
    Movement
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4
}