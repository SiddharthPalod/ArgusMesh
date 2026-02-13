/// Service for managing storage configuration and operations.
/// 
/// Provides a clean interface for storage path configuration
/// and abstracts storage initialization details.

use std::path::PathBuf;
use crate::error::{MeshError, MeshResult};

/// Service for storage operations.
pub struct StorageService;

impl StorageService {
    /// Configures the base directory for persistent storage.
    /// Creates the directory if it doesn't exist.
    pub fn configure_base_dir(base_dir: String) -> MeshResult<()> {
        let mut path = PathBuf::from(&base_dir);
        
        // Ensure directory exists
        std::fs::create_dir_all(&path)
            .map_err(|e| MeshError::Storage(format!("Failed to create directory: {}", e)))?;
        
        path.push("argus_alert_index.db");
        
        let path_str = path.to_str()
            .ok_or_else(|| MeshError::Configuration("Invalid storage path".to_string()))?;
        
        // set_var is unsafe in some contexts, but safe here as we control the environment
        unsafe {
            std::env::set_var("ARGUS_ALERT_INDEX_PATH", path_str);
        }
        
        // Force DB to reload with new path (if it was already opened at default path)
        crate::core::alert_index::reload_database();
        
        Ok(())
    }

    /// Gets the configured storage path.
    pub fn get_storage_path() -> Option<String> {
        std::env::var("ARGUS_ALERT_INDEX_PATH").ok()
    }
}
