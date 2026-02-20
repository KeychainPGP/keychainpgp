//! Application state management.

use std::sync::Mutex;

use keychainpgp_core::SequoiaEngine;
use keychainpgp_keys::Keyring;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub engine: SequoiaEngine,
    pub keyring: Mutex<Keyring>,
}

impl AppState {
    /// Initialize the application state.
    pub fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        let engine = SequoiaEngine::new();
        let keyring = Keyring::open_default().map_err(|e| {
            tracing::error!("failed to open keyring: {e}");
            e
        })?;

        Ok(Self {
            engine,
            keyring: Mutex::new(keyring),
        })
    }
}
