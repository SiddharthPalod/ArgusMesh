use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

use crate::routing::envelope::{Envelope, Priority};
use serde::{Deserialize, Serialize};

/// Stored representation of an Envelope (created_at as u64 for persistence).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEnvelope {
    pub msg_id: uuid::Uuid,
    pub sender_id: String,
    pub created_at_secs: u64,
    pub ttl_secs: u64,
    pub hop_count: u8,
    pub max_hops: u8,
    pub priority: Priority,
    pub payload: Vec<u8>,
    pub sender_pubkey: [u8; 32],
    pub signature: Vec<u8>,
    pub nonce: [u8; 12],
}

impl From<&Envelope> for StoredEnvelope {
    fn from(env: &Envelope) -> Self {
        let created_at_secs = env
            .created_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            msg_id: env.msg_id,
            sender_id: env.sender_id.clone(),
            created_at_secs,
            ttl_secs: env.ttl_secs,
            hop_count: env.hop_count,
            max_hops: env.max_hops,
            priority: env.priority,
            payload: env.payload.clone(),
            sender_pubkey: env.sender_pubkey,
            signature: env.signature.clone(),
            nonce: env.nonce,
        }
    }
}

impl From<StoredEnvelope> for Envelope {
    fn from(s: StoredEnvelope) -> Self {
        let created_at = UNIX_EPOCH + std::time::Duration::from_secs(s.created_at_secs);
        Self {
            msg_id: s.msg_id,
            sender_id: s.sender_id,
            created_at,
            ttl_secs: s.ttl_secs,
            hop_count: s.hop_count,
            max_hops: s.max_hops,
            priority: s.priority,
            payload: s.payload,
            sender_pubkey: s.sender_pubkey,
            signature: s.signature,
            nonce: s.nonce,
        }
    }
}

static STORE: Lazy<Mutex<Option<sled::Db>>> = Lazy::new(|| Mutex::new(None));

/// Force reload the database (useful after path is configured).
/// Closes the old DB and opens a new one at the current path.
pub fn reload_database() {
    // Close old DB if any
    if let Ok(mut guard) = STORE.lock() {
        if let Some(db) = guard.take() {
            let _ = db.flush();
            drop(db);
        }
    }
    
    // Open new DB at current path
    ensure_db_open();
}

fn ensure_db_open() {
    let mut guard = match STORE.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    
    // If DB is already open, we're good
    if guard.is_some() {
        return;
    }
    
    // Otherwise, try to open it
    let path = std::env::var("ARGUS_ALERT_INDEX_PATH")
        .unwrap_or_else(|_| "argus_alert_index.db".into());
    log::info!("[Rust] Opening Sled at path: {}", path);
    match sled::open(&path) {
        Ok(db) => {
            log::info!("[Rust] Sled opened successfully.");
            *guard = Some(db);
        }
        Err(e) => {
            log::warn!("[Rust] Sled open failed: {:?}", e);
        }
    }
}

fn with_db<F, R>(f: F) -> R
where
    F: FnOnce(&sled::Db) -> R,
    R: Default,
{
    ensure_db_open();
    if let Ok(guard) = STORE.lock() {
        if let Some(ref db) = *guard {
            return f(db);
        }
    }
    R::default()
}

pub fn add_alert(env: Envelope) {
    let stored = StoredEnvelope::from(&env);
    let key = env.msg_id.as_bytes();
    if let Ok(serialized) = bincode::serialize(&stored) {
        with_db::<_, ()>(|db| {
            let _ = db.insert(key, serialized);
        });
    }
    // Keep in-memory cache for list_alerts when DB is not available
    in_memory_add(env);
}

static MEMORY: Lazy<Mutex<Vec<Envelope>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn in_memory_add(env: Envelope) {
    if let Ok(mut v) = MEMORY.lock() {
        if !v.iter().any(|e| e.msg_id == env.msg_id) {
            v.push(env);
        }
    }
}

fn load_from_db() -> Vec<Envelope> {
    let mut out = Vec::new();
    with_db::<_, ()>(|db| {
        for item in db.iter().flatten() {
            let (_, value) = item;
            if let Ok(s) = bincode::deserialize::<StoredEnvelope>(&value) {
                out.push(Envelope::from(s));
            }
        }
    });
    out
}

pub fn list_alerts() -> Vec<Envelope> {
    // Always try to load from DB first to ensure we have latest data
    // Then check memory cache
    let from_db = load_from_db();
    if let Ok(mut guard) = MEMORY.lock() {
        if !guard.is_empty() && from_db.is_empty() {
            // If DB is empty but memory has data, use memory (might be new alerts not yet persisted)
            return guard.clone();
        }
        // Update memory cache with DB data
        *guard = from_db.clone();
    }
    from_db
}

/// Explicitly load alerts from DB into memory cache (call on startup).
pub fn load_alerts_from_db() {
    let from_db = load_from_db();
    if let Ok(mut guard) = MEMORY.lock() {
        *guard = from_db;
    }
}

pub fn clear_alerts() {
    if let Ok(mut v) = MEMORY.lock() {
        v.clear();
    }
    with_db::<_, ()>(|db| {
        let _ = db.clear();
    });
}

const RUNTIME_TAG_KEY: &[u8] = b"__runtime_tag__";

/// Persist runtime tag for background/restart recovery.
pub fn set_persisted_runtime_tag(tag: &str) {
    with_db::<_, ()>(|db| {
        let _ = db.insert(RUNTIME_TAG_KEY, tag.as_bytes());
    });
}

/// Restore runtime tag from persistence (e.g. after app was backgrounded and restarted).
pub fn get_persisted_runtime_tag() -> Option<String> {
    let mut out = None;
    with_db::<_, ()>(|db| {
        if let Ok(Some(bytes)) = db.get(RUNTIME_TAG_KEY) {
            out = String::from_utf8(bytes.to_vec()).ok();
        }
    });
    out
}
