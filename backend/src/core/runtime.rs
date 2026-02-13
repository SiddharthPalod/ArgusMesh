use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

/// Global runtime tag for simple status reporting.
static RUNTIME_TAG: OnceCell<Arc<Mutex<Option<String>>>> = OnceCell::new();

/// Simple flag to ensure we only start the mesh node once.
static MESH_STARTED: OnceCell<()> = OnceCell::new();

pub fn set_runtime_tag(tag: String) {
    let cell = RUNTIME_TAG.get_or_init(|| Arc::new(Mutex::new(None)));
    if let Ok(mut guard) = cell.lock() {
        *guard = Some(tag);
    }
}

pub fn get_runtime_tag() -> Option<String> {
    let cell = RUNTIME_TAG.get()?;
    cell.lock().ok().and_then(|g| g.clone())
}

pub fn mark_mesh_started() {
    let _ = MESH_STARTED.set(());
}

pub fn has_mesh_runtime() -> bool {
    MESH_STARTED.get().is_some()
}