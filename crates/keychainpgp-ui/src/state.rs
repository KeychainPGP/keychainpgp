//! Application state management.

use std::path::Path;
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
    /// Initialize the application state using platform-default data directory.
    #[cfg(desktop)]
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

    /// Initialize the application state using an explicit data directory.
    ///
    /// On mobile, `directories::ProjectDirs` does not work, so we use
    /// the app data dir provided by Tauri.
    pub fn initialize_with_dir(data_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = SequoiaEngine::new();
        let keyring = Keyring::open_at(data_dir).map_err(|e| {
            tracing::error!("failed to open keyring at {}: {e}", data_dir.display());
            e
        })?;

        Ok(Self {
            engine,
            keyring: Mutex::new(keyring),
            passphrase_cache: Mutex::new(PassphraseCache::new(DEFAULT_CACHE_TTL)),
        })
    }
}
