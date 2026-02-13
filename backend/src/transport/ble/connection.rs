use std::collections::HashMap;

pub struct ConnectionTable{
    sessions: HashMap<String, bool>
}

impl ConnectionTable {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn connect(&mut self, id: String) {
        self.sessions.insert(id, true);
    }

    pub fn disconnect(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    pub fn is_connected(&self, id: &str) -> bool {
        self.sessions.get(id).cloned().unwrap_or(false)
    }
}