use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use flutter_rust_bridge::frb;

#[frb]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub msg_id: Uuid,
    pub sender_id: String,

    pub created_at: SystemTime,
    pub ttl_secs: u64,

    pub hop_count: u8,
    pub max_hops: u8,

    pub priority: Priority,

    // 🔐 encrypted payload
    pub payload: Vec<u8>,

    // 🔐 security fields (signature as Vec for serde array length limits)
    pub sender_pubkey: [u8; 32],
    pub signature: Vec<u8>,
    pub nonce: [u8; 12],
}

impl Envelope {
    pub fn new(sender: String, priority: Priority, payload: Vec<u8>) -> Self {
        Self {
            msg_id: Uuid::new_v4(),
            sender_id: sender,
            created_at: SystemTime::now(),
            ttl_secs: 300,
            hop_count: 0,
            max_hops: 10,
            priority,
            payload,
            sender_pubkey: [0; 32],
            signature: vec![0; 64],
            nonce: [0; 12],
        }
    }

    pub fn expired(&self) -> bool {
        self.created_at
            .elapsed()
            .unwrap_or(Duration::ZERO)
            .as_secs()
            > self.ttl_secs
    }

    pub fn next_hop(&mut self) -> bool {
        if self.hop_count >= self.max_hops {
            return false;
        }
        self.hop_count += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn hop_increment_stops_at_max() {
        let mut env = Envelope::new("n".into(), Priority::Low, vec![]);
        env.max_hops = 2;

        assert!(env.next_hop()); // hop = 1
        assert!(env.next_hop()); // hop = 2
        assert!(!env.next_hop()); // blocked
    }

    #[test]
    fn ttl_exhaustion_hops() {
        let mut env = Envelope::new("n".into(), Priority::Normal, vec![]);
        env.max_hops = 3;

        assert!(env.next_hop());
        assert!(env.next_hop());
        assert!(env.next_hop());
        assert!(!env.next_hop());
    }

    #[test]
    fn expired_by_time() {
        let mut env = Envelope::new("n".into(), Priority::Normal, vec![]);
        env.ttl_secs = 1;
        env.created_at = SystemTime::now() - Duration::from_secs(10);

        assert!(env.expired());
    }

    #[test]
    fn not_expired_when_recent() {
        let env = Envelope::new("n".into(), Priority::Normal, vec![]);
        assert!(!env.expired());
    }
}
