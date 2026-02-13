use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct Versioned{
    pub id: Uuid,
    pub version: u32,
    pub data: Vec<u8>,
}

pub struct ConflictIndex{
    map: HashMap<Uuid, Versioned>,
}

impl ConflictIndex{
    pub fn new() -> Self{
        Self {
            map: HashMap::new(),
        }
    }

    pub fn merge(&mut self, incoming: Versioned)-> bool {
        match self.map.get(&incoming.id){
            Some(existing) => {
                if incoming.version > existing.version{
                    self.map.insert(incoming.id, incoming);
                    true
                }
                else{
                    false
                }
            },
            None => {
                self.map.insert(incoming.id, incoming);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn v(id: Uuid, version: u32) -> Versioned {
        Versioned {
            id,
            version,
            data: vec![version as u8],
        }
    }

    #[test]
    fn latest_version_wins() {
        let id = Uuid::new_v4();
        let mut idx = ConflictIndex::new();

        assert!(idx.merge(v(id, 1)));
        assert!(idx.merge(v(id, 2)));

        assert_eq!(idx.map.get(&id).unwrap().version, 2);
    }

    #[test]
    fn older_version_rejected() {
        let id = Uuid::new_v4();
        let mut idx = ConflictIndex::new();

        idx.merge(v(id, 5));
        let accepted = idx.merge(v(id, 3));

        assert!(!accepted);
        assert_eq!(idx.map.get(&id).unwrap().version, 5);
    }
}
