//! Application state management.

use std::sync::Mutex;

use keychainpgp_core::SequoiaEngine;
use keychainpgp_keys::Keyring;

use crate::passphrase_cache::PassphraseCache;

/// Default passphrase cache TTL in seconds (10 minutes).
const DEFAULT_CACHE_TTL: u64 = 600;

/// Shared application state managed by Tauri.
pub struct AppState {
    pub engine: SequoiaEngine,
    pub keyring: Mutex<Keyring>,
    pub passphrase_cache: Mutex<PassphraseCache>,
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
            passphrase_cache: Mutex::new(PassphraseCache::new(DEFAULT_CACHE_TTL)),
        })
    }
}
