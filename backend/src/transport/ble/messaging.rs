use uuid::Uuid;
use std::collections::HashMap;
use std::time::{Instant, Duration};

#[derive(Debug)]
pub enum Ack {
    Ack(Uuid),
    Nack(Uuid),
}

pub fn encode_ack(id: Uuid, ok: bool) -> Vec<u8> {
    let mut v = Vec::with_capacity(17);
    v.push(if ok { 1 } else { 0 });
    v.extend_from_slice(id.as_bytes());
    v
}

pub fn decode_ack(data: &[u8]) -> Option<Ack> {
    if data.len() != 17 {
        return None;
    }
    let id = Uuid::from_slice(&data[1..]).ok()?;

    Some(if data[0] == 1 {
        Ack::Ack(id)
    } else {
        Ack::Nack(id)
    })
}

/// Returns true if a raw packet looks like an ACK/NACK (exactly 17 bytes).
pub fn is_ack_packet(data: &[u8]) -> bool {
    data.len() == 17
}

/// Tracks pending outbound messages waiting for acknowledgement.
pub struct AckTracker {
    pending: HashMap<Uuid, Instant>,
    acked: Vec<Uuid>,
    timeout: Duration,
}

impl AckTracker {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            pending: HashMap::new(),
            acked: Vec::new(),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Register an outbound message as pending.
    pub fn register(&mut self, msg_id: Uuid) {
        self.pending.insert(msg_id, Instant::now());
    }

    /// Process an incoming ack packet.  Returns true if the ack was for a
    /// known pending message.
    pub fn receive_ack(&mut self, ack: &Ack) -> bool {
        let id = match ack {
            Ack::Ack(id) | Ack::Nack(id) => *id,
        };
        if self.pending.remove(&id).is_some() {
            self.acked.push(id);
            true
        } else {
            false
        }
    }

    /// Check whether a specific message was acknowledged.
    pub fn is_acked(&self, id: &Uuid) -> bool {
        self.acked.contains(id)
    }

    /// Number of messages still awaiting ack.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Number of messages already acknowledged.
    pub fn acked_count(&self) -> usize {
        self.acked.len()
    }

    /// Remove entries older than the configured timeout.
    pub fn prune_expired(&mut self) {
        let now = Instant::now();
        self.pending.retain(|_, ts| now.duration_since(*ts) < self.timeout);
    }

    /// Combined stats: (pending, acked).
    pub fn stats(&self) -> (usize, usize) {
        (self.pending.len(), self.acked.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ack_encode_decode_roundtrip() {
        let id = Uuid::new_v4();
        let raw = encode_ack(id, true);
        assert_eq!(raw.len(), 17);
        assert!(is_ack_packet(&raw));

        match decode_ack(&raw) {
            Some(Ack::Ack(decoded_id)) => assert_eq!(decoded_id, id),
            _ => panic!("Expected Ack"),
        }
    }

    #[test]
    fn nack_encode_decode_roundtrip() {
        let id = Uuid::new_v4();
        let raw = encode_ack(id, false);
        match decode_ack(&raw) {
            Some(Ack::Nack(decoded_id)) => assert_eq!(decoded_id, id),
            _ => panic!("Expected Nack"),
        }
    }

    #[test]
    fn invalid_length_returns_none() {
        assert!(decode_ack(&[1, 2, 3]).is_none());
        assert!(!is_ack_packet(&[1, 2, 3]));
    }

    #[test]
    fn ack_tracker_register_and_receive() {
        let mut tracker = AckTracker::new(30);
        let id = Uuid::new_v4();

        tracker.register(id);
        assert_eq!(tracker.pending_count(), 1);
        assert!(!tracker.is_acked(&id));

        let ack = Ack::Ack(id);
        assert!(tracker.receive_ack(&ack));
        assert_eq!(tracker.pending_count(), 0);
        assert!(tracker.is_acked(&id));
    }

    #[test]
    fn ack_tracker_unknown_id_ignored() {
        let mut tracker = AckTracker::new(30);
        let unknown = Ack::Ack(Uuid::new_v4());
        assert!(!tracker.receive_ack(&unknown));
    }
}