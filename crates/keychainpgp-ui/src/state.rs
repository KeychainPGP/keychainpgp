//! Application state management.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use keychainpgp_core::SequoiaEngine;
use keychainpgp_keys::Keyring;
use zeroize::Zeroizing;

use crate::passphrase_cache::PassphraseCache;

/// Default passphrase cache TTL in seconds (10 minutes).
const DEFAULT_CACHE_TTL: u64 = 600;

/// Detect portable mode by looking for a `.portable` marker file next to the executable.
///
/// Returns `Some(data_dir)` if portable mode is detected, where `data_dir` is
/// `{exe_dir}/data/`. Returns `None` for normal (installed) mode.
#[cfg(desktop)]
pub fn detect_portable_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    if exe_dir.join(".portable").exists() {
        Some(exe_dir.join("data"))
    } else {
        None
    }
}

/// Shared application state managed by Tauri.
pub struct AppState {
    pub engine: SequoiaEngine,
    pub keyring: Mutex<Keyring>,
    pub passphrase_cache: Mutex<PassphraseCache>,
    /// Whether OPSEC mode is currently active.
    pub opsec_mode: AtomicBool,
    /// Whether closing the window hides to system tray instead of quitting.
    pub close_to_tray: AtomicBool,
    /// In OPSEC mode, secret keys live here (RAM only), not in OS credential store.
    /// Maps fingerprint → secret key bytes (auto-zeroized on drop).
    pub opsec_secret_keys: Mutex<HashMap<String, Zeroizing<Vec<u8>>>>,
    /// Whether the app is running in portable mode (.portable marker detected).
    pub portable: bool,
    /// In portable mode, the data directory next to the executable.
    pub portable_dir: Option<PathBuf>,
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
            opsec_mode: AtomicBool::new(false),
            close_to_tray: AtomicBool::new(false),
            opsec_secret_keys: Mutex::new(HashMap::new()),
            portable: false,
            portable_dir: None,
        })
    }

    /// Initialize the application state using an explicit data directory.
    ///
    /// On mobile, `directories::ProjectDirs` does not work, so we use
    /// the app data dir provided by Tauri.
    #[allow(dead_code)] // Used on mobile via cfg(mobile)
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
            opsec_mode: AtomicBool::new(false),
            close_to_tray: AtomicBool::new(false),
            opsec_secret_keys: Mutex::new(HashMap::new()),
            portable: false,
            portable_dir: None,
        })
    }
}
