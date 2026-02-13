use uuid::Uuid;

pub const CHUNK: usize = 160;

pub fn fragment(msg_id:Uuid, data: &[u8]) -> Vec<Vec<u8>> {
    let total = (data.len() + CHUNK - 1) / CHUNK;

    data.chunks(CHUNK)
        .enumerate()
        .map(|(i, chunk)| {
            let mut out = Vec::new();
            out.extend_from_slice(msg_id.as_bytes());
            out.extend_from_slice(&(i as u16).to_be_bytes());
            out.extend_from_slice(&(total as u16).to_be_bytes());
            out.extend_from_slice(chunk);
            out
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_small() {
        let id = Uuid::new_v4();
        let data = vec![1, 2, 3, 4];
        let parts = fragment(id, &data);

        assert_eq!(parts.len(), 1);
        let part = &parts[0];
        
        // Header: 16 (UUID) + 2 (seq) + 2 (total) = 20 bytes
        assert!(part.len() > 20);
        assert_eq!(part[16..18], [0, 0]); // seq 0
        assert_eq!(part[18..20], [0, 1]); // total 1
        assert_eq!(&part[20..], &data[..]);
    }

    #[test]
    fn test_fragment_large() {
        let id = Uuid::new_v4();
        let data = vec![0u8; CHUNK + 10]; // 1 full chunk + 10 bytes
        let parts = fragment(id, &data);

        assert_eq!(parts.len(), 2);

        // Check first part
        assert_eq!(parts[0][16..18], [0, 0]); // seq 0
        assert_eq!(parts[0][18..20], [0, 2]); // total 2
        assert_eq!(parts[0].len(), 20 + CHUNK);

        // Check second part
        assert_eq!(parts[1][16..18], [0, 1]); // seq 1
        assert_eq!(parts[1][18..20], [0, 2]); // total 2
        assert_eq!(parts[1].len(), 20 + 10);
    }
}