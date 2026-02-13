use std::collections::HashMap;
use std::time::{Instant, Duration};
use uuid::Uuid;


pub struct Reassembler {
    map: HashMap<Uuid, Entry>,
}

struct Entry{
    parts: Vec<Option<Vec<u8>>>,
    last_update: Instant,
}

impl Reassembler{
    pub fn new() -> Self{
        Self{
            map: HashMap::new(),
        }
    }

    pub fn push (&mut self, packet: Vec<u8>) -> Option<Vec<u8>>{
        // 1. Prune expired entries
        self.prune_expired();

        if packet.len() < 20 {
            return None; // Too short for header
        }

        let id = Uuid::from_slice(&packet[..16]).ok()?;
        let seq = u16::from_be_bytes([packet[16], packet[17]]) as usize;
        let total = u16::from_be_bytes([packet[18], packet[19]]) as usize;

        let data = packet[20..].to_vec();

        let entry = self.map.entry(id).or_insert_with(|| Entry {
            parts: vec![None; total],
            last_update: Instant::now(),
        });
        
        if seq >= total {
             // Invalid sequence index
             return None;
        }

        entry.parts[seq] = Some(data);
        entry.last_update = Instant::now();

        if entry.parts.iter().all(|x| x.is_some()) {
            let full = entry.parts.iter().flat_map(|x| x.clone().unwrap()).collect();
            self.map.remove(&id);
            Some(full)
        } else {
            None
        }
    }

    fn prune_expired(&mut self) {
        let now = Instant::now();
        self.map.retain(|_, entry| {
            now.duration_since(entry.last_update) < Duration::from_secs(30)
        });
    }

    pub fn metrics(&self) -> (usize, usize) {
        (self.map.len(), self.map.values().map(|e| e.parts.len()).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::ble::fragment::fragment;

    #[test]
    fn test_reassembly_single() {
        let mut r = Reassembler::new();
        let id = Uuid::new_v4();
        let data = vec![1, 2, 3, 4];
        
        // Manual fragment creation for single packet
        // Header: 16 (UUID) + 2 (seq=0) + 2 (total=1) + Payload
        let mut packet = Vec::new();
        packet.extend_from_slice(id.as_bytes());
        packet.extend_from_slice(&(0u16).to_be_bytes());
        packet.extend_from_slice(&(1u16).to_be_bytes()); 
        packet.extend_from_slice(&data);

        let res = r.push(packet);
        assert_eq!(res, Some(data));
    }

    #[test]
    fn test_reassembly_multi() {
        let mut r = Reassembler::new();
        let id = Uuid::new_v4();
        let data = vec![0u8; 300]; // Should be 2 fragments (chunk=160)
        
        // Use the fragment helper we know works from previous test
        let parts = fragment(id, &data);
        assert_eq!(parts.len(), 2);

        // Push part 0
        let res1 = r.push(parts[0].clone());
        assert_eq!(res1, None);

        // Push part 1
        let res2 = r.push(parts[1].clone());
        assert_eq!(res2, Some(data));
    }

    #[test]
    fn test_reassembly_out_of_order() {
        let mut r = Reassembler::new();
        let id = Uuid::new_v4();
        let data = vec![0u8; 300]; 
        let parts = fragment(id, &data);

        // Push part 1 first
        let res1 = r.push(parts[1].clone());
        assert_eq!(res1, None);

        // Push part 0 second
        let res2 = r.push(parts[0].clone());
        assert_eq!(res2, Some(data));
    }
}